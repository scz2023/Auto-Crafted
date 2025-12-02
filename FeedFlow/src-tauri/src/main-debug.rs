// Debug version with console window enabled
// Copy this to main.rs temporarily to see console output

mod db;

use db::get_connection;
use serde::{Deserialize, Serialize};
use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Serialize, Deserialize)]
struct DbQuery {
    sql: String,
    params: Vec<serde_json::Value>,
}

#[tauri::command]
async fn db_query(app: AppHandle, query: DbQuery) -> Result<serde_json::Value, String> {
    let conn = get_connection(&app).map_err(|e| format!("数据库连接失败: {}", e))?;
    
    let sql_upper = query.sql.trim().to_uppercase();
    
    if sql_upper.starts_with("SELECT") {
        let mut stmt = conn
            .prepare(&query.sql)
            .map_err(|e| format!("SQL 准备失败: {}", e))?;
        
        // 将 JSON 值转换为 rusqlite 参数
        let params: Vec<rusqlite::types::Value> = query.params.iter()
            .map(|v| {
                match v {
                    serde_json::Value::Null => rusqlite::types::Value::Null,
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            rusqlite::types::Value::Integer(i)
                        } else if let Some(f) = n.as_f64() {
                            rusqlite::types::Value::Real(f)
                        } else {
                            rusqlite::types::Value::Text(v.to_string())
                        }
                    }
                    serde_json::Value::String(s) => rusqlite::types::Value::Text(s.clone()),
                    serde_json::Value::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
                    _ => rusqlite::types::Value::Text(v.to_string()),
                }
            })
            .collect();
        
        let rows = stmt
            .query_map(
                rusqlite::params_from_iter(params.iter()),
                |row| {
                    let mut map = serde_json::Map::new();
                    for (i, name) in row.as_ref().column_names().iter().enumerate() {
                        let value: rusqlite::types::Value = row.get(i).unwrap_or(rusqlite::types::Value::Null);
                        let json_value = match value {
                            rusqlite::types::Value::Null => serde_json::Value::Null,
                            rusqlite::types::Value::Integer(i) => serde_json::Value::Number(i.into()),
                            rusqlite::types::Value::Real(f) => {
                                serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap())
                            }
                            rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                            rusqlite::types::Value::Blob(_) => serde_json::Value::String("BLOB".to_string()),
                        };
                        map.insert(name.to_string(), json_value);
                    }
                    Ok(serde_json::Value::Object(map))
                },
            )
            .map_err(|e| format!("查询执行失败: {}", e))?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("行解析失败: {}", e))?);
        }
        
        Ok(serde_json::json!({ "success": true, "data": results }))
    } else {
        let mut stmt = conn
            .prepare(&query.sql)
            .map_err(|e| format!("SQL 准备失败: {}", e))?;
        
        // 将 JSON 值转换为 rusqlite 参数
        let params: Vec<rusqlite::types::Value> = query.params.iter()
            .map(|v| {
                match v {
                    serde_json::Value::Null => rusqlite::types::Value::Null,
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            rusqlite::types::Value::Integer(i)
                        } else if let Some(f) = n.as_f64() {
                            rusqlite::types::Value::Real(f)
                        } else {
                            rusqlite::types::Value::Text(v.to_string())
                        }
                    }
                    serde_json::Value::String(s) => rusqlite::types::Value::Text(s.clone()),
                    serde_json::Value::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
                    _ => rusqlite::types::Value::Text(v.to_string()),
                }
            })
            .collect();
        
        let result = stmt
            .execute(rusqlite::params_from_iter(params.iter()))
            .map_err(|e| format!("执行失败: {}", e))?;
        
        Ok(serde_json::json!({
            "success": true,
            "data": {
                "lastInsertRowid": conn.last_insert_rowid(),
                "changes": result
            }
        }))
    }
}

// 服务器进程状态
type ServerProcess = Arc<Mutex<Option<Child>>>;

