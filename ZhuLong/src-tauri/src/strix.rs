use crate::{ScanConfig, db};
use crate::python_embed;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Command, Child as TokioChild};

type ScanProcesses = Arc<Mutex<HashMap<i64, TokioChild>>>;

pub async fn run_scan(
    app: &AppHandle,
    scan_id: i64,
    config: ScanConfig,
    processes: ScanProcesses,
) -> Result<(), String> {
    // 获取 Strix 路径
    let strix_path = get_strix_path(app)?;
    let strix_dir = strix_path.parent()
        .ok_or("无法获取 Strix 目录")?
        .parent()
        .ok_or("无法获取 Strix 根目录")?
        .to_path_buf();
    
    // 设置环境变量
    let mut env = std::env::vars().collect::<HashMap<_, _>>();
    env.insert("STRIX_LLM".to_string(), config.llm_provider.clone());
    env.insert("LLM_API_KEY".to_string(), config.llm_api_key.clone());
    
    if let Some(base) = config.llm_api_base {
        env.insert("LLM_API_BASE".to_string(), base);
    }
    
    // 构建命令行参数
    let mut args = vec!["strix_main".to_string(), "--target".to_string()];
    for target in &config.targets {
        args.push(target.clone());
    }
    
    if let Some(instruction) = &config.instruction {
        args.push("--instruction".to_string());
        args.push(instruction.clone());
    }
    
    if let Some(run_name) = &config.run_name {
        args.push("--run-name".to_string());
        args.push(run_name.clone());
    }
    
    if config.non_interactive {
        args.push("--non-interactive".to_string());
    }
    
    let workspace_dir = get_workspace_dir(app)?;
    std::fs::create_dir_all(&workspace_dir)
        .map_err(|e| format!("创建工作目录失败: {}", e))?;
    
    // 更新状态为运行中
    db::update_scan_status(app, scan_id, "running", "扫描进行中...").await
        .map_err(|e| format!("更新扫描状态失败: {}", e))?;
    
    // 尝试使用嵌入的 Python，如果失败则回退到系统 Python
    let use_embedded = std::env::var("USE_EMBEDDED_PYTHON")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    if use_embedded {
        // 使用嵌入的 Python 解释器（不使用进程管理）
        return run_with_embedded_python(
            app,
            scan_id,
            strix_path,
            strix_dir,
            args,
            env,
            processes.clone(),
        ).await;
    }
    
    // 使用 Python（优先打包的，回退到系统）
    let python_cmd = get_python_executable(app)?;
    
    let mut cmd = Command::new(&python_cmd);
    cmd.arg(&strix_path)
        .arg("--target");
    
    for target in &config.targets {
        cmd.arg(target);
    }
    
    if let Some(instruction) = &config.instruction {
        cmd.arg("--instruction").arg(instruction);
    }
    
    if let Some(run_name) = &config.run_name {
        cmd.arg("--run-name").arg(run_name);
    }
    
    if config.non_interactive {
        cmd.arg("--non-interactive");
    }
    
    cmd.envs(&env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&workspace_dir);
    
    // 启动进程
    let mut child = cmd.spawn().map_err(|e| format!("启动 Strix 失败: {}", e))?;
    
    // 获取 stdout 和 stderr
    let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
    let stderr = child.stderr.take().ok_or("无法获取 stderr")?;
    
    // 保存进程引用到状态
    {
        let mut procs = processes.lock().unwrap();
        procs.insert(scan_id, child);
    }
    
    // 读取输出并发送事件
    let app_clone = app.clone();
    
    // 读取 stdout
    let app_stdout = app_clone.clone();
    let scan_id_stdout = scan_id;
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            if !line.trim().is_empty() {
                // 发送日志事件到前端
                let _ = app_stdout.emit("scan-log", serde_json::json!({
                    "scan_id": scan_id_stdout,
                    "level": "info",
                    "message": line
                }));
                
                // 保存日志到数据库
                let _ = db::add_scan_log(&app_stdout, scan_id_stdout, "info", &line).await;
            }
        }
    });
    
    // 读取 stderr
    let app_stderr = app_clone.clone();
    let scan_id_stderr = scan_id;
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            if !line.trim().is_empty() {
                // 发送错误日志事件
                let _ = app_stderr.emit("scan-log", serde_json::json!({
                    "scan_id": scan_id_stderr,
                    "level": "error",
                    "message": line
                }));
                
                // 保存日志到数据库
                let _ = db::add_scan_log(&app_stderr, scan_id_stderr, "error", &line).await;
            }
        }
    });
    
    // 更新状态为运行中
    db::update_scan_status(app, scan_id, "running", "扫描进行中...").await
        .map_err(|e| format!("更新扫描状态失败: {}", e))?;
    
    // 等待进程完成
    // 使用定期检查的方式等待进程完成
    use tokio::time::{sleep, Duration};
    let status = loop {
        sleep(Duration::from_millis(500)).await;
        
        let mut child_guard = processes.lock().unwrap();
        if let Some(child) = child_guard.get_mut(&scan_id) {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // 进程已完成
                    child_guard.remove(&scan_id);
                    break status;
                }
                Ok(None) => {
                    // 进程仍在运行，继续等待
                    drop(child_guard);
                    continue;
                }
                Err(e) => {
                    child_guard.remove(&scan_id);
                    return Err(format!("等待进程失败: {}", e));
                }
            }
        } else {
            return Err("进程未找到".to_string());
        }
    };
    
    // 更新数据库状态
    if status.success() {
        db::update_scan_status(app, scan_id, "completed", "扫描完成").await
            .map_err(|e| format!("更新扫描状态失败: {}", e))?;
    } else {
        db::update_scan_status(app, scan_id, "failed", "扫描失败").await
            .map_err(|e| format!("更新扫描状态失败: {}", e))?;
    }
    
    // 解析结果
    parse_scan_results(app, scan_id).await?;
    
    Ok(())
}

