# 烛龙 (ZhuLong) 项目设置指南

## 📋 项目概述

烛龙是一个基于 Tauri + Nuxt 的桌面应用，集成了 Strix AI 渗透测试工具，提供图形化界面进行安全扫描。

## 🛠️ 环境要求

### 必需
- **Node.js** 18+ 
- **Rust** 1.70+
- **Docker** (Strix 需要 Docker 环境)

### 可选
- **Python** 3.12+ (如果本地没有，应用会尝试使用内置运行时)

## 📦 安装步骤

### 1. 安装依赖

```bash
cd ZhuLong
npm install
```

### 2. 验证 Strix 源代码

确保 `strix-0.4.0` 目录存在，包含完整的 Strix 源代码。

### 3. 开发模式运行

```bash
npm run tauri:dev
```

这将：
- 启动 Nuxt 开发服务器（前端）
- 编译 Rust 代码（后端）
- 启动 Tauri 应用窗口

### 4. 构建生产版本

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/`

## ⚙️ 配置说明

### LLM 配置

在应用中配置 LLM 提供商：

1. **OpenAI**
   - LLM 提供商：`openai/gpt-5`
   - 需要 API Key

2. **Anthropic**
   - LLM 提供商：`anthropic/claude-sonnet-4-5`
   - 需要 API Key

3. **本地模型（Ollama）**
   - LLM 提供商：`ollama/llama3`
   - API Base URL：`http://localhost:11434`
   - API Key：可选

### Docker 配置

Strix 需要 Docker 来运行安全扫描环境：

1. 确保 Docker Desktop 已安装并运行
2. 在应用中点击"检查环境"验证 Docker 状态
3. 首次运行会自动拉取 Strix Docker 镜像

## 🐛 常见问题

### 1. Python 未找到

**问题**：应用无法找到 Python 解释器

**解决方案**：
- 确保系统已安装 Python 3.12+
- 或使用应用内置的 Python 运行时（需要额外配置）

### 2. Docker 连接失败

**问题**：无法连接到 Docker

**解决方案**：
- 确保 Docker Desktop 正在运行
- 检查 Docker 服务状态：`docker ps`
- 在 Windows 上，确保 WSL2 已正确配置

### 3. Strix 主程序未找到

**问题**：启动扫描时提示"Strix 主程序未找到"

**解决方案**：
- 确保 `strix-0.4.0` 目录存在于项目根目录
- 检查 `strix-0.4.0/strix/interface/main.py` 是否存在
- 在开发环境中，确保从项目根目录运行

### 4. 数据库初始化失败

**问题**：应用启动时数据库初始化失败

**解决方案**：
- 检查应用数据目录权限
- 在 Windows 上，确保有写入 `%APPDATA%` 的权限
- 查看控制台错误信息

## 📝 开发说明

### 项目结构

```
ZhuLong/
├── src-tauri/              # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs         # 主入口，Tauri 命令定义
│   │   ├── db.rs           # SQLite 数据库操作
│   │   └── strix.rs        # Strix 集成逻辑
│   ├── Cargo.toml          # Rust 依赖配置
│   └── tauri.conf.json     # Tauri 应用配置
├── pages/                  # Nuxt 页面
│   └── index.vue           # 主页面
├── components/             # Vue 组件
│   ├── NewScan.vue         # 新建扫描组件
│   ├── ScanHistory.vue     # 扫描历史组件
│   └── Settings.vue       # 设置组件
├── assets/                 # 静态资源
│   └── css/
│       └── global.css      # 全局样式
└── strix-0.4.0/           # Strix 源代码
```

### 添加新功能

#### 前端（Vue）

1. 在 `components/` 中创建新组件
2. 在 `pages/` 中创建新页面（如需要）
3. 使用 `invoke()` 调用 Tauri 命令

#### 后端（Rust）

1. 在 `src-tauri/src/` 中添加新模块
2. 在 `main.rs` 中定义新的 `#[tauri::command]` 函数
3. 在 `invoke_handler` 中注册新命令

### 数据库操作

数据库文件位于应用数据目录：
- Windows: `%APPDATA%\zhulong\zhulong.db`
- macOS: `~/Library/Application Support/zhulong/zhulong.db`
- Linux: `~/.local/share/zhulong/zhulong.db`

### 调试

#### 前端调试
- 使用浏览器开发者工具（F12）
- 查看控制台日志

#### 后端调试
- 查看终端输出（Rust 的 `println!` 和 `eprintln!`）
- 使用 Rust 调试器（如 VS Code 的 rust-analyzer）

## 🚀 部署

### Windows

构建后会在 `src-tauri/target/release/bundle/msi/` 生成 MSI 安装包。

### macOS

需要配置签名和公证（用于分发）。

### Linux

生成 AppImage 或 deb 包。

## 📚 相关资源

- [Tauri 文档](https://tauri.app/)
- [Nuxt 文档](https://nuxt.com/)
- [Strix 文档](https://github.com/usestrix/strix)
- [Element Plus 文档](https://element-plus.org/)

## ⚠️ 注意事项

1. **安全使用**：仅测试自己拥有或获得授权的应用
2. **API Key 安全**：不要将 API Key 提交到版本控制
3. **Docker 资源**：Strix 扫描会消耗 Docker 资源，注意监控
4. **数据隐私**：所有扫描数据存储在本地，注意备份

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

