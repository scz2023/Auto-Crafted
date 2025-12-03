# 🕯️ 烛龙 (ZhuLong)

基于 Tauri + Nuxt 的 AI 驱动渗透测试工具，集成 Strix 安全扫描引擎。

## ✨ 特性

- 🎯 **图形化界面**：直观的 Web UI，无需命令行操作
- 🤖 **AI 驱动**：集成 Strix AI 代理，自动发现和验证安全漏洞
- 🐍 **Python 集成**：通过 pyo3 集成 Python，支持自动初始化和回退机制
- 📊 **扫描管理**：完整的扫描历史、状态跟踪和结果展示
- 📝 **实时日志**：实时显示扫描过程的输出日志
- 🔒 **安全可靠**：所有数据本地存储，保护隐私

## 🚀 快速开始

### 前置要求

- Node.js 18+
- Rust 1.70+
- Docker（Strix 需要 Docker 环境）

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装 Rust 依赖（会自动执行）
npm run tauri:dev
```

### 开发模式

```bash
npm run tauri:dev
```

### 构建应用

```bash
npm run tauri:build
```

## 📖 使用说明

### 1. 配置 LLM

在"新建扫描"页面配置：
- **LLM 提供商**：选择 OpenAI、Anthropic 或本地模型
- **API Key**：输入对应的 API Key
- **API Base URL**：如果使用本地模型（如 Ollama），需要设置 API Base URL

### 2. 创建扫描

1. 点击"新建扫描"
2. 输入扫描目标（URL、GitHub 仓库、本地路径或 IP 地址）
3. 可选：添加自定义指令，指定扫描重点
4. 点击"开始扫描"

### 3. 查看实时日志

1. 启动扫描后，切换到"扫描历史"页面
2. 点击运行中的扫描的"查看日志"按钮
3. 右侧会实时显示扫描输出

### 4. 查看结果

1. 在"扫描历史"中查看所有扫描任务
2. 点击"查看结果"查看详细的漏洞报告
3. 每个漏洞包含：标题、严重程度、描述、PoC 和修复建议

## 🛠️ 技术栈

- **前端**：Nuxt 4.2.1 + Vue 3 + TypeScript + Element Plus
- **后端**：Tauri 2.9.1 + Rust
- **Python 集成**：pyo3-embed（内置 Python 解释器）
- **安全引擎**：Strix 0.4.0
- **数据库**：SQLite (rusqlite)

## 📁 项目结构

```
ZhuLong/
├── src-tauri/              # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs         # 主入口，Tauri 命令定义
│   │   ├── db.rs           # SQLite 数据库操作
│   │   ├── strix.rs        # Strix 集成逻辑
│   │   └── python_embed.rs # Python 嵌入模块
│   ├── Cargo.toml          # Rust 依赖配置
│   └── tauri.conf.json     # Tauri 应用配置
├── pages/                  # Nuxt 页面
│   └── index.vue           # 主页面
├── components/             # Vue 组件
│   ├── NewScan.vue         # 新建扫描组件
│   ├── ScanHistory.vue     # 扫描历史组件
│   ├── ScanLogs.vue        # 实时日志组件
│   └── Settings.vue        # 设置组件
├── assets/                 # 静态资源
│   └── css/
│       └── global.css      # 全局样式
└── strix-0.4.0/           # Strix 源代码
```

## ⚙️ 配置说明

### Python 运行模式

应用支持两种 Python 运行模式：

1. **pyo3 集成**（默认）：通过 pyo3 直接调用 Python 解释器
   - 使用系统 Python，但通过 Rust 直接调用
   - 更好的性能和集成度
   - 可通过环境变量 `USE_EMBEDDED_PYTHON=false` 禁用

2. **命令行执行**（回退）：通过命令行执行 Python 脚本
   - 如果 pyo3 集成失败，自动回退
   - 需要系统安装 Python 3.12+
   - 更稳定，但性能稍低

### LLM 配置

- `STRIX_LLM`：LLM 模型名称（如 `openai/gpt-5`）
- `LLM_API_KEY`：LLM API Key
- `LLM_API_BASE`：可选，本地模型的 API Base URL

### Docker 要求

Strix 需要 Docker 来运行安全扫描环境：

1. 确保 Docker Desktop 已安装并运行
2. 在应用中点击"检查环境"验证 Docker 状态
3. 首次运行会自动拉取 Strix Docker 镜像

## 🔧 开发

### 添加新功能

#### 前端（Vue）

1. 在 `components/` 中创建新组件
2. 在 `pages/` 中创建新页面（如需要）
3. 使用 `invoke()` 调用 Tauri 命令

#### 后端（Rust）

1. 在 `src-tauri/src/` 中添加新模块
2. 在 `main.rs` 中定义新的 `#[tauri::command]` 函数
3. 在 `invoke_handler` 中注册新命令

### 调试

#### 前端调试
- 使用浏览器开发者工具（F12）
- 查看控制台日志

#### 后端调试
- 查看终端输出（Rust 的 `println!` 和 `eprintln!`）
- 使用 Rust 调试器（如 VS Code 的 rust-analyzer）

## 📚 相关资源

- [Tauri 文档](https://tauri.app/)
- [Nuxt 文档](https://nuxt.com/)
- [Strix 文档](https://github.com/usestrix/strix)
- [Element Plus 文档](https://element-plus.org/)
- [pyo3 文档](https://pyo3.rs/)

## ⚠️ 注意事项

1. **安全使用**：仅测试自己拥有或获得授权的应用
2. **API Key 安全**：不要将 API Key 提交到版本控制
3. **Docker 资源**：Strix 扫描会消耗 Docker 资源，注意监控
4. **数据隐私**：所有扫描数据存储在本地，注意备份

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📝 更新日志

### v1.0.0
- ✅ 集成 Strix 0.4.0
- ✅ 内置 Python 解释器支持（pyo3-embed）
- ✅ 实时日志功能
- ✅ 扫描结果自动解析
- ✅ 进程管理功能
