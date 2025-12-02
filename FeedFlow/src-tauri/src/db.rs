use rusqlite::{Connection, Result};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn get_db_path(app: &AppHandle) -> Result<PathBuf> {
    // 使用 Tauri 的应用数据目录
    let app_data_dir = app.path().app_data_dir().unwrap();
    std::fs::create_dir_all(&app_data_dir).unwrap();
    Ok(app_data_dir.join("rss-reader.db"))
}

pub fn init_database(db_path: &PathBuf) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    
    // 启用外键约束（PRAGMA 不返回结果，使用 execute）
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    
    // 设置 WAL 模式（PRAGMA 可能返回结果，使用 execute_batch 或忽略结果）
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    
    // 创建表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            url TEXT NOT NULL UNIQUE,
            description TEXT,
            favicon TEXT,
            category_id INTEGER,
            last_update TEXT,
            is_subscribed INTEGER DEFAULT 1,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            feed_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            link TEXT,
            content TEXT,
            snippet TEXT,
            pub_date TEXT,
            guid TEXT,
            is_favorite INTEGER DEFAULT 0,
            ai_summary TEXT,
            title_translated TEXT,
            content_translated TEXT,
            detected_language TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE,
            UNIQUE(feed_id, guid)
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
        [],
    )?;
    
    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_articles_feed_id ON articles(feed_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_articles_pub_date ON articles(pub_date)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_feeds_category_id ON feeds(category_id)",
        [],
    )?;
    
    Ok(conn)
}

pub fn get_connection(app: &AppHandle) -> Result<Connection> {
    let db_path = get_db_path(app)?;
    init_database(&db_path)
}

