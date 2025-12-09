use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ScanRecord {
    pub id: i64,
    pub run_name: String,
    pub targets: String, // JSON array
    pub status: String,
    pub progress: f64,
    pub message: String,
    pub results_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub fn init_database(app: &AppHandle) -> SqlResult<()> {
    let app_dir = app.path().app_data_dir().unwrap();
    std::fs::create_dir_all(&app_dir).unwrap();
    let db_path = app_dir.join("zhulong.db");
    
    let conn = Connection::open(&db_path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_name TEXT NOT NULL,
            targets TEXT NOT NULL,
            instruction TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            progress REAL NOT NULL DEFAULT 0.0,
            message TEXT NOT NULL DEFAULT '',
            results_path TEXT,
            llm_provider TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS vulnerabilities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_id INTEGER NOT NULL,
            vuln_id TEXT NOT NULL,
            title TEXT NOT NULL,
            severity TEXT NOT NULL,
            description TEXT NOT NULL,
            poc TEXT,
            remediation TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_scan_id ON vulnerabilities(scan_id)",
        [],
    )?;
    
    // 创建扫描日志表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scan_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_id INTEGER NOT NULL,
            level TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_log_scan_id ON scan_logs(scan_id)",
        [],
    )?;
    
    // 创建设置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    
    Ok(())
}

pub fn get_connection(app: &AppHandle) -> SqlResult<Connection> {
    let app_dir = app.path().app_data_dir().unwrap();
    let db_path = app_dir.join("zhulong.db");
    Connection::open(&db_path)
}

pub fn create_scan(app: &AppHandle, config: &crate::ScanConfig) -> SqlResult<i64> {
    let conn = get_connection(app)?;
    let run_name = config.run_name.clone().unwrap_or_else(|| {
        format!("scan_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"))
    });
    
    let targets_json = serde_json::to_string(&config.targets).unwrap();
    
    conn.execute(
        "INSERT INTO scans (run_name, targets, instruction, status, llm_provider) 
         VALUES (?1, ?2, ?3, 'running', ?4)",
        params![
            run_name,
            targets_json,
            config.instruction,
            config.llm_provider
        ],
    )?;
    
    Ok(conn.last_insert_rowid())
}