pub async fn stop_scan(
    app: &AppHandle,
    scan_id: i64,
    processes: ScanProcesses,
) -> Result<(), String> {
    // 先获取子进程，然后释放锁
    let child = {
        let mut procs = processes.lock().unwrap();
        procs.remove(&scan_id)
    };
    
    if let Some(mut child) = child {
        // 系统 Python 进程，可以停止
        child.kill().await.map_err(|e| format!("停止扫描失败: {}", e))?;
        db::update_scan_status(app, scan_id, "stopped", "用户停止").await
            .map_err(|e| format!("更新扫描状态失败: {}", e))?;
        
        // 发送停止事件
        let _ = app.emit("scan-log", serde_json::json!({
            "scan_id": scan_id,
            "level": "info",
            "message": "扫描已由用户停止"
        }));
    } else {
        // 可能是嵌入 Python 执行的扫描，无法直接停止
        // 更新状态为停止
        db::update_scan_status(app, scan_id, "stopped", "用户停止（嵌入 Python 执行中，无法强制终止）").await
            .map_err(|e| format!("更新扫描状态失败: {}", e))?;
        
        // 发送停止事件
        let _ = app.emit("scan-log", serde_json::json!({
            "scan_id": scan_id,
            "level": "warning",
            "message": "扫描标记为停止，但嵌入 Python 执行可能仍在进行中"
        }));
    }
    
    Ok(())
}

fn get_strix_path(app: &AppHandle) -> Result<PathBuf, String> {
    // 尝试从资源目录获取
    if let Ok(resource_dir) = app.path().resource_dir() {
        let strix_main = resource_dir
            .join("strix-0.4.0")
            .join("strix")
            .join("interface")
            .join("main.py");
        
        if strix_main.exists() {
            return Ok(strix_main);
        }
    }
    
    // 尝试从项目目录获取（开发环境）
    let project_dir = std::env::current_dir()
        .map_err(|_| "无法获取当前目录")?;
    
    let strix_main = project_dir
        .join("strix-0.4.0")
        .join("strix")
        .join("interface")
        .join("main.py");
    
    if strix_main.exists() {
        return Ok(strix_main);
    }
    
    Err("Strix 主程序未找到，请确保 strix-0.4.0 目录存在".to_string())
}

fn get_workspace_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;
    
    Ok(app_dir.join("workspace"))
}

/// 获取打包的 Python 解释器路径
fn get_bundled_python(app: &AppHandle) -> Option<PathBuf> {
    // 尝试从资源目录获取（打包后）
    if let Ok(resource_dir) = app.path().resource_dir() {
        #[cfg(target_os = "windows")]
        let python_exe = resource_dir.join("python-runtime").join("python").join("python.exe");
        #[cfg(not(target_os = "windows"))]
        let python_exe = resource_dir.join("python-runtime").join("python").join("python3");
        
        if python_exe.exists() {
            return Some(python_exe);
        }
    }
    
    // 尝试从项目目录获取（开发环境）
    if let Ok(project_dir) = std::env::current_dir() {
        #[cfg(target_os = "windows")]
        let python_exe = project_dir.join("python-runtime").join("python").join("python.exe");
        #[cfg(not(target_os = "windows"))]
        let python_exe = project_dir.join("python-runtime").join("python").join("python3");
        
        if python_exe.exists() {
            return Some(python_exe);
        }
    }
    
    None
}

