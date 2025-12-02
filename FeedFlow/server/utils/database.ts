import Database from 'better-sqlite3'
import { join, dirname } from 'path'
import { existsSync, mkdirSync } from 'fs'

let dbInstance: Database.Database | null = null

export function useDatabase(): Database.Database {
  if (dbInstance) {
    return dbInstance
  }

  try {
    // 在服务器端使用
    // 在 Nuxt 3 中，process.cwd() 应该指向项目根目录
    const dbPath = join(process.cwd(), 'rss-reader.db')
    
    console.log('Initializing database at:', dbPath)
    
    // 检查数据库文件目录是否存在，如果不存在则创建
    const dbDir = dirname(dbPath)
    if (!existsSync(dbDir)) {
      mkdirSync(dbDir, { recursive: true })
    }
    
    dbInstance = new Database(dbPath, {
      // 设置超时时间，避免数据库锁定
      timeout: 5000,
      // 启用 WAL 模式以提高并发性能
      verbose: process.env.NODE_ENV === 'development' ? console.log : undefined
    })
    
    // 启用外键约束
    dbInstance.pragma('foreign_keys = ON')
    
    // 设置 WAL 模式以提高并发性能
    try {
      dbInstance.pragma('journal_mode = WAL')
    } catch {
      // 如果设置失败，忽略（某些情况下可能不支持）
    }
    
    // 初始化数据库表
    initDatabase(dbInstance)
    
    console.log('Database initialized successfully')
    return dbInstance
  } catch (error: any) {
    console.error('Failed to initialize database:', error)
    console.error('Error stack:', error.stack)
    // 清理实例，以便下次重试
    dbInstance = null
    throw new Error(`Database initialization failed: ${error.message || 'Unknown error'}`)
  }
}

function initDatabase(db: Database.Database) {
  // 创建分类表
  db.exec(`
    CREATE TABLE IF NOT EXISTS categories (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL UNIQUE,
      description TEXT,
      created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )
  `)

  // 创建订阅表
  db.exec(`
    CREATE TABLE IF NOT EXISTS feeds (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      title TEXT NOT NULL,
      url TEXT NOT NULL UNIQUE,
      description TEXT,
      favicon TEXT,
      category_id INTEGER,
      last_update TEXT,
      created_at TEXT DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
    )
  `)
  
  // 如果表已存在但没有favicon字段，添加该字段
  try {
    db.exec(`ALTER TABLE feeds ADD COLUMN favicon TEXT`)
  } catch {
    // 字段已存在，忽略错误
  }

  // 如果表已存在但没有category_id字段，添加该字段
  try {
    db.exec(`ALTER TABLE feeds ADD COLUMN category_id INTEGER`)
    // 添加外键约束（如果表已存在）
    try {
      db.exec(`CREATE INDEX IF NOT EXISTS idx_feeds_category_id ON feeds(category_id)`)
    } catch {
      // 索引可能已存在
    }
  } catch {
    // 字段已存在，忽略错误
  }

  // 如果表已存在但没有is_subscribed字段，添加该字段（默认为已订阅，兼容旧数据）
  try {
    db.exec(`ALTER TABLE feeds ADD COLUMN is_subscribed INTEGER DEFAULT 1`)
    // 将现有订阅源都设置为已订阅
    db.exec(`UPDATE feeds SET is_subscribed = 1 WHERE is_subscribed IS NULL`)
  } catch {
    // 字段已存在，忽略错误
  }

  // 创建文章表
  db.exec(`
    CREATE TABLE IF NOT EXISTS articles (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      feed_id INTEGER NOT NULL,
      title TEXT NOT NULL,
      link TEXT,
      content TEXT,
      snippet TEXT,
      pub_date TEXT,
      guid TEXT,
      is_favorite INTEGER DEFAULT 0,
      created_at TEXT DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE,
      UNIQUE(feed_id, guid)
    )
  `)
  
  // 如果表已存在但没有is_favorite字段，添加该字段
  try {
    db.exec(`ALTER TABLE articles ADD COLUMN is_favorite INTEGER DEFAULT 0`)
  } catch {
    // 字段已存在，忽略错误
  }

  // 如果表已存在但没有ai_summary字段，添加该字段
  try {
    // 先检查字段是否存在
    const tableInfo = db.prepare(`PRAGMA table_info(articles)`).all() as any[]
    const hasAiSummary = tableInfo.some((col: any) => col.name === 'ai_summary')
    
    if (!hasAiSummary) {
      db.exec(`ALTER TABLE articles ADD COLUMN ai_summary TEXT`)
      console.log('Added ai_summary column to articles table')
    }
  } catch (error: any) {
    // 如果出错，尝试直接添加（可能字段已存在但检查失败）
    try {
      db.exec(`ALTER TABLE articles ADD COLUMN ai_summary TEXT`)
    } catch {
      // 字段已存在，忽略错误
    }
  }

  // 如果表已存在但没有翻译相关字段，添加这些字段
  try {
    const tableInfo = db.prepare(`PRAGMA table_info(articles)`).all() as any[]
    const columnNames = tableInfo.map((col: any) => col.name)
    
    if (!columnNames.includes('title_translated')) {
      db.exec(`ALTER TABLE articles ADD COLUMN title_translated TEXT`)
      console.log('Added title_translated column to articles table')
    }
    if (!columnNames.includes('content_translated')) {
      db.exec(`ALTER TABLE articles ADD COLUMN content_translated TEXT`)
      console.log('Added content_translated column to articles table')
    }
    if (!columnNames.includes('detected_language')) {
      db.exec(`ALTER TABLE articles ADD COLUMN detected_language TEXT`)
      console.log('Added detected_language column to articles table')
    }
  } catch (error: any) {
    // 如果出错，尝试直接添加
    try {
      db.exec(`ALTER TABLE articles ADD COLUMN title_translated TEXT`)
    } catch {}
    try {
      db.exec(`ALTER TABLE articles ADD COLUMN content_translated TEXT`)
    } catch {}
    try {
      db.exec(`ALTER TABLE articles ADD COLUMN detected_language TEXT`)
    } catch {}
  }

  // 创建设置表
  db.exec(`
    CREATE TABLE IF NOT EXISTS settings (
      key TEXT PRIMARY KEY,
      value TEXT
    )
  `)

  // 创建索引
  db.exec(`
    CREATE INDEX IF NOT EXISTS idx_articles_feed_id ON articles(feed_id);
    CREATE INDEX IF NOT EXISTS idx_articles_pub_date ON articles(pub_date);
  `)
}