pub fn get_scan_status(app: &AppHandle, scan_id: i64) -> SqlResult<crate::ScanStatus> {
    let conn = get_connection(app)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, run_name, status, progress, message, created_at 
         FROM scans WHERE id = ?1"
    )?;
    
    let row = stmt.query_row(params![scan_id], |row| {
        Ok(crate::ScanStatus {
            id: row.get(0)?,
            run_name: row.get(1)?,
            status: row.get(2)?,
            progress: row.get(3)?,
            message: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    
    Ok(row)
}

pub async fn update_scan_status(
    app: &AppHandle,
    scan_id: i64,
    status: &str,
    message: &str,
) -> SqlResult<()> {
    let conn = get_connection(app)?;
    
    conn.execute(
        "UPDATE scans SET status = ?1, message = ?2, updated_at = datetime('now') 
         WHERE id = ?3",
        params![status, message, scan_id],
    )?;
    
    Ok(())
}

pub fn list_scans(app: &AppHandle) -> SqlResult<Vec<crate::ScanStatus>> {
    let conn = get_connection(app)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, run_name, status, progress, message, created_at 
         FROM scans ORDER BY created_at DESC LIMIT 100"
    )?;
    
    let rows = stmt.query_map([], |row| {
        Ok(crate::ScanStatus {
            id: row.get(0)?,
            run_name: row.get(1)?,
            status: row.get(2)?,
            progress: row.get(3)?,
            message: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    
    let mut scans = Vec::new();
    for row in rows {
        scans.push(row?);
    }
    
    Ok(scans)
}

pub fn get_scan_result(app: &AppHandle, scan_id: i64) -> SqlResult<crate::ScanResult> {
    let conn = get_connection(app)?;
    
    // 获取扫描基本信息
    let mut stmt = conn.prepare(
        "SELECT run_name, targets, results_path, created_at 
         FROM scans WHERE id = ?1"
    )?;
    
    let (run_name, targets_json, results_path, created_at) = stmt.query_row(
        params![scan_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    )?;
    
    let targets: Vec<String> = serde_json::from_str(&targets_json).unwrap_or_default();
    
    // 获取漏洞列表
    let mut vuln_stmt = conn.prepare(
        "SELECT vuln_id, title, severity, description, poc, remediation 
         FROM vulnerabilities WHERE scan_id = ?1 ORDER BY 
         CASE severity 
             WHEN 'critical' THEN 1 
             WHEN 'high' THEN 2 
             WHEN 'medium' THEN 3 
             WHEN 'low' THEN 4 
             ELSE 5 
         END"
    )?;
    
    let vuln_rows = vuln_stmt.query_map(params![scan_id], |row| {
        Ok(crate::Vulnerability {
            id: row.get(0)?,
            title: row.get(1)?,
            severity: row.get(2)?,
            description: row.get(3)?,
            poc: row.get(4)?,
            remediation: row.get(5)?,
        })
    })?;
    
    let mut vulnerabilities = Vec::new();
    for row in vuln_rows {
        vulnerabilities.push(row?);
    }
    
    // 计算统计信息
    let stats = crate::ScanStats {
        total_vulnerabilities: vulnerabilities.len() as i32,
        critical: vulnerabilities.iter().filter(|v| v.severity == "critical").count() as i32,
        high: vulnerabilities.iter().filter(|v| v.severity == "high").count() as i32,
        medium: vulnerabilities.iter().filter(|v| v.severity == "medium").count() as i32,
        low: vulnerabilities.iter().filter(|v| v.severity == "low").count() as i32,
    };
    
    Ok(crate::ScanResult {
        id: scan_id,
        run_name,
        targets,
        vulnerabilities,
        stats,
        results_path: results_path.unwrap_or_default(),
        created_at,
    })
}

pub async fn add_scan_log(
    app: &AppHandle,
    scan_id: i64,
    level: &str,
    message: &str,
) -> SqlResult<()> {
    let conn = get_connection(app)?;
    
    conn.execute(
        "INSERT INTO scan_logs (scan_id, level, message) VALUES (?1, ?2, ?3)",
        params![scan_id, level, message],
    )?;
    
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanLog {
    pub id: i64,
    pub scan_id: i64,
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub fn get_scan_logs(app: &AppHandle, scan_id: i64) -> SqlResult<Vec<ScanLog>> {
    let conn = get_connection(app)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, scan_id, created_at, level, message FROM scan_logs 
         WHERE scan_id = ?1 ORDER BY created_at ASC"
    )?;
    
    let rows = stmt.query_map(params![scan_id], |row| {
        Ok(ScanLog {
            id: row.get(0)?,
            scan_id: row.get(1)?,
            timestamp: row.get::<_, String>(2)?,
            level: row.get(3)?,
            message: row.get(4)?,
        })
    })?;
    
    let mut logs = Vec::new();
    for row in rows {
        logs.push(row?);
    }
    
    Ok(logs)
}

pub struct ScanInfo {
    pub run_name: String,
}

pub fn get_scan_info(app: &AppHandle, scan_id: i64) -> SqlResult<ScanInfo> {
    let conn = get_connection(app)?;
    
    let mut stmt = conn.prepare("SELECT run_name FROM scans WHERE id = ?1")?;
    
    let run_name = stmt.query_row(params![scan_id], |row| {
        Ok(row.get::<_, String>(0)?)
    })?;
    
    Ok(ScanInfo { run_name })
}

pub fn save_vulnerability(
    app: &AppHandle,
    scan_id: i64,
    vuln: &crate::Vulnerability,
) -> SqlResult<()> {
    let conn = get_connection(app)?;
    
    conn.execute(
        "INSERT OR REPLACE INTO vulnerabilities 
         (scan_id, vuln_id, title, severity, description, poc, remediation) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            scan_id,
            vuln.id,
            vuln.title,
            vuln.severity,
            vuln.description,
            vuln.poc,
            vuln.remediation
        ],
    )?;
    
    Ok(())
}

pub fn update_scan_results_path(
    app: &AppHandle,
    scan_id: i64,
    results_path: &str,
) -> SqlResult<()> {
    let conn = get_connection(app)?;
    
    conn.execute(
        "UPDATE scans SET results_path = ?1 WHERE id = ?2",
        params![results_path, scan_id],
    )?;
    
    Ok(())
}

pub fn save_setting(app: &AppHandle, key: &str, value: &str) -> SqlResult<()> {
    let conn = get_connection(app)?;
    
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    
    Ok(())
}

pub fn get_setting(app: &AppHandle, key: &str) -> SqlResult<Option<String>> {
    let conn = get_connection(app)?;
    
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    
    let mut rows = stmt.query_map(params![key], |row| {
        Ok(row.get::<_, String>(0)?)
    })?;
    
    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

pub fn get_all_settings(app: &AppHandle) -> SqlResult<std::collections::HashMap<String, String>> {
    let conn = get_connection(app)?;
    
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    
    let mut settings = std::collections::HashMap::new();
    for row in rows {
        let (key, value) = row?;
        settings.insert(key, value);
    }
    
    Ok(settings)
}

pub fn delete_scan(app: &AppHandle, scan_id: i64) -> SqlResult<()> {
    let conn = get_connection(app)?;
    
    // 由于外键约束设置了 ON DELETE CASCADE，删除扫描会自动删除相关的漏洞和日志
    conn.execute("DELETE FROM scans WHERE id = ?1", params![scan_id])?;
    
    Ok(())
}

pub fn clear_all_scans(app: &AppHandle) -> SqlResult<()> {
    let conn = get_connection(app)?;
    
    // 删除所有扫描（由于外键约束，会自动删除相关的漏洞和日志）
    conn.execute("DELETE FROM scans", [])?;
    
    Ok(())
}