/// 获取 Python 解释器路径（优先级：打包的 Python > 系统 Python）
fn get_python_executable(app: &AppHandle) -> Result<String, String> {
    // 1. 优先使用打包的 Python
    if let Some(bundled_python) = get_bundled_python(app) {
        let python_path = bundled_python.to_string_lossy().to_string();
        println!("✅ 使用打包的 Python: {}", python_path);
        return Ok(python_path);
    }
    
    // 2. 回退到系统 Python
    if std::process::Command::new("python")
        .arg("--version")
        .output()
        .is_ok()
    {
        println!("ℹ️  使用系统 Python: python");
        return Ok("python".to_string());
    }
    
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_ok()
    {
        println!("ℹ️  使用系统 Python: python3");
        return Ok("python3".to_string());
    }
    
    Err("未找到 Python 解释器。请运行 scripts/setup-python-runtime.ps1 打包 Python，或确保系统已安装 Python".to_string())
}

async fn parse_scan_results(app: &AppHandle, scan_id: i64) -> Result<(), String> {
    use std::fs;
    use serde_json::Value;
    
    // 获取扫描信息
    let scan_info = db::get_scan_info(app, scan_id)
        .map_err(|e| format!("获取扫描信息失败: {}", e))?;
    let run_name = scan_info.run_name;
    
    // Strix 结果通常保存在 strix_runs/<run_name> 目录
    let workspace_dir = get_workspace_dir(app)?;
    let results_dir = workspace_dir.join("strix_runs").join(&run_name);
    
    if !results_dir.exists() {
        return Ok(()); // 结果目录不存在，可能扫描还在进行中
    }
    
    // 查找报告文件（通常是 JSON 或 Markdown）
    let report_files = vec![
        results_dir.join("report.json"),
        results_dir.join("vulnerabilities.json"),
        results_dir.join("findings.json"),
    ];
    
    let mut vulnerabilities = Vec::new();
    
    for report_file in report_files {
        if report_file.exists() {
            match fs::read_to_string(&report_file) {
                Ok(content) => {
                    // 尝试解析 JSON
                    if let Ok(json_value) = serde_json::from_str::<Value>(&content) {
                        vulnerabilities.extend(parse_json_report(&json_value)?);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("读取报告文件失败: {}", e);
                }
            }
        }
    }
    
    // 如果没有找到 JSON，尝试解析 Markdown
    if vulnerabilities.is_empty() {
        let md_files = vec![
            results_dir.join("report.md"),
            results_dir.join("README.md"),
        ];
        
        for md_file in md_files {
            if md_file.exists() {
                if let Ok(content) = fs::read_to_string(&md_file) {
                    vulnerabilities.extend(parse_markdown_report(&content)?);
                    break;
                }
            }
        }
    }
    
    // 保存漏洞到数据库
    for vuln in vulnerabilities {
        db::save_vulnerability(app, scan_id, &vuln)
            .map_err(|e| format!("保存漏洞失败: {}", e))?;
    }
    
    // 更新结果路径
    db::update_scan_results_path(app, scan_id, &results_dir.to_string_lossy())
        .map_err(|e| format!("更新结果路径失败: {}", e))?;
    
    Ok(())
}

fn parse_json_report(json: &serde_json::Value) -> Result<Vec<crate::Vulnerability>, String> {
    let mut vulnerabilities = Vec::new();
    
    // 尝试不同的 JSON 结构
    if let Some(vulns_array) = json.get("vulnerabilities").and_then(|v| v.as_array()) {
        for vuln in vulns_array {
            if let Ok(v) = parse_vulnerability_json(vuln) {
                vulnerabilities.push(v);
            }
        }
    } else if let Some(findings_array) = json.get("findings").and_then(|v| v.as_array()) {
        for finding in findings_array {
            if let Ok(v) = parse_vulnerability_json(finding) {
                vulnerabilities.push(v);
            }
        }
    } else if json.is_array() {
        for item in json.as_array().unwrap() {
            if let Ok(v) = parse_vulnerability_json(item) {
                vulnerabilities.push(v);
            }
        }
    }
    
    Ok(vulnerabilities)
}

fn parse_vulnerability_json(v: &serde_json::Value) -> Result<crate::Vulnerability, String> {
    Ok(crate::Vulnerability {
        id: v.get("id")
            .or_else(|| v.get("vuln_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        title: v.get("title")
            .or_else(|| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("未命名漏洞")
            .to_string(),
        severity: v.get("severity")
            .or_else(|| v.get("level"))
            .and_then(|v| v.as_str())
            .unwrap_or("medium")
            .to_lowercase(),
        description: v.get("description")
            .or_else(|| v.get("detail"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        poc: v.get("poc")
            .or_else(|| v.get("proof_of_concept"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        remediation: v.get("remediation")
            .or_else(|| v.get("fix"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}

fn parse_markdown_report(md: &str) -> Result<Vec<crate::Vulnerability>, String> {
    let mut vulnerabilities = Vec::new();
    
    // 简单的 Markdown 解析
    // 查找 ## 或 ### 开头的标题作为漏洞标题
    let lines: Vec<&str> = md.lines().collect();
    let mut current_vuln: Option<crate::Vulnerability> = None;
    
    for (_i, line) in lines.iter().enumerate() {
        if line.starts_with("## ") || line.starts_with("### ") {
            // 保存前一个漏洞
            if let Some(vuln) = current_vuln.take() {
                vulnerabilities.push(vuln);
            }
            
            // 开始新漏洞
            let title = line.trim_start_matches('#').trim().to_string();
            current_vuln = Some(crate::Vulnerability {
                id: format!("vuln_{}", vulnerabilities.len() + 1),
                title,
                severity: "medium".to_string(),
                description: String::new(),
                poc: None,
                remediation: None,
            });
        } else if let Some(ref mut vuln) = current_vuln {
            // 解析漏洞详情
            if line.to_lowercase().contains("severity:") || line.to_lowercase().contains("严重程度:") {
                let severity = line.split(':').nth(1).unwrap_or("").trim().to_lowercase();
                vuln.severity = severity;
            } else if line.to_lowercase().contains("description:") || line.to_lowercase().contains("描述:") {
                // 描述可能跨多行
                let desc = line.split(':').nth(1).unwrap_or("").trim();
                if !desc.is_empty() {
                    vuln.description = desc.to_string();
                }
            } else if !line.trim().is_empty() && vuln.description.is_empty() {
                // 第一段非空文本作为描述
                vuln.description = line.trim().to_string();
            }
        }
    }
    
    // 保存最后一个漏洞
    if let Some(vuln) = current_vuln {
        vulnerabilities.push(vuln);
    }
    
    Ok(vulnerabilities)
}

/// 使用嵌入的 Python 解释器运行扫描
async fn run_with_embedded_python(
    app: &AppHandle,
    scan_id: i64,
    strix_path: PathBuf,
    strix_dir: PathBuf,
    args: Vec<String>,
    env: HashMap<String, String>,
    _processes: ScanProcesses,
) -> Result<(), String> {
    // 初始化 Python（如果尚未初始化）
    python_embed::init_python()
        .map_err(|e| format!("初始化 Python 失败: {}", e))?;
    
    // 在后台任务中运行 Python 代码（不阻塞主线程）
    let app_clone = app.clone();
    let scan_id_clone = scan_id;
    
    tokio::spawn(async move {
        // 运行 Python 代码并捕获输出
        match python_embed::run_strix_with_embedded_python(
            strix_path.clone(),
            strix_dir.clone(),
            args,
            env,
        ).await {
            Ok((stdout, stderr, exit_code)) => {
                // 处理输出
                for line in stdout.split('\n') {
                    let line = line.trim();
                    if !line.is_empty() {
                        let _ = app_clone.emit("scan-log", serde_json::json!({
                            "scan_id": scan_id_clone,
                            "level": "info",
                            "message": line
                        }));
                        let _ = db::add_scan_log(&app_clone, scan_id_clone, "info", line).await;
                    }
                }
                
                for line in stderr.split('\n') {
                    let line = line.trim();
                    if !line.is_empty() {
                        let _ = app_clone.emit("scan-log", serde_json::json!({
                            "scan_id": scan_id_clone,
                            "level": "error",
                            "message": line
                        }));
                        let _ = db::add_scan_log(&app_clone, scan_id_clone, "error", line).await;
                    }
                }
                
                // 更新状态
                if exit_code == 0 {
                    let _ = db::update_scan_status(&app_clone, scan_id_clone, "completed", "扫描完成").await;
                } else {
                    let _ = db::update_scan_status(&app_clone, scan_id_clone, "failed", "扫描失败").await;
                }
                
                // 解析结果
                let _ = parse_scan_results(&app_clone, scan_id_clone).await;
                
                // 解析结果
                let _ = parse_scan_results(&app_clone, scan_id_clone).await;
            }
            Err(e) => {
                let _ = db::update_scan_status(&app_clone, scan_id_clone, "failed", &format!("Python 执行错误: {}", e)).await;
            }
        }
    });
    
    // 立即返回，Python 代码在后台执行
    Ok(())
}

