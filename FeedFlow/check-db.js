import Database from 'better-sqlite3';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const dbPath = join(__dirname, 'rss-reader.db');
const db = new Database(dbPath);

console.log('=== 数据库检查 ===\n');

// 检查表是否存在
const tables = db.prepare("SELECT name FROM sqlite_master WHERE type='table'").all();
console.log('数据库表:', tables.map(t => t.name).join(', '));
console.log('');

// 检查订阅数量
const feedCount = db.prepare('SELECT COUNT(*) as count FROM feeds').get();
console.log(`订阅数量: ${feedCount.count}`);

if (feedCount.count > 0) {
  const feeds = db.prepare('SELECT id, title, url FROM feeds').all();
  console.log('\n订阅列表:');
  feeds.forEach(feed => {
    console.log(`  - [${feed.id}] ${feed.title}`);
    console.log(`    URL: ${feed.url}`);
  });
}

console.log('');

// 检查文章数量
const articleCount = db.prepare('SELECT COUNT(*) as count FROM articles').get();
console.log(`文章总数: ${articleCount.count}`);

if (articleCount.count > 0) {
  // 按订阅分组统计
  const articlesByFeed = db.prepare(`
    SELECT 
      f.title as feed_title,
      COUNT(a.id) as article_count
    FROM feeds f
    LEFT JOIN articles a ON f.id = a.feed_id
    GROUP BY f.id
  `).all();
  
  console.log('\n各订阅的文章数量:');
  articlesByFeed.forEach(item => {
    console.log(`  - ${item.feed_title}: ${item.article_count} 篇`);
  });
  
  // 显示最新的5篇文章
  const latestArticles = db.prepare(`
    SELECT 
      a.id,
      a.title,
      f.title as feed_name,
      a.pub_date
    FROM articles a
    LEFT JOIN feeds f ON a.feed_id = f.id
    ORDER BY a.pub_date DESC
    LIMIT 5
  `).all();
  
  console.log('\n最新5篇文章:');
  latestArticles.forEach(article => {
    console.log(`  - [${article.feed_name}] ${article.title}`);
    console.log(`    发布时间: ${article.pub_date || '未知'}`);
  });
}

db.close();