fn start_nitro_server(app: &AppHandle) -> Result<Child, String> {
    // 获取可执行文件所在目录
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    let exe_dir = exe_path.parent()
        .ok_or("Failed to get executable directory")?;
    
    println!("Executable directory: {:?}", exe_dir);
    
    // 尝试获取资源目录（Tauri 打包的资源文件）
    let resource_dir = app.path().resource_dir().ok();
    if let Some(ref res_dir) = resource_dir {
        println!("Resource directory: {:?}", res_dir);
    }
    
    // 尝试多个可能的路径
    let mut possible_paths = vec![
        // 便携版：服务器在 .output/server 目录（与 exe 同级）
        exe_dir.join(".output").join("server").join("index.mjs"),
        // 便携版：服务器在 exe 所在目录的父目录
        exe_dir.parent().map(|p| p.join(".output").join("server").join("index.mjs")),
    ];
    
    // 如果资源目录存在，也检查那里
    if let Some(ref res_dir) = resource_dir {
        possible_paths.push(Some(res_dir.join(".output").join("server").join("index.mjs")));
        possible_paths.push(Some(res_dir.join("server").join("index.mjs")));
    }
    
    // 开发环境：在项目根目录
    possible_paths.push(
        exe_dir.parent().map(|p| p.parent().map(|pp| pp.join(".output").join("server").join("index.mjs"))).flatten()
    );
    
    let mut server_dir = None;
    let mut server_index = None;
    let mut tried_paths = Vec::new();
    
    for path_opt in possible_paths {
        if let Some(path) = path_opt {
            tried_paths.push(path.clone());
            if path.exists() {
                println!("Found server at: {:?}", path);
                server_index = Some(path.clone());
                server_dir = path.parent().map(|p| p.to_path_buf());
                break;
            }
        }
    }
    
    let (server_dir, server_index) = match (server_dir, server_index) {
        (Some(dir), Some(idx)) => (dir, idx),
        _ => {
            let error_msg = format!(
                "Server file index.mjs not found. Tried paths:\n{}",
                tried_paths.iter()
                    .map(|p| format!("  - {:?}", p))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            eprintln!("{}", error_msg);
            return Err(error_msg);
        }
    };
    
    start_server_at(&server_dir, &server_index)
}

fn start_server_at(server_dir: &std::path::Path, server_index: &std::path::Path) -> Result<Child, String> {
    // 查找 Node.js
    let node_cmd = if cfg!(target_os = "windows") {
        "node.exe"
    } else {
        "node"
    };
    
    // 尝试直接使用 node 命令（假设在 PATH 中）
    let mut cmd = Command::new(node_cmd);
    cmd.arg(server_index)
       .current_dir(server_dir)
       .stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped());
    
    // 设置环境变量
    cmd.env("NODE_ENV", "production");
    cmd.env("PORT", "3000");
    cmd.env("HOST", "127.0.0.1");
    
    println!("Starting Nitro server: {:?}", server_index);
    println!("Working directory: {:?}", server_dir);
    println!("Node command: {}", node_cmd);
    
    let child = cmd.spawn()
        .map_err(|e| format!("Failed to start server: {}. Please ensure Node.js is installed and in PATH", e))?;
    
    // 等待服务器启动，最多等待 10 秒
    let max_attempts = 20;
    let mut attempts = 0;
    let client = reqwest::blocking::Client::new();
    
    while attempts < max_attempts {
        std::thread::sleep(std::time::Duration::from_millis(500));
        attempts += 1;
        
        // 尝试连接服务器
        match client.get("http://127.0.0.1:3000").send() {
            Ok(_) => {
                println!("Nitro server started and responding (attempt {}/{})", attempts, max_attempts);
                return Ok(child);
            }
            Err(e) => {
                // 服务器还未启动，继续等待
                if attempts % 4 == 0 {
                    println!("Waiting for server to start... ({}/{}) - Error: {}", attempts, max_attempts, e);
                }
            }
        }
    }
    
    // 如果超时，返回错误但保留进程（可能服务器正在启动）
    eprintln!("Warning: Server startup timeout, but process is still running");
    eprintln!("Server may still be starting. The application will continue, but API calls may fail.");
    Ok(child)
}

fn main() {
    // 在 Windows 上分配控制台窗口（用于调试）
    #[cfg(windows)]
    {
        use std::io::Write;
        use winapi::um::consoleapi::AllocConsole;
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_OUTPUT_HANDLE;
        use winapi::um::wincon::SetConsoleTitleA;
        
        unsafe {
            AllocConsole();
            let stdout = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut handle = std::io::BufWriter::new(std::fs::File::from_raw_handle(std::io::RawHandle::from_raw_handle(stdout as *mut _)));
            std::io::stdout().flush().unwrap();
        }
        
        unsafe {
            SetConsoleTitleA(b"FeedFlow Debug Console\0".as_ptr() as *const i8);
        }
    }
    
    let server_process: ServerProcess = Arc::new(Mutex::new(None));
    let server_process_clone = server_process.clone();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_path::init())
        .invoke_handler(tauri::generate_handler![db_query])
        .setup(move |app| {
            // 应用启动时启动服务器
            println!("Attempting to start Nitro server...");
            match start_nitro_server(app.handle()) {
                Ok(child) => {
                    // 保存服务器进程
                    let mut process = server_process.lock().unwrap();
                    *process = Some(child);
                    println!("Nitro server process started successfully");
                }
                Err(e) => {
                    eprintln!("ERROR: Failed to start Nitro server: {}", e);
                    eprintln!("The application will continue, but server-side API may not be available.");
                    eprintln!("Please ensure:");
                    eprintln!("  1. Node.js is installed and in PATH");
                    eprintln!("  2. .output/server directory exists in the portable version");
                    eprintln!("  3. Check the console for more details");
                }
            }
            
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |_app_handle, event| {
            // 应用退出时停止服务器
            if let tauri::RunEvent::Exit = event {
                if let Ok(mut process) = server_process_clone.lock() {
                    if let Some(mut child) = process.take() {
                        let _ = child.kill();
                        println!("Nitro server stopped");
                    }
                }
            }
        });
}

