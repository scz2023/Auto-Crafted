// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

use db::get_connection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use feed_rs::parser;
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
struct DbQuery {
    sql: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParseFeedRequest {
    url: String,
    timeout: Option<u64>, // 超时时间（秒）
}

#[derive(Debug, Serialize, Deserialize)]
struct FeedItem {
    title: String,
    link: Option<String>,
    description: Option<String>,
    content: Option<String>,
    author: Option<String>,
    published: Option<String>,
    updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FeedData {
    title: String,
    description: Option<String>,
    link: Option<String>,
    items: Vec<FeedItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpmlFeed {
    title: String,
    url: String,
    description: Option<String>,
    category: Option<String>,
    html_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpmlData {
    title: Option<String>,
    feeds: Vec<OpmlFeed>,
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
        // 对于非 SELECT 语句，先尝试使用 execute
        // 如果返回结果错误，则使用 query_map 处理
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
        
        // 尝试使用 execute，如果失败且是因为返回了结果，则使用 query_map
        match stmt.execute(rusqlite::params_from_iter(params.iter())) {
            Ok(changes) => {
                Ok(serde_json::json!({
                    "success": true,
                    "data": {
                        "lastInsertRowid": conn.last_insert_rowid(),
                        "changes": changes
                    }
                }))
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("Execute returned results") {
                    // 如果 execute 返回了结果错误，说明这个语句实际上返回了结果
                    // 重新准备语句并使用 query_map
                    let mut stmt2 = conn
                        .prepare(&query.sql)
                        .map_err(|e| format!("SQL 准备失败: {}", e))?;
                    
                    let rows = stmt2
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
                    Err(format!("执行失败: {}", error_msg))
                }
            }
        }
    }
}

// RSS 解析命令 - 使用 Rust 直接解析，无需 Node.js 服务器
#[tauri::command]
async fn parse_feed(request: ParseFeedRequest) -> Result<FeedData, String> {
    let timeout_secs = request.timeout.unwrap_or(30);
    
    println!("[parse_feed] 开始解析 RSS: {}", request.url);
    
    // 标准化 URL（处理 rsshub:// 协议）
    let normalized_url = normalize_rss_url(&request.url);
    println!("[parse_feed] 标准化后的 URL: {}", normalized_url);
    
    // 创建 HTTP 客户端
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    // 获取 RSS 内容
    let response = client
        .get(&normalized_url)
        .send()
        .await
        .map_err(|e| format!("获取 RSS 失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP 错误: {}", response.status()));
    }
    
    let content = response
        .bytes()
        .await
        .map_err(|e| format!("读取响应内容失败: {}", e))?;
    
    // 解析 RSS/Atom feed
    let feed = parser::parse(content.as_ref())
        .map_err(|e| format!("解析 RSS 失败: {}", e))?;
    
    // 转换 feed_rs 格式到我们的格式
    let items: Vec<FeedItem> = feed
        .entries
        .iter()
        .map(|entry| {
            FeedItem {
                title: entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default(),
                link: entry.links.first().map(|l| l.href.clone()),
                description: entry.summary.as_ref().map(|s| s.content.clone()),
                content: entry.content.as_ref().and_then(|c| {
                    c.body.as_ref().map(|b| b.clone())
                }),
                author: entry.authors.first().map(|a| a.name.clone()),
                published: entry.published.map(|d| d.to_rfc3339()),
                updated: entry.updated.map(|d| d.to_rfc3339()),
            }
        })
        .collect();
    
    let feed_data = FeedData {
        title: feed.title.as_ref().map(|t| t.content.clone()).unwrap_or_default(),
        description: feed.description.as_ref().map(|d| d.content.clone()),
        link: feed.links.first().map(|l| l.href.clone()),
        items,
    };
    
    println!("[parse_feed] 解析成功: {} 个条目", feed_data.items.len());
    
    Ok(feed_data)
}

// 标准化 RSS URL（处理 rsshub:// 协议）
fn normalize_rss_url(url: &str) -> String {
    // 处理 rsshub:// 协议
    if url.starts_with("rsshub://") {
        let path = url.strip_prefix("rsshub://").unwrap_or(url);
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        
        if let Some(domain) = parts.first() {
            if domain.contains('.') || domain.starts_with("localhost") {
                // 自定义域名
                if parts.len() > 1 {
                    format!("https://{}/{}", domain, parts[1])
                } else {
                    format!("https://{}", domain)
                }
            } else {
                // 默认 rsshub.app
                format!("https://rsshub.app/{}", path)
            }
        } else {
            format!("https://rsshub.app/{}", path)
        }
    } else if url.starts_with("rsshub:") {
        let path = url.strip_prefix("rsshub:").unwrap_or(url).trim_start_matches('/');
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        
        if let Some(domain) = parts.first() {
            if domain.contains('.') || domain.starts_with("localhost") {
                // 自定义域名
                if parts.len() > 1 {
                    format!("https://{}/{}", domain, parts[1])
                } else {
                    format!("https://{}", domain)
                }
            } else {
                // 默认 rsshub.app
                format!("https://rsshub.app/{}", path)
            }
        } else {
            format!("https://rsshub.app/{}", path)
        }
    } else {
        url.to_string()
    }
}

// OPML 获取命令
#[tauri::command]
async fn fetch_opml(url: String) -> Result<String, String> {
    println!("[fetch_opml] 开始获取 OPML: {}", url);
    
    // 验证 URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("只支持 HTTP 和 HTTPS URL".to_string());
    }
    
