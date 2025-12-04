# 使用 Strix Docker 镜像说明

## 🐳 如果使用 Strix 的 Docker 镜像

### 答案：**不需要在主机上安装 Python 和依赖！**

Docker 镜像已经包含了所有必需的内容：

### ✅ Docker 镜像包含的内容

根据 `strix-0.4.0/containers/Dockerfile`，镜像已经包含：

1. **Python 3.12+**
   - 在镜像构建时已安装
   - 位于容器内部，不在主机上

2. **所有 Python 依赖**
   - 通过 `poetry install` 安装（第 161 行）
   - 包括：`litellm[proxy]`, `openai`, `pydantic`, `rich`, `fastapi`, `uvicorn` 等所有依赖
   - 安装在容器内的虚拟环境中

3. **所有安全工具**
   - nmap, sqlmap, nuclei, subfinder, naabu, ffuf 等
   - Playwright, Docker 客户端等
   - 所有工具都已预装在容器中

4. **Strix 源代码**
   - Strix 代码已复制到容器中（通过 Dockerfile）

### 📋 使用 Docker 镜像的优势

1. **零依赖安装**
   - 主机上不需要安装 Python
   - 主机上不需要安装任何 Python 依赖
   - 主机上不需要安装安全工具

2. **环境隔离**
   - 完全隔离的运行环境
   - 不会污染主机环境
   - 不会与其他项目冲突

3. **跨平台兼容**
   - 同一镜像可以在 Windows、Linux、Mac 上运行
   - 环境一致性有保障

4. **易于部署**
   - 只需安装 Docker
   - 拉取镜像即可使用
   - 无需配置复杂的环境

### ⚠️ 当前代码状态

**注意**：当前应用代码（`ZhuLong`）**还没有实现通过 Docker 运行 Strix**。

当前实现方式：
- 使用嵌入的 Python（pyo3）或系统 Python
- 直接在主机上运行 Strix 脚本
- 需要主机上安装 Python 和依赖

### 🔄 如何切换到 Docker 方式

如果要使用 Docker 方式，需要修改代码：

#### 1. 构建或拉取 Strix Docker 镜像

```bash
cd strix-0.4.0
docker build -f containers/Dockerfile -t strix:0.4.0 .
```

或者使用 Strix 官方提供的镜像（如果有的话）。

#### 2. 修改 `run_scan` 函数

将当前的 Python 执行方式改为 Docker 执行：

```rust
// 当前方式（需要 Python 和依赖）
let mut cmd = Command::new(&python_cmd);
cmd.arg(&strix_path).arg("--target")...

// 改为 Docker 方式（不需要 Python 和依赖）
let mut cmd = Command::new("docker");
cmd.arg("run")
    .arg("--rm")
    .arg("-v")
    .arg(format!("{}:/workspace", workspace_dir))
    .arg("-e")
    .arg(format!("STRIX_LLM={}", config.llm_provider))
    .arg("-e")
    .arg(format!("LLM_API_KEY={}", config.llm_api_key))
    .arg("strix:0.4.0")
    .arg("strix")
    .arg("--target")...
```

#### 3. 修改 LLM 测试函数

LLM 测试也可以通过 Docker 执行：

```rust
let mut cmd = Command::new("docker");
cmd.arg("run")
    .arg("--rm")
    .arg("-e")
    .arg(format!("STRIX_LLM={}", llm_provider))
    .arg("-e")
    .arg(format!("LLM_API_KEY={}", llm_api_key))
    .arg("strix:0.4.0")
    .arg("python")
    .arg("-c")
    .arg(&test_script);
```

### 📊 对比表

| 特性 | 当前方式（Python） | Docker 方式 |
|------|-------------------|------------|
| 主机需要 Python | ✅ 是 | ❌ 否 |
| 主机需要安装依赖 | ✅ 是 | ❌ 否 |
| 主机需要安装工具 | ✅ 是（部分） | ❌ 否 |
| 环境隔离 | ❌ 否 | ✅ 是 |
| 跨平台一致性 | ⚠️ 依赖系统环境 | ✅ 完全一致 |
| 部署复杂度 | ⚠️ 中等 | ✅ 简单（只需 Docker） |
| 性能开销 | ✅ 低 | ⚠️ 中等（容器启动） |
| 资源占用 | ✅ 低 | ⚠️ 中等（镜像体积） |

### 🎯 推荐方案

#### 开发环境
- 使用当前方式（Python + 依赖）
- 便于调试和开发
- 启动速度快

#### 生产环境
- 使用 Docker 方式
- 部署简单
- 环境一致
- 无需配置依赖

### 📝 总结

**如果使用 Strix 的 Docker 镜像：**
- ✅ **不需要**在主机上安装 Python
- ✅ **不需要**在主机上安装 Python 依赖
- ✅ **不需要**在主机上安装安全工具
- ✅ 只需要安装 **Docker**

**但是：**
- ⚠️ 当前代码还没有实现 Docker 方式
- ⚠️ 需要修改代码以支持 Docker 执行
- ⚠️ Docker 镜像体积较大（几 GB）

### 🔧 实现 Docker 支持的建议

如果需要实现 Docker 支持，可以：

1. **添加配置选项**
   - 在设置中添加"使用 Docker"选项
   - 让用户选择执行方式

2. **自动检测**
   - 检测 Docker 是否可用
   - 检测 Strix 镜像是否存在
   - 自动选择最佳执行方式

3. **混合模式**
   - LLM 测试使用系统 Python（快速）
   - 实际扫描使用 Docker（隔离）

