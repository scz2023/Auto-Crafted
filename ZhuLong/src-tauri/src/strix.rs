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
    // 获取 Strix 路径（用于验证，但使用 Docker 时不需要）
    let _strix_path = get_strix_path(app)?;
    
    // 设置环境变量（不再需要，因为使用 Docker 时通过 -e 传递）
    // 保留这部分代码以防将来需要，但不使用 config.llm_api_base，避免移动
    let _env = std::env::vars().collect::<HashMap<_, _>>();
    
    // 构建命令行参数
    // sys.argv[0] 应该是脚本文件名，后续参数才是真正的命令行参数
    let mut args = vec!["--target".to_string()];
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
    
    // 首先检查是否有运行中的容器
    let mut running_container_name: Option<String> = None;
    
    // 方法1: 通过容器名称查找
    let ps_output = Command::new("docker")
        .arg("ps")
        .arg("--format")
        .arg("{{.Names}}")
        .arg("--filter")
        .arg("name=strix-scan-")
        .output()
        .await;
    
    if let Ok(output) = ps_output {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if let Some(first_line) = stdout.lines().next() {
                let name = first_line.trim();
                if !name.is_empty() {
                    running_container_name = Some(name.to_string());
                }
            }
        }
    }
    
    // 方法2: 通过标签查找
    if running_container_name.is_none() {
        let ps_output = Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("{{.Names}}")
            .arg("--filter")
            .arg("label=strix-scan-id")
            .output()
            .await;
        
        if let Ok(output) = ps_output {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if let Some(first_line) = stdout.lines().next() {
                    let name = first_line.trim();
                    if !name.is_empty() {
                        running_container_name = Some(name.to_string());
                    }
                }
            }
        }
    }
    
    // 构建 Docker 命令
    let workspace_dir_str = workspace_dir.to_string_lossy().replace('\\', "/");
    
    let mut cmd = Command::new("docker");
    
    if let Some(container_name) = running_container_name {
        // 使用运行中的容器执行扫描
        cmd.arg("exec")
            .arg("-i")  // 交互模式
            .arg("-w")
            .arg("/workspace")  // 设置工作目录
            .arg("-e")
            .arg(format!("STRIX_LLM={}", config.llm_provider))
            .arg("-e")
            .arg(format!("LLM_API_KEY={}", config.llm_api_key))
            .arg("-e")
            .arg("PYTHONPATH=/app");
        
        if let Some(base) = &config.llm_api_base {
            if !base.is_empty() {
                cmd.arg("-e").arg(format!("LLM_API_BASE={}", base));
            }
        }
        
        // 根据模型类型设置特定的 API Key 环境变量
        if config.llm_provider.starts_with("deepseek/") {
            cmd.arg("-e").arg(format!("DEEPSEEK_API_KEY={}", config.llm_api_key));
            if let Some(base) = &config.llm_api_base {
                if !base.is_empty() {
                    cmd.arg("-e").arg(format!("DEEPSEEK_API_BASE={}", base));
                }
            }
        } else if config.llm_provider.starts_with("openai/") {
            cmd.arg("-e").arg(format!("OPENAI_API_KEY={}", config.llm_api_key));
        } else if config.llm_provider.starts_with("anthropic/") {
            cmd.arg("-e").arg(format!("ANTHROPIC_API_KEY={}", config.llm_api_key));
        }
        
        // 构建 strix 命令字符串
        // 使用 bash -c 执行，确保能加载代理配置
        // 根据 pyproject.toml，strix 是 poetry 脚本，应该通过 poetry run 执行
        let mut strix_cmd_parts = vec!["cd /workspace".to_string()];
        strix_cmd_parts.push("source /etc/profile.d/proxy.sh 2>/dev/null || true".to_string());
        strix_cmd_parts.push("poetry run strix".to_string());
        
        // 添加 --target 参数（每个目标都需要单独的 --target）
        for target in &config.targets {
            strix_cmd_parts.push("--target".to_string());
            // 转义参数以安全地在 shell 中使用
            let escaped = format!("'{}'", target.replace('\'', "'\"'\"'"));
            strix_cmd_parts.push(escaped);
        }
        
        if let Some(instruction) = &config.instruction {
            strix_cmd_parts.push("--instruction".to_string());
            let escaped = format!("'{}'", instruction.replace('\'', "'\"'\"'"));
            strix_cmd_parts.push(escaped);
        }
        
        if let Some(run_name) = &config.run_name {
            strix_cmd_parts.push("--run-name".to_string());
            let escaped = format!("'{}'", run_name.replace('\'', "'\"'\"'"));
            strix_cmd_parts.push(escaped);
        }
        
        if config.non_interactive {
            strix_cmd_parts.push("--non-interactive".to_string());
        }
        
        let strix_cmd = strix_cmd_parts.join(" ");
        
        cmd.arg(&container_name)
            .arg("bash")
            .arg("-c")
            .arg(&strix_cmd);
    } else {
        // 查找 Strix 镜像
        let image_names = vec![
            "strix:0.4.0",
            "strix-agent:0.4.0",
            "usestrix/strix:0.4.0",
            "strix:latest",
            "ghcr.io/usestrix/strix-sandbox:0.1.10",
            "ghcr.io/usestrix/strix-sandbox:latest",
        ];
        
        let mut strix_image = None;
        for image_name in &image_names {
            let output = Command::new("docker")
                .arg("images")
                .arg("--format")
                .arg("{{.Repository}}:{{.Tag}}")
                .output()
                .await;
            
            if let Ok(output) = output {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if stdout.lines().any(|line| line.trim() == *image_name) {
                        strix_image = Some(image_name.to_string());
                        break;
                    }
                }
            }
        }
        
        let image_name = strix_image.ok_or_else(|| {
            "未找到 Strix Docker 镜像。请运行: cd strix-0.4.0 && docker build -f containers/Dockerfile -t strix:0.4.0 .".to_string()
        })?;
        
        // 创建新容器需要设置所有必需的环境变量
        // 查找可用端口
        use tokio::net::TcpListener;
        let caido_port = {
            let listener = TcpListener::bind("127.0.0.1:0").await
                .map_err(|e| format!("无法绑定端口: {}", e))?;
            listener.local_addr()
                .map_err(|e| format!("无法获取端口地址: {}", e))?
                .port()
        };
        
        let tool_server_port = {
            let listener = TcpListener::bind("127.0.0.1:0").await
                .map_err(|e| format!("无法绑定端口: {}", e))?;
            listener.local_addr()
                .map_err(|e| format!("无法获取端口地址: {}", e))?
                .port()
        };
        
        // 生成 token
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        let tool_server_token: String = (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        cmd.arg("run")
            .arg("--rm")
            .arg("-i")  // 交互模式，保持 stdin 打开
            .arg("-v")
            .arg(format!("{}:/workspace", workspace_dir_str))
            .arg("-e")
            .arg("PYTHONUNBUFFERED=1")
            .arg("-e")
            .arg(format!("CAIDO_PORT={}", caido_port))
            .arg("-e")
            .arg(format!("TOOL_SERVER_PORT={}", tool_server_port))
            .arg("-e")
            .arg(format!("TOOL_SERVER_TOKEN={}", tool_server_token))
            .arg("-e")
            .arg(format!("STRIX_LLM={}", config.llm_provider))
            .arg("-e")
            .arg(format!("LLM_API_KEY={}", config.llm_api_key))
            .arg("-e")
            .arg("PYTHONPATH=/app");
        
        if let Some(base) = &config.llm_api_base {
            if !base.is_empty() {
                cmd.arg("-e").arg(format!("LLM_API_BASE={}", base));
            }
        }
        
        // 根据模型类型设置特定的 API Key 环境变量
        if config.llm_provider.starts_with("deepseek/") {
            cmd.arg("-e").arg(format!("DEEPSEEK_API_KEY={}", config.llm_api_key));
            if let Some(base) = &config.llm_api_base {
                if !base.is_empty() {
                    cmd.arg("-e").arg(format!("DEEPSEEK_API_BASE={}", base));
                }
            }
        } else if config.llm_provider.starts_with("openai/") {
            cmd.arg("-e").arg(format!("OPENAI_API_KEY={}", config.llm_api_key));
        } else if config.llm_provider.starts_with("anthropic/") {
            cmd.arg("-e").arg(format!("ANTHROPIC_API_KEY={}", config.llm_api_key));
        }
        
        cmd.arg("--cap-add=NET_ADMIN")
            .arg("--cap-add=NET_RAW")
            .arg(&image_name)
            .arg("strix")
            .arg("--target");
    }
    
    // 添加扫描参数
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
    
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
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
    // 尝试从资源目录获取（打包后）
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
    
    // 尝试从当前目录获取（开发环境）
    if let Ok(current_dir) = std::env::current_dir() {
        // 优先：如果当前目录是 src-tauri，直接查找 strix-0.4.0
        if current_dir.ends_with("src-tauri") {
            let strix_main = current_dir
                .join("strix-0.4.0")
                .join("strix")
                .join("interface")
                .join("main.py");
            
            if strix_main.exists() {
                return Ok(strix_main);
            }
        }
        
        // 检查当前目录（可能是项目根目录）
        let strix_main = current_dir
            .join("strix-0.4.0")
            .join("strix")
            .join("interface")
            .join("main.py");
        
        if strix_main.exists() {
            return Ok(strix_main);
        }
        
        // 如果当前目录是 src-tauri，尝试父目录（向后兼容）
        if current_dir.ends_with("src-tauri") {
            if let Some(parent) = current_dir.parent() {
                let strix_main = parent
                    .join("strix-0.4.0")
                    .join("strix")
                    .join("interface")
                    .join("main.py");
                
                if strix_main.exists() {
                    return Ok(strix_main);
                }
            }
        }
        
        // 尝试向上查找项目根目录（查找包含 strix-0.4.0 的目录）
        let mut search_dir = current_dir.clone();
        for _ in 0..5 {
            // 检查当前层级
            let strix_main = search_dir
                .join("strix-0.4.0")
                .join("strix")
                .join("interface")
                .join("main.py");
            
            if strix_main.exists() {
                return Ok(strix_main);
            }
            
            // 检查 src-tauri 子目录
            let strix_main = search_dir
                .join("src-tauri")
                .join("strix-0.4.0")
                .join("strix")
                .join("interface")
                .join("main.py");
            
            if strix_main.exists() {
                return Ok(strix_main);
            }
            
            if let Some(parent) = search_dir.parent() {
                search_dir = parent.to_path_buf();
            } else {
                break;
            }
        }
    }
    
    Err("Strix 主程序未找到，请确保 strix-0.4.0 目录存在于 src-tauri 或项目根目录".to_string())
}

