mod db;
mod strix;
mod python_embed;

use db::init_database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::process::Command;

// 进程管理状态 - 使用 tokio::process::Child
use tokio::process::Child as TokioChild;
type ScanProcesses = Arc<Mutex<HashMap<i64, TokioChild>>>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanConfig {
    pub targets: Vec<String>,
    pub instruction: Option<String>,
    pub run_name: Option<String>,
    pub llm_provider: String,
    pub llm_api_key: String,
    pub llm_api_base: Option<String>,
    pub non_interactive: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanStatus {
    pub id: i64,
    pub run_name: String,
    pub status: String, // "running", "completed", "failed", "stopped"
    pub progress: f64,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: i64,
    pub run_name: String,
    pub targets: Vec<String>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub stats: ScanStats,
    pub results_path: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub description: String,
    pub poc: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanStats {
    pub total_vulnerabilities: i32,
    pub critical: i32,
    pub high: i32,
    pub medium: i32,
    pub low: i32,
}

#[tauri::command]
async fn start_scan(
    app: AppHandle,
    config: ScanConfig,
    processes: State<'_, ScanProcesses>,
) -> Result<i64, String> {
    let scan_id = db::create_scan(&app, &config).map_err(|e| format!("创建扫描失败: {}", e))?;
    
    // 在后台启动扫描
    let app_clone = app.clone();
    let processes_clone = processes.inner().clone();
    tokio::spawn(async move {
        if let Err(e) = strix::run_scan(&app_clone, scan_id, config, processes_clone).await {
            eprintln!("扫描执行失败: {}", e);
            let _ = db::update_scan_status(&app_clone, scan_id, "failed", &e).await;
        }
    });
    
    Ok(scan_id)
}

#[tauri::command]
async fn get_scan_status(app: AppHandle, scan_id: i64) -> Result<ScanStatus, String> {
    db::get_scan_status(&app, scan_id).map_err(|e| format!("获取扫描状态失败: {}", e))
}

#[tauri::command]
async fn get_scan_logs(app: AppHandle, scan_id: i64) -> Result<Vec<db::ScanLog>, String> {
    db::get_scan_logs(&app, scan_id).map_err(|e| format!("获取扫描日志失败: {}", e))
}

#[tauri::command]
async fn get_scan_details(app: AppHandle, scan_id: i64) -> Result<serde_json::Value, String> {
    use serde_json::json;
    use rusqlite::params;
    
    // 获取扫描状态
    let status = db::get_scan_status(&app, scan_id)
        .map_err(|e| format!("获取扫描状态失败: {}", e))?;
    
    // 获取扫描日志
    let logs = db::get_scan_logs(&app, scan_id)
        .map_err(|e| format!("获取扫描日志失败: {}", e))?;
    
    // 获取扫描配置信息以构建命令
    let conn = db::get_connection(&app)
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;
    
    let mut stmt = conn.prepare(
        "SELECT run_name, targets, instruction, llm_provider FROM scans WHERE id = ?1"
    )
    .map_err(|e| format!("准备查询失败: {}", e))?;
    
    let (run_name, targets_json, instruction, _llm_provider) = stmt.query_row(
        params![scan_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        },
    )
    .map_err(|e| format!("查询扫描信息失败: {}", e))?;
    
    let targets: Vec<String> = serde_json::from_str(&targets_json)
        .unwrap_or_default();
    
    // 构建命令字符串
    let mut cmd_parts = vec!["poetry run strix".to_string()];
    for target in &targets {
        cmd_parts.push("--target".to_string());
        cmd_parts.push(format!("'{}'", target.replace('\'', "'\"'\"'")));
    }
    if let Some(inst) = &instruction {
        if !inst.is_empty() {
            cmd_parts.push("--instruction".to_string());
            cmd_parts.push(format!("'{}'", inst.replace('\'', "'\"'\"'")));
        }
    }
    cmd_parts.push("--run-name".to_string());
    cmd_parts.push(format!("'{}'", run_name.replace('\'', "'\"'\"'")));
    
    let command = cmd_parts.join(" ");
    
    // 格式化日志
    let formatted_logs: Vec<serde_json::Value> = logs.iter().map(|log| {
        json!({
            "time": log.timestamp,
            "level": log.level,
            "message": log.message
        })
    }).collect();
    
    Ok(json!({
        "runName": status.run_name,
        "status": status.status,
        "progress": status.progress,
        "message": status.message,
        "createdAt": status.created_at,
        "command": command,
        "logs": formatted_logs
    }))
}

#[tauri::command]
async fn get_scan_result(app: AppHandle, scan_id: i64) -> Result<ScanResult, String> {
    db::get_scan_result(&app, scan_id).map_err(|e| format!("获取扫描结果失败: {}", e))
}

#[tauri::command]
async fn list_scans(app: AppHandle) -> Result<Vec<ScanStatus>, String> {
    db::list_scans(&app).map_err(|e| format!("获取扫描列表失败: {}", e))
}

#[tauri::command]
async fn stop_scan(
    app: AppHandle,
    scan_id: i64,
    processes: State<'_, ScanProcesses>,
) -> Result<(), String> {
    strix::stop_scan(&app, scan_id, processes.inner().clone()).await
}

#[tauri::command]
async fn check_docker() -> Result<serde_json::Value, String> {
    use serde_json::json;
    
    // 检查 Docker 是否安装
    let docker_installed = Command::new("docker")
        .arg("--version")
        .output()
        .await
        .is_ok();
    
    if !docker_installed {
        return Ok(json!({
            "installed": false,
            "image_pulled": false,
            "message": "Docker 未安装或不在 PATH 中"
        }));
    }
    
    // 检查 Docker 是否运行
    let docker_running = Command::new("docker")
        .arg("ps")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    if !docker_running {
        return Ok(json!({
            "installed": true,
            "running": false,
            "image_pulled": false,
            "message": "Docker 已安装但未运行，请启动 Docker Desktop"
        }));
    }
    
    // 检查 Strix 镜像是否存在
    // 可能的镜像名称：strix:0.4.0, strix-agent:0.4.0, usestrix/strix:0.4.0 等
    let image_names = vec![
        "strix:0.4.0",
        "strix-agent:0.4.0",
        "usestrix/strix:0.4.0",
        "strix:latest",
        "ghcr.io/usestrix/strix-sandbox:0.1.10",
        "ghcr.io/usestrix/strix-sandbox:latest",
    ];
    
    let mut image_found = false;
    let mut found_image_name = String::new();
    
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
                    image_found = true;
                    found_image_name = image_name.to_string();
                    break;
                }
            }
        }
    }
    
    // 检查是否有正在运行的 Strix 容器
    let mut running_containers: Vec<serde_json::Value> = Vec::new();
    
    // 方法1: 通过容器名称查找（名称包含 strix-scan-）
    let ps_output = Command::new("docker")
        .arg("ps")
        .arg("--format")
        .arg("{{.Names}}|{{.Image}}|{{.Status}}|{{.ID}}")
        .arg("--filter")
        .arg("name=strix-scan-")
        .output()
        .await;
    
    if let Ok(output) = ps_output {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    let name = parts[0].trim().to_string();
                    let image = parts[1].trim().to_string();
                    let status = parts[2].trim().to_string();
                    let id = parts[3].trim().to_string();
                    
                    running_containers.push(json!({
                        "name": name,
                        "image": image,
                        "status": status,
                        "id": id,
                    }));
                }
            }
        }
    }
    
    // 方法2: 通过标签查找（标签包含 strix-scan-id）
    if running_containers.is_empty() {
        let ps_output = Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("{{.Names}}|{{.Image}}|{{.Status}}|{{.ID}}")
            .arg("--filter")
            .arg("label=strix-scan-id")
            .output()
            .await;
        
        if let Ok(output) = ps_output {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 4 {
                        let name = parts[0].trim().to_string();
                        let image = parts[1].trim().to_string();
                        let status = parts[2].trim().to_string();
                        let id = parts[3].trim().to_string();
                        
                        // 避免重复添加
                        if !running_containers.iter().any(|c| c["name"].as_str() == Some(&name)) {
                            running_containers.push(json!({
                                "name": name,
                                "image": image,
                                "status": status,
                                "id": id,
                            }));
                        }
                    }
                }
            }
        }
    }
    
    // 方法3: 通过镜像名称查找（使用 strix 相关镜像的容器）
    if running_containers.is_empty() && image_found {
        let ps_output = Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("{{.Names}}|{{.Image}}|{{.Status}}|{{.ID}}")
            .arg("--filter")
            .arg(format!("ancestor={}", found_image_name))
            .output()
            .await;
        
        if let Ok(output) = ps_output {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 4 {
                        let name = parts[0].trim().to_string();
                        let image = parts[1].trim().to_string();
                        let status = parts[2].trim().to_string();
                        let id = parts[3].trim().to_string();
                        
                        // 只添加名称包含 strix 的容器
                        if name.contains("strix") || name.contains("Strix") {
                            running_containers.push(json!({
                                "name": name,
                                "image": image,
                                "status": status,
                                "id": id,
                            }));
                        }
                    }
                }
            }
        }
    }
    
    let has_running_container = !running_containers.is_empty();
    
    if image_found {
        let mut message = "Docker 环境就绪".to_string();
        if has_running_container {
            message = format!("Docker 环境就绪，发现 {} 个正在运行的 Strix 容器", running_containers.len());
        }
        
        Ok(json!({
            "installed": true,
            "running": true,
            "image_pulled": true,
            "image_name": found_image_name,
            "has_running_container": has_running_container,
            "running_containers": running_containers,
            "message": message
        }))
    } else {
        let mut message = "Docker 已安装并运行，但未找到 Strix 镜像。请运行: cd strix-0.4.0 && docker build -f containers/Dockerfile -t strix:0.4.0 .".to_string();
        if has_running_container {
            message = format!("发现 {} 个正在运行的 Strix 容器，但未找到 Strix 镜像", running_containers.len());
        }
        
        Ok(json!({
            "installed": true,
            "running": true,
            "image_pulled": false,
            "has_running_container": has_running_container,
            "running_containers": running_containers,
            "message": message
        }))
    }
}