    // 创建 HTTP 客户端
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    // 获取 OPML 内容
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("获取 OPML 失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP 错误: {}", response.status()));
    }
    
    let text = response
        .text()
        .await
        .map_err(|e| format!("读取响应内容失败: {}", e))?;
    
    println!("[fetch_opml] 获取成功，内容长度: {} 字节", text.len());
    Ok(text)
}

// OPML 解析命令 - 使用 Rust 解析 OPML 文件
// 注意：参数名使用 camelCase（opmlText），与前端 invoke 保持一致
#[tauri::command]
async fn parse_opml(opmlText: String) -> Result<OpmlData, String> {
    println!("[parse_opml] 开始解析 OPML，内容长度: {} 字节", opmlText.len());
    
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use std::collections::HashMap;
    
    let mut reader = Reader::from_str(&opmlText);
    reader.trim_text(true);
    reader.check_end_names(false); // 不检查结束标签名称，提高容错性
    
    let mut title: Option<String> = None;
    let mut feeds: Vec<OpmlFeed> = Vec::new();
    let mut category_path_stack: Vec<Vec<String>> = vec![Vec::new()]; // 使用栈来管理嵌套的分类路径
    let mut buf = Vec::new();
    let mut current_outline: HashMap<String, String> = HashMap::new();
    let mut in_head = false;
    let mut in_body = false;
    let mut in_title = false;
    let mut current_text = String::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"opml" => {
                        // OPML 根元素
                    }
                    b"head" => {
                        in_head = true;
                    }
                    b"title" if in_head => {
                        in_title = true;
                        current_text.clear();
                    }
                    b"body" => {
                        in_body = true;
                        in_head = false;
                    }
                    b"outline" => {
                        current_outline.clear();
                        // 读取所有属性
                        for attr_result in e.attributes() {
                            match attr_result {
                                Ok(attr) => {
                                    // 解析属性键
                                    let key_result = attr.key.as_ref().try_into();
                                    let key = match key_result {
                                        Ok(k) => String::from_utf8_lossy(k).to_string(),
                                        Err(_) => {
                                            // 如果直接转换失败，尝试从 BytesStart 获取
                                            String::from_utf8_lossy(attr.key.as_ref()).to_string()
                                        }
                                    };
                                    
                                    // 解析属性值（XML 属性值通常已经是 unescaped 的）
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    
                                    current_outline.insert(key, value);
                                }
                                Err(e) => {
                                    println!("[parse_opml] 警告: 解析属性失败: {}", e);
                                }
                            }
                        }
                        
                        // 如果是分类文件夹，创建新的路径层级
                        let text = current_outline.get("text")
                            .or_else(|| current_outline.get("title"))
                            .cloned()
                            .unwrap_or_default();
                        let xml_url = current_outline.get("xmlUrl");
                        if xml_url.is_none() && !text.is_empty() {
                            // 这是一个分类文件夹，创建新的路径
                            let mut new_path = category_path_stack.last().unwrap().clone();
                            new_path.push(text);
                            category_path_stack.push(new_path);
                        }
                    }
                    _ => {}
                }
            }
            // 处理自闭合的 outline 标签：<outline ... />
            Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"outline" && in_body {
                    current_outline.clear();
                    // 读取所有属性
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            // 解析属性键
                            let key_result = attr.key.as_ref().try_into();
                            let key = match key_result {
                                Ok(k) => String::from_utf8_lossy(k).to_string(),
                                Err(_) => String::from_utf8_lossy(attr.key.as_ref()).to_string(),
                            };

                            // 属性值
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            current_outline.insert(key, value);
                        }
                    }

                    // 处理 outline 元素（与 End(outline) 中的逻辑基本一致）
                    let xml_url = current_outline.get("xmlUrl").cloned();
                    let text = current_outline.get("text")
                        .or_else(|| current_outline.get("title"))
                        .cloned()
                        .unwrap_or_default();
                    let html_url = current_outline.get("htmlUrl").cloned();
                    let description = current_outline.get("description").cloned();
                    let category_attr = current_outline.get("category").cloned().unwrap_or_default();
                    let outline_type = current_outline.get("type").cloned().unwrap_or_default();

                    // 如果有 xmlUrl，说明这是一个 RSS feed（即你说的订阅源）
                    if let Some(url) = xml_url {
                        if outline_type == "rss" || outline_type == "atom" || outline_type.is_empty() {
                            // 解析分类
                            let mut final_category: Option<String> = None;

                            if !category_attr.is_empty() {
                                let categories: Vec<&str> = category_attr
                                    .split(',')
                                    .map(|s| s.trim())
                                    .filter(|s| !s.is_empty() && s.to_lowercase() != "all")
                                    .collect();

                                if !categories.is_empty() {
                                    final_category = Some(categories[0].to_string());
                                }
                            }

                            // 如果没有从 category 属性获取到分类，使用父级路径
                            let current_path = category_path_stack.last().unwrap();
                            if final_category.is_none() && !current_path.is_empty() {
                                final_category = Some(current_path.join(" / "));
                            }

                            feeds.push(OpmlFeed {
                                title: if text.is_empty() { "未命名订阅".to_string() } else { text },
                                url,
                                description,
                                category: final_category,
                                html_url,
                            });
                        }
                    }

                    current_outline.clear();
                }
            }
            Ok(Event::Text(e)) => {
                if in_title {
                    match e.unescape() {
                        Ok(unescaped) => {
                            current_text.push_str(&unescaped);
                        }
                        Err(_) => {
                            // 如果 unescape 失败，直接使用原始字节
                            current_text.push_str(&String::from_utf8_lossy(&e));
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"title" if in_head => {
                        title = Some(current_text.trim().to_string());
                        in_title = false;
                        current_text.clear();
                    }
                    b"head" => {
                        in_head = false;
                    }
                    b"outline" if in_body => {
                        // 处理 outline 元素
                        let xml_url = current_outline.get("xmlUrl").cloned();
                        let text = current_outline.get("text")
                            .or_else(|| current_outline.get("title"))
                            .cloned()
                            .unwrap_or_default();
                        let html_url = current_outline.get("htmlUrl").cloned();
                        let description = current_outline.get("description").cloned();
                        let category_attr = current_outline.get("category").cloned().unwrap_or_default();
                        let outline_type = current_outline.get("type").cloned().unwrap_or_default();
                        
                        // 如果有 xmlUrl，说明这是一个 RSS feed
                        if let Some(url) = xml_url {
                            if outline_type == "rss" || outline_type == "atom" || outline_type.is_empty() {
                                // 解析分类
                                let mut final_category: Option<String> = None;
                                
                                if !category_attr.is_empty() {
                                    let categories: Vec<&str> = category_attr
                                        .split(',')
                                        .map(|s| s.trim())
                                        .filter(|s| !s.is_empty() && s.to_lowercase() != "all")
                                        .collect();
                                    
                                    if !categories.is_empty() {
                                        final_category = Some(categories[0].to_string());
                                    }
                                }
                                
                                // 如果没有从 category 属性获取到分类，使用父级路径
                                let current_path = category_path_stack.last().unwrap();
                                if final_category.is_none() && !current_path.is_empty() {
                                    final_category = Some(current_path.join(" / "));
                                }
                                
                                feeds.push(OpmlFeed {
                                    title: if text.is_empty() { "未命名订阅".to_string() } else { text },
                                    url,
                                    description,
                                    category: final_category,
                                    html_url,
                                });
                            }
                        }
                        
                        // 如果是分类文件夹的结束，弹出路径栈
                        let text_end = current_outline.get("text")
                            .or_else(|| current_outline.get("title"))
                            .cloned()
                            .unwrap_or_default();
                        let xml_url_end = current_outline.get("xmlUrl");
                        if xml_url_end.is_none() && !text_end.is_empty() && category_path_stack.len() > 1 {
                            category_path_stack.pop();
                        }
                        
                        current_outline.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                let error_msg = format!("解析 OPML 失败: {} (位置: {:?})", e, reader.buffer_position());
                println!("[parse_opml] {}", error_msg);
                return Err(error_msg);
            }
            _ => {}
        }
        buf.clear();
    }
    
    println!("[parse_opml] 解析成功: {} 个订阅源", feeds.len());
    
    Ok(OpmlData {
        title,
        feeds,
    })
}

// 获取完整文章内容命令
#[tauri::command]
async fn fetch_full_article(url: String) -> Result<serde_json::Value, String> {
    println!("[fetch_full_article] 开始获取文章内容: {}", url);
    
    // 验证 URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("只支持 HTTP 和 HTTPS URL".to_string());
    }
    
    // 创建 HTTP 客户端
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    // 获取 HTML 内容
    let html = client
        .get(&url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .send()
        .await
        .map_err(|e| format!("获取文章失败: {}", e))?
        .text()
        .await
        .map_err(|e| format!("读取响应内容失败: {}", e))?;
    
    // 简单的 HTML 内容提取
    let mut content = html.clone();
    
    // 尝试提取常见的文章内容容器
    // 使用原始字符串字面量，避免转义问题
    let patterns = vec![
        (r#"(?i)<article[^>]*>([\s\S]*?)</article>"#, 1),
        (r#"(?i)<main[^>]*>([\s\S]*?)</main>"#, 1),
        (r#"(?i)<div[^>]*class="[^"]*content[^"]*"[^>]*>([\s\S]*?)</div>"#, 1),
        (r#"(?i)<div[^>]*class="[^"]*post[^"]*"[^>]*>([\s\S]*?)</div>"#, 1),
        (r#"(?i)<div[^>]*class="[^"]*article[^"]*"[^>]*>([\s\S]*?)</div>"#, 1),
        (r#"(?i)<div[^>]*id="[^"]*content[^"]*"[^>]*>([\s\S]*?)</div>"#, 1),
    ];
    
    // 使用正则表达式提取内容
    for (pattern, _) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(&html) {
                if let Some(matched) = captures.get(1) {
                    content = matched.as_str().to_string();
                    break;
                }
            }
        }
    }
    
    // 如果没找到特定容器，尝试提取 body 标签内容
    if content == html {
        if let Ok(re) = Regex::new(r#"(?i)<body[^>]*>([\s\S]*?)</body>"#) {
            if let Some(captures) = re.captures(&html) {
                if let Some(matched) = captures.get(1) {
                    content = matched.as_str().to_string();
                }
            }
        }
    }
    
    // 清理不需要的标签
    let cleanup_patterns = vec![
        r#"(?i)<script[^>]*>[\s\S]*?</script>"#,
        r#"(?i)<style[^>]*>[\s\S]*?</style>"#,
        r#"(?i)<nav[^>]*>[\s\S]*?</nav>"#,
        r#"(?i)<header[^>]*>[\s\S]*?</header>"#,
        r#"(?i)<footer[^>]*>[\s\S]*?</footer>"#,
        r#"(?i)<aside[^>]*>[\s\S]*?</aside>"#,
    ];
    
    for pattern in cleanup_patterns {
        if let Ok(re) = Regex::new(pattern) {
            content = re.replace_all(&content, "").to_string();
        }
    }
    
    println!("[fetch_full_article] 提取成功，内容长度: {} 字符", content.len());
    
    Ok(serde_json::json!({
        "success": true,
        "content": content,
        "url": url
    }))
}

// 获取 Favicon 命令
#[tauri::command]
async fn get_favicon(url: String) -> Result<serde_json::Value, String> {
    println!("[get_favicon] 开始获取 Favicon: {}", url);
    
    // 验证 URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("只支持 HTTP 和 HTTPS URL".to_string());
    }
    
    // 构建 favicon URL - 简单地从 URL 中提取域名
    let favicon_url = if let Some(protocol_end) = url.find("://") {
        let after_protocol = &url[protocol_end + 3..];
        if let Some(path_start) = after_protocol.find('/') {
            format!("{}://{}/favicon.ico", &url[..protocol_end + 3], &after_protocol[..path_start])
        } else {
            format!("{}/favicon.ico", url)
        }
    } else {
        return Err("无效的 URL 格式".to_string());
    };
    
    // 创建 HTTP 客户端（短超时）
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    // 尝试获取 favicon
    match client.get(&favicon_url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                // 404 或其他错误，返回 null
                return Ok(serde_json::json!({
                    "success": false,
                    "favicon": null
                }));
            }
            
            // 检查内容类型
            if let Some(content_type) = response.headers().get("content-type") {
                if let Ok(ct_str) = content_type.to_str() {
                    if !ct_str.starts_with("image/") {
                        return Ok(serde_json::json!({
                            "success": false,
                            "favicon": null
                        }));
                    }
                }
            }
            
            println!("[get_favicon] 获取成功: {}", favicon_url);
            Ok(serde_json::json!({
                "success": true,
                "favicon": favicon_url
            }))
        }
        Err(_) => {
            // 获取失败，返回 null（不抛出错误）
            Ok(serde_json::json!({
                "success": false,
                "favicon": null
            }))
        }
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
    let mut possible_paths: Vec<PathBuf> = vec![
        // 便携版：服务器在 .output/server 目录（与 exe 同级）
        exe_dir.join(".output").join("server").join("index.mjs"),
    ];
    
    // 便携版：服务器在 exe 所在目录的父目录
    if let Some(parent) = exe_dir.parent() {
        possible_paths.push(parent.join(".output").join("server").join("index.mjs"));
    }
    
    // 如果资源目录存在，也检查那里
    if let Some(ref res_dir) = resource_dir {
        possible_paths.push(res_dir.join(".output").join("server").join("index.mjs"));
        possible_paths.push(res_dir.join("server").join("index.mjs"));
    }
    
    // 开发环境：在项目根目录
    if let Some(parent) = exe_dir.parent() {
        if let Some(grandparent) = parent.parent() {
            possible_paths.push(grandparent.join(".output").join("server").join("index.mjs"));
        }
    }
    
    let mut server_dir: Option<PathBuf> = None;
    let mut server_index: Option<PathBuf> = None;
    let mut tried_paths = Vec::new();
    
    for path in possible_paths {
        tried_paths.push(path.clone());
        if path.exists() {
            println!("Found server at: {:?}", path);
            server_index = Some(path.clone());
            server_dir = path.parent().map(|p| p.to_path_buf());
            break;
        }
    }
    
    let (server_dir, server_index): (PathBuf, PathBuf) = match (server_dir, server_index) {
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
       .stdout(std::process::Stdio::piped())  // 保留输出以便调试
       .stderr(std::process::Stdio::piped());
    
    // 在 Windows 上隐藏控制台窗口
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    
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
    let server_process: ServerProcess = Arc::new(Mutex::new(None));
    let server_process_clone = server_process.clone();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![db_query, parse_feed, fetch_opml, parse_opml, fetch_full_article, get_favicon])
        .setup(move |app| {
            // 启用开发者工具（即使在 release 模式下也启用，以便调试）
            // 尝试获取主窗口，如果没有 label，则获取第一个窗口
            let window = app.get_webview_window("main")
                .or_else(|| {
                    // 如果没有找到 "main" 窗口，尝试获取第一个窗口
                    app.webview_windows().values().next().cloned()
                });
            
            if let Some(window) = window {
                #[cfg(debug_assertions)]
                {
                    let _ = window.open_devtools();
                }
                // 在 release 模式下，F12 快捷键应该已经通过配置文件中的 devtools: true 启用
                // 如果仍然不工作，可能需要检查 capabilities 配置
            }
            
            // 应用启动时启动服务器
            // 在开发模式下，Nuxt 开发服务器已经通过 beforeDevCommand 启动，不需要再启动 Nitro 服务器
            #[cfg(not(debug_assertions))]
            {
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
            }
            
            #[cfg(debug_assertions)]
            {
                println!("Development mode: Using Nuxt dev server (already running via beforeDevCommand)");
                println!("Server should be available at http://localhost:3000");
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
                        println!("Nitro 服务器已停止");
                    }
                }
            }
        });
}

