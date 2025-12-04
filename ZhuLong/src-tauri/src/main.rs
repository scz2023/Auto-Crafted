mod db;
mod strix;
mod python_embed;

use db::init_database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};
use tokio::process::Command;

// 进程管理状态 - 使用 tokio::process::Child
use tokio::process::Child as TokioChild;
type ScanProcesses = Arc<Mutex<HashMap<i64, TokioChild>>>;

#[derive(Debug, Serialize, Deserialize)]
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
async fn check_docker() -> Result<bool, String> {
    let output = Command::new("docker")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Docker 未安装或不在 PATH 中: {}", e))?;
    
    if !output.status.success() {
        return Err("Docker 命令执行失败".to_string());
    }
    
    Ok(true)
}

#[tauri::command]
async fn check_python() -> Result<bool, String> {
    let output = Command::new("python")
        .arg("--version")
        .output()
        .await;
    
    let output = match output {
        Ok(o) => o,
        Err(_) => {
            Command::new("python3")
                .arg("--version")
                .output()
                .await
                .map_err(|e| format!("系统 Python 未安装或不在 PATH 中: {}", e))?
        }
    };
    
    Ok(output.status.success())
}

#[tauri::command]
async fn check_embedded_python() -> Result<bool, String> {
    match python_embed::init_python() {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("内嵌 Python 初始化失败: {}", e))
    }
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            start_scan,
            get_scan_status,
            get_scan_result,
            list_scans,
            stop_scan,
            get_scan_logs,
            check_docker,
            check_python,
            check_embedded_python,
            save_setting,
            get_setting,
            get_all_settings
        ])
        .manage(ScanProcesses::default())
        .setup(|app| {
            // 初始化数据库
            if let Err(e) = init_database(app.handle()) {
                eprintln!("数据库初始化失败: {}", e);
            }
            
            // 初始化嵌入的 Python 解释器
            if let Err(e) = python_embed::init_python() {
                eprintln!("初始化嵌入 Python 失败: {}", e);
                eprintln!("将回退到使用系统 Python");
            } else {
                println!("嵌入 Python 解释器已初始化");
            }
            
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

