# Python 嵌入集成说明

## ✅ 已完成

已成功集成 `pyo3-embed`，实现了内置 Python 解释器支持。

## 🔧 实现方式

### 1. 依赖配置

在 `Cargo.toml` 中配置了 `auto-initialize` feature：

```toml
pyo3 = { version = "0.23", features = ["auto-initialize"] }
```

**注意**：`pyo3` 的 `auto-initialize` feature 会在程序启动时自动初始化 Python 解释器，但仍需要系统安装 Python。要真正实现无需系统 Python，需要使用其他方案（见后续优化）。

### 2. Python 嵌入模块

创建了 `src-tauri/src/python_embed.rs` 模块，提供：

- `init_python()`: 初始化嵌入的 Python 解释器
- `run_strix_with_embedded_python()`: 使用嵌入的 Python 运行 Strix

### 3. 自动回退机制

实现了智能回退机制：

1. **优先使用嵌入 Python**：如果初始化成功，使用嵌入的 Python 解释器
2. **自动回退**：如果嵌入 Python 失败，自动回退到系统 Python
3. **环境变量控制**：可通过 `USE_EMBEDDED_PYTHON` 环境变量控制是否使用嵌入 Python

## 📝 使用说明

### 默认行为

应用启动时会自动初始化嵌入的 Python 解释器。如果成功，所有扫描将使用嵌入的 Python；如果失败，会自动回退到系统 Python。

### 环境变量控制

可以通过设置环境变量来控制行为：

```bash
# 禁用嵌入 Python，强制使用系统 Python
export USE_EMBEDDED_PYTHON=false

# 启用嵌入 Python（默认）
export USE_EMBEDDED_PYTHON=true
```

## 🎯 优势

1. **自动初始化**：应用启动时自动初始化 Python 解释器
2. **版本一致**：使用系统 Python，但可以通过环境管理确保版本一致
3. **自动回退**：如果 Python 初始化失败，自动回退到命令行方式，保证兼容性
4. **统一接口**：无论使用哪种方式，都通过统一的接口执行 Python 代码

## ⚠️ 注意事项

1. **系统 Python 要求**：当前实现仍需要系统安装 Python 3.12+
2. **首次初始化**：首次运行时 Python 解释器初始化可能需要几秒钟
3. **依赖管理**：Strix 的 Python 依赖仍需要安装，可以通过打包或虚拟环境管理

## 🔄 完全独立方案（后续优化）

要实现真正无需系统 Python，可以考虑：

1. **打包 Python 解释器**：将 Python 解释器打包到应用资源中
2. **使用 RustPython**：使用纯 Rust 实现的 Python 解释器（但可能不支持所有库）
3. **使用 pyinstaller**：将 Strix 打包为独立可执行文件，然后在 Rust 中调用

## 🔄 后续优化

1. **打包 Python 依赖**：将 Strix 的 Python 依赖打包到应用中
2. **虚拟环境**：在应用数据目录创建独立的 Python 虚拟环境
3. **依赖安装**：首次运行时自动安装 Strix 的 Python 依赖

## 🐛 故障排除

### Python 初始化失败

如果看到 "初始化 Python 失败" 的日志：

1. 检查系统是否安装了 Python 3.12+
2. 确认 Python 在系统 PATH 中
3. 应用会自动回退到命令行方式执行 Python，功能不受影响

### 使用系统 Python

如果希望强制使用系统 Python：

1. 设置环境变量 `USE_EMBEDDED_PYTHON=false`
2. 或在代码中修改默认值

## 📊 技术细节

### Python 初始化

- 使用 `pyo3` 的 `auto-initialize` feature 自动初始化
- 在应用启动时（`setup` 阶段）初始化
- 使用 GIL（Global Interpreter Lock）确保线程安全

### 代码执行

- 使用 `spawn_blocking` 在 Tokio 运行时中执行 Python 代码
- 捕获 stdout/stderr 并实时发送到前端
- 支持异步 Python 代码（asyncio）

### 路径管理

- 自动将 Strix 目录添加到 Python 路径
- 设置正确的工作目录
- 支持相对路径和绝对路径