// 已废弃：不再需要检查 Python，因为使用 Docker 方式
// #[tauri::command]
// async fn check_python() -> Result<bool, String> { ... }

// #[tauri::command]
// async fn check_embedded_python() -> Result<bool, String> { ... }

/// 查找可用端口
async fn find_available_port() -> Result<u16, String> {
    use tokio::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("无法绑定端口: {}", e))?;
    let addr = listener.local_addr()
        .map_err(|e| format!("无法获取端口地址: {}", e))?;
    Ok(addr.port())
}

/// 生成随机 token
fn generate_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 获取工作目录（用于容器挂载）
fn get_workspace_dir_for_container(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;
    
    let workspace_dir = app_dir.join("workspace");
    std::fs::create_dir_all(&workspace_dir)
        .map_err(|e| format!("创建工作目录失败: {}", e))?;
    
    Ok(workspace_dir)
}

#[tauri::command]
async fn create_strix_container(
    app: AppHandle,
    container_name: Option<String>,
) -> Result<serde_json::Value, String> {
    use serde_json::json;
    use std::time::Duration;
    
    // 检查 Docker 是否运行
    let docker_running = Command::new("docker")
        .arg("ps")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    if !docker_running {
        return Err("Docker 未运行，请先启动 Docker Desktop".to_string());
    }
    
    // 查找 Strix 镜像
    let image_names = vec![
        "strix:0.4.0",
        "strix-agent:0.4.0",
        "usestrix/strix:0.4.0",
        "strix:latest",
        "ghcr.io/usestrix/strix-sandbox:0.1.10",
        "ghcr.io/usestrix/strix-sandbox:latest",
    ];
    
    let mut found_image_name = None;
    
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
                    found_image_name = Some(image_name.to_string());
                    break;
                }
            }
        }
    }
    
    let image_name = found_image_name.ok_or_else(|| {
        "未找到 Strix Docker 镜像。请先构建镜像：cd strix-0.4.0 && docker build -f containers/Dockerfile -t strix:0.4.0 .".to_string()
    })?;
    
    // 生成容器名称
    let name = container_name.unwrap_or_else(|| {
        format!("strix-scan-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"))
    });
    
    // 检查容器是否已存在
    let check_output = Command::new("docker")
        .arg("ps")
        .arg("-a")
        .arg("--filter")
        .arg(format!("name={}", name))
        .arg("--format")
        .arg("{{.Names}}")
        .output()
        .await;
    
    if let Ok(output) = check_output {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if stdout.lines().any(|line| line.trim() == name) {
                // 容器已存在，尝试启动
                let start_output = Command::new("docker")
                    .arg("start")
                    .arg(&name)
                    .output()
                    .await;
                
                if let Ok(output) = start_output {
                    if output.status.success() {
                        // 获取容器信息
                        let inspect_output = Command::new("docker")
                            .arg("inspect")
                            .arg(&name)
                            .arg("--format")
                            .arg("{{.State.Status}}|{{.Id}}|{{.Config.Image}}")
                            .output()
                            .await;
                        
                        if let Ok(output) = inspect_output {
                            if let Ok(stdout) = String::from_utf8(output.stdout) {
                                let parts: Vec<&str> = stdout.trim().split('|').collect();
                                if parts.len() >= 3 {
                                    return Ok(json!({
                                        "success": true,
                                        "message": format!("容器 {} 已存在并已启动", name),
                                        "container_name": name,
                                        "container_id": parts[1].trim(),
                                        "image": parts[2].trim(),
                                        "status": parts[0].trim(),
                                    }));
                                }
                            }
                        }
                        
                        return Ok(json!({
                            "success": true,
                            "message": format!("容器 {} 已启动", name),
                            "container_name": name,
                        }));
                    }
                }
                
                return Err(format!("容器 {} 已存在但无法启动", name));
            }
        }
    }
    
    // 查找可用端口
    let caido_port = find_available_port().await?;
    let tool_server_port = find_available_port().await?;
    let tool_server_token = generate_token();
    
    // 获取工作目录
    let workspace_dir = get_workspace_dir_for_container(&app)?;
    let workspace_dir_str = workspace_dir.to_string_lossy().replace('\\', "/");
    
    // 构建 Docker run 命令
    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("-d")  // 后台运行
        .arg("--name")
        .arg(&name)
        .arg("--hostname")
        .arg(format!("strix-scan-{}", name.replace("strix-scan-", "")))
        .arg("-v")
        .arg(format!("{}:/workspace", workspace_dir_str))
        .arg("-p")
        .arg(format!("{}:{}", caido_port, caido_port))
        .arg("-p")
        .arg(format!("{}:{}", tool_server_port, tool_server_port))
        .arg("--cap-add=NET_ADMIN")
        .arg("--cap-add=NET_RAW")
        .arg("--label")
        .arg(format!("strix-scan-id={}", name.replace("strix-scan-", "")))
        .arg("-e")
        .arg("PYTHONUNBUFFERED=1")
        .arg("-e")
        .arg(format!("CAIDO_PORT={}", caido_port))
        .arg("-e")
        .arg(format!("TOOL_SERVER_PORT={}", tool_server_port))
        .arg("-e")
        .arg(format!("TOOL_SERVER_TOKEN={}", tool_server_token))
        .arg("--tty")
        .arg(&image_name)
        .arg("sleep")
        .arg("infinity");
    
    // 执行命令
    let output = cmd.output().await
        .map_err(|e| format!("创建容器失败: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("创建容器失败: {}", error_msg));
    }
    
    let container_id = String::from_utf8(output.stdout)
        .unwrap_or_default()
        .trim()
        .to_string();
    
    // 等待容器启动
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 检查容器状态
    let status_output = Command::new("docker")
        .arg("inspect")
        .arg(&name)
        .arg("--format")
        .arg("{{.State.Status}}")
        .output()
        .await;
    
    let status = if let Ok(output) = status_output {
        String::from_utf8(output.stdout).unwrap_or_default().trim().to_string()
    } else {
        "unknown".to_string()
    };
    
    Ok(json!({
        "success": true,
        "message": format!("容器 {} 创建成功", name),
        "container_name": name,
        "container_id": container_id,
        "image": image_name,
        "status": status,
        "caido_port": caido_port,
        "tool_server_port": tool_server_port,
        "workspace": workspace_dir_str,
    }))
}

