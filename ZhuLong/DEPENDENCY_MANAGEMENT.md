# Strix 依赖管理说明

## 📦 依赖安装位置

### 1. 使用 `pip install litellm[proxy]`

**安装位置**：**全局 Python 环境**

- 如果直接运行 `pip install litellm[proxy]`，依赖会安装到系统 Python 的全局 site-packages 目录
- Windows 示例路径：`C:\Users\<用户名>\AppData\Local\Programs\Python\Python312\Lib\site-packages\`
- Linux/Mac 示例路径：`/usr/local/lib/python3.12/site-packages/` 或 `~/.local/lib/python3.12/site-packages/`

**优点**：
- 简单直接，所有 Python 环境都可以使用

**缺点**：
- 可能与其他项目的依赖冲突
- 全局污染系统环境

### 2. 使用 `poetry install`（推荐）

**安装位置**：**Poetry 管理的虚拟环境**

在 `strix-0.4.0` 目录中运行：
```bash
cd strix-0.4.0
poetry install
```

Poetry 会创建一个独立的虚拟环境，通常位于：
- Windows: `%APPDATA%\pypoetry\Cache\virtualenvs\`
- Linux/Mac: `~/.cache/pypoetry/virtualenvs/`

**优点**：
- 依赖隔离，不会影响系统环境
- 版本锁定，确保依赖版本一致
- 可以包含所有依赖（包括 dev 依赖）

**缺点**：
- 需要安装 Poetry
- 虚拟环境路径可能不固定

### 3. 使用 `python -m venv` + `pip install`

**安装位置**：**项目本地虚拟环境**

```bash
cd strix-0.4.0
python -m venv venv
# Windows
venv\Scripts\activate
# Linux/Mac
source venv/bin/activate
pip install -r requirements.txt  # 如果有的话
# 或者
pip install litellm[proxy] openai pydantic rich ...
```

虚拟环境位于 `strix-0.4.0/venv/` 目录中。

**优点**：
- 依赖与项目绑定，便于打包
- 不依赖外部工具（Poetry）

**缺点**：
- 需要手动管理依赖列表
- 虚拟环境目录较大，可能不适合打包

## 🔄 跨机器部署

### 问题

**如果只是拷贝 `strix-0.4.0` 目录到其他电脑，依赖不会一起拷贝！**

原因：
1. 全局安装的依赖在系统 Python 目录中，不在 `strix-0.4.0` 目录中
2. Poetry 虚拟环境通常在用户缓存目录，不在项目目录中
3. 即使使用本地 venv，虚拟环境可能包含系统特定的路径

### 解决方案

#### 方案 1：在目标机器上重新安装依赖（推荐）

**步骤**：
1. 拷贝 `strix-0.4.0` 目录到目标机器
2. 在目标机器上安装依赖：

```bash
cd strix-0.4.0

# 方式 A：使用 Poetry（推荐）
poetry install

# 方式 B：使用 pip（如果系统 Python 已安装）
pip install litellm[proxy] openai pydantic rich fastapi uvicorn tenacity numpydoc ipython openhands-aci playwright docker gql textual xmltodict pyte requests libtmux

# 方式 C：创建虚拟环境
python -m venv venv
# Windows
venv\Scripts\activate
# Linux/Mac
source venv/bin/activate
pip install litellm[proxy] openai pydantic rich fastapi uvicorn tenacity numpydoc ipython openhands-aci playwright docker gql textual xmltodict pyte requests libtmux
```

#### 方案 2：打包虚拟环境（适用于本地 venv）

如果使用本地 `venv`，可以：
1. 在源机器上创建虚拟环境并安装依赖
2. 将整个 `strix-0.4.0` 目录（包括 `venv`）打包
3. 在目标机器上解压

**注意**：
- 虚拟环境可能包含绝对路径，需要修改
- 跨平台（Windows/Linux/Mac）不兼容
- 虚拟环境体积较大（几百 MB）

#### 方案 3：使用 Docker（最可靠）

将 Strix 和依赖打包到 Docker 镜像中：
1. 使用 Strix 提供的 Dockerfile
2. 构建 Docker 镜像
3. 在目标机器上运行 Docker 容器

**优点**：
- 完全隔离
- 跨平台兼容
- 依赖已包含

**缺点**：
- 需要 Docker
- 镜像体积较大

## 🎯 当前应用中的使用方式

### LLM 连接测试

使用**系统 Python**（通过 subprocess），因此需要：
- 系统 Python 中已安装 `litellm` 等依赖
- 或使用 Poetry 虚拟环境中的 Python

### 实际扫描

使用**嵌入的 Python**（通过 pyo3），因此需要：
- 系统 Python 中已安装 Strix 的所有依赖
- 或配置 Python 路径指向虚拟环境

## 💡 推荐做法

### 开发环境

```bash
cd strix-0.4.0
poetry install
```

### 生产部署

**选项 A：使用系统 Python**
```bash
# 在目标机器上
pip install litellm[proxy] openai pydantic rich fastapi uvicorn tenacity numpydoc ipython openhands-aci playwright docker gql textual xmltodict pyte requests libtmux
```

**选项 B：使用本地虚拟环境**
```bash
cd strix-0.4.0
python -m venv venv
# 激活虚拟环境
pip install litellm[proxy] ...
# 在应用中配置 Python 路径指向 venv
```

**选项 C：打包到应用（未来优化）**
- 将 Python 依赖打包到应用资源目录
- 首次运行时自动安装到应用数据目录
- 实现完全独立的部署

## 📝 检查依赖是否安装

```bash
# 检查全局安装
pip list | grep litellm

# 检查 Poetry 虚拟环境
cd strix-0.4.0
poetry run pip list | grep litellm

# 检查本地虚拟环境
venv\Scripts\python.exe -m pip list | grep litellm  # Windows
venv/bin/python -m pip list | grep litellm  # Linux/Mac
```

## ⚠️ 常见问题

### Q: 为什么拷贝 strix-0.4.0 后提示找不到 litellm？

A: 因为依赖没有一起拷贝。需要在目标机器上重新安装依赖。

### Q: 如何知道依赖安装在哪里？

A: 运行 `python -c "import litellm; print(litellm.__file__)"` 查看模块路径。

### Q: 可以使用相对路径的虚拟环境吗？

A: 可以，使用 `python -m venv venv` 在项目目录中创建虚拟环境，但需要注意路径问题。