fn get_workspace_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;
    
    Ok(app_dir.join("workspace"))
}

/// 获取打包的 Python 解释器路径
/// 已废弃：使用 Docker 方式，不再需要
#[allow(dead_code)]
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
    if let Ok(current_dir) = std::env::current_dir() {
        // 检查当前目录
        #[cfg(target_os = "windows")]
        let python_exe = current_dir.join("python-runtime").join("python").join("python.exe");
        #[cfg(not(target_os = "windows"))]
        let python_exe = current_dir.join("python-runtime").join("python").join("python3");
        
        if python_exe.exists() {
            return Some(python_exe);
        }
        
        // 如果当前目录是 src-tauri，尝试父目录
        if current_dir.ends_with("src-tauri") {
            if let Some(parent) = current_dir.parent() {
                #[cfg(target_os = "windows")]
                let python_exe = parent.join("python-runtime").join("python").join("python.exe");
                #[cfg(not(target_os = "windows"))]
                let python_exe = parent.join("python-runtime").join("python").join("python3");
                
                if python_exe.exists() {
                    return Some(python_exe);
                }
            }
        }
        
        // 尝试向上查找项目根目录（查找包含 python-runtime 的目录）
        let mut search_dir = current_dir.clone();
        for _ in 0..5 {
            #[cfg(target_os = "windows")]
            let python_exe = search_dir.join("python-runtime").join("python").join("python.exe");
            #[cfg(not(target_os = "windows"))]
            let python_exe = search_dir.join("python-runtime").join("python").join("python3");
            
            if python_exe.exists() {
                return Some(python_exe);
            }
            
            if let Some(parent) = search_dir.parent() {
                search_dir = parent.to_path_buf();
            } else {
                break;
            }
        }
    }
    
    None
}