#[tauri::command]
fn save_setting(app: AppHandle, key: String, value: String) -> Result<(), String> {
    db::save_setting(&app, &key, &value)
        .map_err(|e| format!("保存设置失败: {}", e))
}

#[tauri::command]
fn get_setting(app: AppHandle, key: String) -> Result<Option<String>, String> {
    db::get_setting(&app, &key)
        .map_err(|e| format!("获取设置失败: {}", e))
}

#[tauri::command]
fn get_all_settings(app: AppHandle) -> Result<std::collections::HashMap<String, String>, String> {
    db::get_all_settings(&app)
        .map_err(|e| format!("获取所有设置失败: {}", e))
}

#[tauri::command]
async fn test_llm_connection(
    _app: AppHandle,
    llm_provider: String,
    llm_api_key: String,
    llm_api_base: Option<String>,
) -> Result<String, String> {
    use tokio::process::Command;
    
    // 查找 Strix 镜像
    let image_names = vec![
        "strix:0.4.0",
        "strix-agent:0.4.0",
        "usestrix/strix:0.4.0",
        "strix:latest",
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
    
    // 构建测试脚本
    let api_base_str = llm_api_base.as_ref()
        .map(|s| format!("r'{}'", s.replace('\\', r"\\").replace('\'', r"\'")))
        .unwrap_or_else(|| "None".to_string());
    
    let test_script = format!(
        r#"
import os
import sys
import traceback

try:
    import litellm
    
    # 设置环境变量
    os.environ["STRIX_LLM"] = r"{}"
    api_key = r"{}"
    os.environ["LLM_API_KEY"] = api_key
    
    # 根据模型类型设置特定的 API Key 环境变量
    model_name = r"{}"
    if model_name.startswith("deepseek/"):
        os.environ["DEEPSEEK_API_KEY"] = api_key
    elif model_name.startswith("openai/"):
        os.environ["OPENAI_API_KEY"] = api_key
    elif model_name.startswith("anthropic/"):
        os.environ["ANTHROPIC_API_KEY"] = api_key
    elif model_name.startswith("ollama/"):
        os.environ["OLLAMA_API_KEY"] = api_key
    
    if {}:
        api_base = {}
        os.environ["LLM_API_BASE"] = api_base
        # 对于 Deepseek，也设置 DEEPSEEK_API_BASE
        if model_name.startswith("deepseek/"):
            os.environ["DEEPSEEK_API_BASE"] = api_base
    else:
        api_base = None
    
    # 获取配置（从环境变量读取，确保一致性）
    model_name = os.getenv("STRIX_LLM", "openai/gpt-5")
    api_key = os.getenv("LLM_API_KEY")
    api_base = (
        os.getenv("LLM_API_BASE")
        or os.getenv("OPENAI_API_BASE")
        or os.getenv("DEEPSEEK_API_BASE")
        or os.getenv("LITELLM_BASE_URL")
        or os.getenv("OLLAMA_API_BASE")
    )
    
    if not api_key:
        raise ValueError("LLM_API_KEY 未设置")
    
    # 配置 litellm - 使用环境变量方式，让 litellm 自动识别
    # 对于 Deepseek，litellm 会自动从 DEEPSEEK_API_KEY 读取
    # 但也可以显式设置
    if model_name.startswith("deepseek/"):
        # Deepseek 特殊处理：确保使用正确的 API key
        if not os.getenv("DEEPSEEK_API_KEY"):
            os.environ["DEEPSEEK_API_KEY"] = api_key
    else:
        # 其他模型使用通用方式
        litellm.api_key = api_key
    
    if api_base:
        litellm.api_base = api_base
    
    # 测试消息
    test_messages = [
        {{"role": "system", "content": "You are a helpful assistant."}},
        {{"role": "user", "content": "Reply with just 'OK'."}},
    ]
    
    llm_timeout = int(os.getenv("LLM_TIMEOUT", "60"))
    
    print(f"[TEST] 正在测试 LLM 连接...")
    print(f"[TEST] 模型: {{model_name}}")
    print(f"[TEST] API Base: {{api_base if api_base else '默认'}}")
    
    # 执行测试请求
    response = litellm.completion(
        model=model_name,
        messages=test_messages,
        timeout=llm_timeout,
    )
    
    # 验证响应（简化版本，不依赖 strix.interface.utils）
    if not response or not hasattr(response, 'choices') or len(response.choices) == 0:
        raise ValueError("LLM 响应无效：没有返回 choices")
    
    if not hasattr(response.choices[0], 'message') or not hasattr(response.choices[0].message, 'content'):
        raise ValueError("LLM 响应无效：choices[0].message.content 不存在")
    
    # 获取响应内容
    response_content = response.choices[0].message.content
    if not response_content or not response_content.strip():
        raise ValueError("LLM 响应无效：响应内容为空")
    
    print(f"[TEST] ✅ LLM 连接成功！")
    print(f"[TEST] 响应: {{response_content}}")
    
    result_msg = f"✅ LLM 连接成功！\\n\\n模型: {{model_name}}\\nAPI Base: {{api_base if api_base else '默认'}}\\n响应: {{response_content}}"
    print(result_msg)
    
except ImportError as e:
    error_msg = f"导入模块失败: {{e}}"
    print(f"[TEST] ❌ {{error_msg}}", file=sys.stderr)
    sys.exit(1)
except Exception as e:
    error_type = type(e).__name__
    error_msg = str(e)
    error_trace = traceback.format_exc()
    print(f"[TEST] ❌ LLM 连接失败: {{error_type}}: {{error_msg}}", file=sys.stderr)
    print(error_trace, file=sys.stderr)
    sys.exit(1)
"#,
        llm_provider.replace('\\', r"\\").replace('\'', r"\'"),
        llm_api_key.replace('\\', r"\\").replace('\'', r"\'"),
        llm_provider.replace('\\', r"\\").replace('\'', r"\'"),  // 模型名称（用于判断类型）
        if llm_api_base.is_some() { "True" } else { "False" },
        api_base_str
    );
    
    // 执行 Docker 命令
    let mut cmd = Command::new("docker");
    
    if let Some(container_name) = running_container_name {
        // 使用运行中的容器执行测试
        // 直接使用虚拟环境中的 Python，避免 Poetry 路径问题
        cmd.arg("exec")
            .arg("-w")
            .arg("/app")  // 设置工作目录为 /app，确保能找到依赖
            .arg("-e")
            .arg(format!("STRIX_LLM={}", llm_provider))
            .arg("-e")
            .arg(format!("LLM_API_KEY={}", llm_api_key))
            .arg("-e")
            .arg("PYTHONPATH=/app");
        
        if let Some(base) = &llm_api_base {
            if !base.is_empty() {
                cmd.arg("-e").arg(format!("LLM_API_BASE={}", base));
            }
        }
        
        cmd.arg(&container_name)
            .arg("/app/venv/bin/python")  // 直接使用虚拟环境中的 Python
            .arg("-c")
            .arg(&test_script);
    } else {
        // 创建临时容器执行测试
        cmd.arg("run")
            .arg("--rm")
            .arg("-w")
            .arg("/app")
            .arg("-e")
            .arg(format!("STRIX_LLM={}", llm_provider))
            .arg("-e")
            .arg(format!("LLM_API_KEY={}", llm_api_key))
            .arg("-e")
            .arg("PYTHONPATH=/app");
        
        if let Some(base) = &llm_api_base {
            if !base.is_empty() {
                cmd.arg("-e").arg(format!("LLM_API_BASE={}", base));
            }
        }
        
        cmd.arg("--entrypoint")
            .arg("poetry")
            .arg(&image_name)
            .arg("run")
            .arg("python")
            .arg("-c")
            .arg(&test_script);
    }
    
    let output = cmd.output()
        .await
        .map_err(|e| format!("执行 Docker 失败: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if output.status.success() {
        if stdout.trim().is_empty() {
            Ok("✅ LLM 连接测试成功，但未收到输出".to_string())
        } else {
            Ok(stdout.trim().to_string())
        }
    } else {
        let error_msg = if stderr.trim().is_empty() {
            format!("LLM 连接测试失败: {}", stdout.trim())
        } else {
            format!("{}\n{}", stderr.trim(), stdout.trim())
        };
        Err(error_msg)
    }
}

#[tauri::command]
fn delete_scan(app: AppHandle, scan_id: i64) -> Result<(), String> {
    db::delete_scan(&app, scan_id)
        .map_err(|e| format!("删除扫描失败: {}", e))
}

#[tauri::command]
fn clear_all_scans(app: AppHandle) -> Result<(), String> {
    db::clear_all_scans(&app)
        .map_err(|e| format!("清空扫描历史失败: {}", e))
}

#[tauri::command]
async fn build_strix_image(app: AppHandle) -> Result<(), String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    
    // 查找 strix-0.4.0 目录
    let strix_dir = if let Ok(current_dir) = std::env::current_dir() {
        // 优先检查 src-tauri/strix-0.4.0
        let strix_path = current_dir.join("src-tauri").join("strix-0.4.0");
        if strix_path.exists() {
            strix_path
        } else {
            // 检查当前目录
            let strix_path = current_dir.join("strix-0.4.0");
            if strix_path.exists() {
                strix_path
            } else {
                return Err("未找到 strix-0.4.0 目录。请确保 strix-0.4.0 目录存在于 src-tauri 或项目根目录".to_string());
            }
        }
    } else {
        return Err("无法获取当前工作目录".to_string());
    };
    
    let dockerfile_path = strix_dir.join("containers").join("Dockerfile");
    if !dockerfile_path.exists() {
        return Err(format!("未找到 Dockerfile: {}", dockerfile_path.display()));
    }
    
    // 发送初始消息
    let _ = app.emit("docker-build-log", serde_json::json!({
        "level": "info",
        "message": "开始构建 Docker 镜像..."
    }));
    
    // 构建 Docker 命令
    // 禁用 BuildKit 以确保输出格式一致，使用 --progress=plain 显示实时输出
    // 使用镜像加速源 docker.1ms.run
    let mut cmd = Command::new("docker");
    cmd.env("DOCKER_BUILDKIT", "0")  // 禁用 BuildKit 以获得传统输出格式
        .env("BUILDKIT_PROGRESS", "plain")  // 如果 BuildKit 启用，使用 plain 格式
        .arg("build")
        .arg("--progress=plain")
        .arg("--build-arg")
        .arg("DOCKER_REGISTRY_MIRROR=docker.1ms.run")  // 使用镜像加速源
        .arg("-f")
        .arg(dockerfile_path.to_string_lossy().as_ref())
        .arg("-t")
        .arg("strix:0.4.0")
        .arg(".")
        .current_dir(&strix_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    // 发送调试信息
    let _ = app.emit("docker-build-log", serde_json::json!({
        "level": "info",
        "message": format!("执行命令: docker build --progress=plain --build-arg DOCKER_REGISTRY_MIRROR=docker.1ms.run -f {} -t strix:0.4.0 .", dockerfile_path.display())
    }));
    let _ = app.emit("docker-build-log", serde_json::json!({
        "level": "info",
        "message": "使用镜像加速源: docker.1ms.run"
    }));
    
    // 启动进程
    let mut child = cmd.spawn().map_err(|e| format!("启动 Docker build 失败: {}", e))?;
    
    // 获取 stdout 和 stderr
    let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
    let stderr = child.stderr.take().ok_or("无法获取 stderr")?;
    
    let app_clone = app.clone();
    
    // 读取 stdout 并发送事件（使用较小的缓冲区以确保实时性）
    // Docker build 的输出通常混合在 stdout 和 stderr 中
    tokio::spawn(async move {
        let mut reader = BufReader::with_capacity(512, stdout);  // 更小的缓冲区
        let mut line = String::new();
        
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF，发送结束消息
                    let _ = app_clone.emit("docker-build-log", serde_json::json!({
                        "level": "info",
                        "message": "[stdout] 输出流已结束"
                    }));
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // 立即发送，不等待
                        let _ = app_clone.emit("docker-build-log", serde_json::json!({
                            "level": "info",
                            "message": trimmed
                        }));
                    }
                }
                Err(e) => {
                    let _ = app_clone.emit("docker-build-log", serde_json::json!({
                        "level": "error",
                        "message": format!("读取输出错误: {}", e)
                    }));
                    break;
                }
            }
        }
    });
    
    // 读取 stderr 并发送事件（Docker build 的进度信息通常输出到 stderr）
    let app_clone2 = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::with_capacity(512, stderr);  // 更小的缓冲区
        let mut line = String::new();
        
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF，发送结束消息
                    let _ = app_clone2.emit("docker-build-log", serde_json::json!({
                        "level": "info",
                        "message": "[stderr] 输出流已结束"
                    }));
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // 立即发送，不等待
                        let _ = app_clone2.emit("docker-build-log", serde_json::json!({
                            "level": "info",
                            "message": trimmed
                        }));
                    }
                }
                Err(e) => {
                    let _ = app_clone2.emit("docker-build-log", serde_json::json!({
                        "level": "error",
                        "message": format!("读取错误输出错误: {}", e)
                    }));
                    break;
                }
            }
        }
    });
    
    // 等待进程完成
    let status = child.wait().await.map_err(|e| format!("等待 Docker build 失败: {}", e))?;
    
    if status.success() {
        let _ = app.emit("docker-build-log", serde_json::json!({
            "level": "success",
            "message": "✅ Docker 镜像构建成功！"
        }));
        Ok(())
    } else {
        let _ = app.emit("docker-build-log", serde_json::json!({
            "level": "error",
            "message": "❌ Docker 镜像构建失败"
        }));
        Err("Docker 镜像构建失败".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            start_scan,
            get_scan_status,
            get_scan_result,
            get_scan_details,
            list_scans,
            stop_scan,
            get_scan_logs,
            delete_scan,
            clear_all_scans,
            check_docker,
            build_strix_image,
            create_strix_container,
            save_setting,
            get_setting,
            get_all_settings,
            test_llm_connection
        ])
        .manage(ScanProcesses::default())
        .setup(|app| {
            // 初始化数据库
            if let Err(e) = init_database(app.handle()) {
                eprintln!("数据库初始化失败: {}", e);
            }
            
            // 不再需要初始化嵌入的 Python，因为使用 Docker 方式
            // if let Err(e) = python_embed::init_python() { ... }
            
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // 清理资源
            }
        });
}

