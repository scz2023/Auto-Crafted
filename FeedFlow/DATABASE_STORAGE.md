# RSS 数据存储说明

## 数据库结构

RSS 阅读器使用 SQLite 数据库存储数据，数据库文件位于项目根目录：`rss-reader.db`

### 数据表

#### 1. feeds（订阅表）
```sql
CREATE TABLE feeds (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  url TEXT NOT NULL UNIQUE,
  description TEXT,
  last_update TEXT,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
)
```

#### 2. articles（文章表）
```sql
CREATE TABLE articles (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feed_id INTEGER NOT NULL,
  title TEXT NOT NULL,
  link TEXT,
  content TEXT,
  snippet TEXT,
  pub_date TEXT,
  guid TEXT,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE,
  UNIQUE(feed_id, guid)
)
```

#### 3. settings（设置表）
```sql
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT
)
```

## 数据保存流程

### 1. 添加订阅时
当用户添加新的 RSS 订阅时：

1. **解析 RSS Feed** (`composables/useRss.ts`)
   - 使用 `rss-parser` 解析 RSS URL
   - 获取订阅信息和文章列表

2. **保存订阅信息** (`stores/feed.ts` - `addFeed`)
   ```typescript
   INSERT INTO feeds (title, url, description, last_update)
   VALUES (?, ?, ?, ?)
   ```

3. **保存文章** (`stores/feed.ts` - `saveArticles`)
   ```typescript
   INSERT OR IGNORE INTO articles (feed_id, title, link, content, snippet, pub_date, guid)
   VALUES (?, ?, ?, ?, ?, ?, ?)
   ```
   - 使用 `INSERT OR IGNORE` 避免重复文章（基于 `feed_id` 和 `guid` 的唯一约束）

### 2. 刷新订阅时
当用户刷新订阅时：

1. **重新解析 RSS Feed** (`stores/feed.ts` - `refreshFeed`)
2. **更新订阅信息**（标题、描述、最后更新时间）
3. **保存新文章**（只保存新文章，已存在的文章会被忽略）

## 数据存储位置

- **开发环境**：项目根目录 `rss-reader.db`
- **生产环境（Tauri）**：应用数据目录（通过 Tauri API 获取）

## 检查数据库内容

可以使用以下脚本检查数据库：

```bash
node check-db.js
```

或者使用 SQLite 工具直接查看：
```bash
sqlite3 rss-reader.db
```

## 注意事项

1. **文章去重**：使用 `(feed_id, guid)` 的唯一约束确保同一订阅的文章不会重复
2. **级联删除**：删除订阅时，相关的文章会自动删除（`ON DELETE CASCADE`）
3. **数据持久化**：所有数据都保存在本地 SQLite 数据库中，无需网络连接即可查看

## 当前状态

运行 `node check-db.js` 可以查看：
- 订阅数量
- 文章总数
- 各订阅的文章统计
- 最新文章列表

如果数据库为空，说明：
1. 还没有添加任何订阅
2. 或者添加订阅时出现了错误（检查浏览器控制台）