/// 获取 Python 解释器路径（优先级：打包的 Python > 系统 Python）
/// 已废弃：使用 Docker 方式，不再需要
#[allow(dead_code)]
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
/// 已废弃：使用 Docker 方式，不再需要
#[allow(dead_code)]
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
    
    // 获取工作目录
    let workspace_dir = get_workspace_dir(app)?;
    std::fs::create_dir_all(&workspace_dir)
        .map_err(|e| format!("创建工作目录失败: {}", e))?;
    
    let workspace_dir_clone = workspace_dir.clone();
    
    tokio::spawn(async move {
        // 运行 Python 代码并捕获输出
        match python_embed::run_strix_with_embedded_python(
            strix_path.clone(),
            strix_dir.clone(),
            workspace_dir_clone,
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
                    // 从 stderr 中提取错误信息
                    let error_msg = if !stderr.is_empty() {
                        // 取最后几行错误信息
                        let error_lines: Vec<&str> = stderr.lines().rev().take(5).collect();
                        format!("扫描失败: {}", error_lines.into_iter().rev().collect::<Vec<_>>().join("\n"))
                    } else {
                        "扫描失败".to_string()
                    };
                    let _ = db::update_scan_status(&app_clone, scan_id_clone, "failed", &error_msg).await;
                }
                
                // 解析结果
                let _ = parse_scan_results(&app_clone, scan_id_clone).await;
                
                // 解析结果
                let _ = parse_scan_results(&app_clone, scan_id_clone).await;
            }
            Err(e) => {
                // 记录详细错误信息
                let error_msg = format!("Python 执行错误: {}", e);
                let _ = app_clone.emit("scan-log", serde_json::json!({
                    "scan_id": scan_id_clone,
                    "level": "error",
                    "message": error_msg
                }));
                let _ = db::add_scan_log(&app_clone, scan_id_clone, "error", &error_msg).await;
                let _ = db::update_scan_status(&app_clone, scan_id_clone, "failed", &error_msg).await;
            }
        }
    });
    
    // 立即返回，Python 代码在后台执行
    Ok(())
}

