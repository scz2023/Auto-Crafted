# Python 运行时打包指南

## 📋 概述

本指南说明如何为"烛龙"应用打包 Python 解释器，使应用可以完全独立运行，无需系统安装 Python。

## 🎯 目标

- 将 Python 解释器打包到应用中
- 应用可以独立运行，不依赖系统 Python
- 支持 Windows、Linux 和 macOS

## 🚀 快速开始

### Windows

1. **运行打包脚本**：
   ```powershell
   cd ZhuLong
   .\scripts\setup-python-runtime.ps1
   ```

   或指定版本和平台：
   ```powershell
   .\scripts\setup-python-runtime.ps1 -PythonVersion "3.13.0" -Platform "windows-x64"
   ```

2. **验证安装**：
   ```powershell
   .\python-runtime\python\python.exe --version
   ```

### Linux/macOS

1. **运行打包脚本**：
   ```bash
   cd ZhuLong
   chmod +x scripts/setup-python-runtime.sh
   ./scripts/setup-python-runtime.sh 3.13.0 linux-x64
   ```

2. **验证安装**：
   ```bash
   ./python-runtime/python/python3 --version
   ```

## 📁 目录结构

打包后的目录结构：

```
python-runtime/
├── python/
│   ├── python.exe (Windows) 或 python3 (Linux/macOS)
│   ├── python*.dll (Windows)
│   ├── python*.pth
│   ├── get-pip.py (可选)
│   └── ... (其他 Python 文件)
└── README.md
```

## ⚙️ 配置应用

### 1. 更新 Tauri 配置

确保 `src-tauri/tauri.conf.json` 中包含 `python-runtime` 目录：

```json
{
  "bundle": {
    "resources": [
      "../python-runtime/**/*",
      "../strix-0.4.0/**/*"
    ]
  }
}
```

### 2. 修改代码使用打包的 Python

应用会自动检测并使用打包的 Python。代码逻辑：

1. **优先使用打包的 Python**：检查 `python-runtime/python/python.exe` 是否存在
2. **回退到系统 Python**：如果打包的 Python 不存在，使用系统 Python
3. **回退到 pyo3**：如果系统 Python 也不可用，使用 pyo3 嵌入的 Python

## 🔧 手动打包（高级）

### Windows - 使用嵌入版 Python

1. **下载嵌入版 Python**：
   - 访问：https://www.python.org/downloads/windows/
   - 下载 "Windows embeddable package (64-bit)"
   - 解压到 `python-runtime/python/` 目录

2. **配置 Python**：
   - 重命名 `python313._pth` 为 `python313.pth`
   - 编辑 `.pth` 文件，取消注释 `import site` 行

3. **安装 pip（可选）**：
   ```powershell
   .\python-runtime\python\python.exe -m ensurepip --upgrade
   ```

### Linux - 编译 Python

1. **下载 Python 源码**：
   ```bash
   wget https://www.python.org/ftp/python/3.13.0/Python-3.13.0.tgz
   tar -xzf Python-3.13.0.tgz
   ```

2. **编译并安装到指定目录**：
   ```bash
   cd Python-3.13.0
   ./configure --prefix=$(pwd)/../python-runtime/python
   make -j$(nproc)
   make install
   ```

### macOS - 使用 Homebrew

1. **安装 Python**：
   ```bash
   brew install python@3.13
   ```

2. **复制到运行时目录**：
   ```bash
   mkdir -p python-runtime/python
   cp -r /opt/homebrew/opt/python@3.13/* python-runtime/python/
   ```

## 📦 打包到应用

打包 Python 运行时后，在构建应用时会自动包含：

```bash
npm run tauri:build
```

Python 运行时会被打包到应用的资源目录中。

## 🔍 验证

### 检查 Python 路径

应用启动后，检查日志确认使用的 Python 路径：

- 如果看到 "使用打包的 Python"，说明成功使用打包的 Python
- 如果看到 "使用系统 Python"，说明回退到系统 Python

### 测试 Python 功能

在应用中启动一个扫描任务，检查是否能正常执行 Python 代码。

## ⚠️ 注意事项

### 1. 文件大小

- Windows 嵌入版：约 10-15 MB
- Linux 完整版：约 50-100 MB
- macOS 完整版：约 50-100 MB

### 2. 许可证

Python 使用 PSF 许可证，可以自由分发。确保在应用中包含 Python 许可证信息。

### 3. 依赖管理

- 打包的 Python 可能不包含所有标准库
- 某些需要编译的包可能无法安装
- 建议使用预编译的 wheel 包

### 4. 平台兼容性

- Windows：使用嵌入版 Python，体积小但功能有限
- Linux：需要编译或使用系统 Python
- macOS：可以使用 Homebrew 版本或编译

## 🐛 故障排除

### Python 未找到

1. 检查 `python-runtime/python/` 目录是否存在
2. 检查可执行文件名称是否正确（Windows: `python.exe`, Linux/macOS: `python3`）
3. 检查文件权限（Linux/macOS）

### 导入错误

1. 检查 `.pth` 文件配置
2. 确保 `import site` 已启用
3. 检查 Python 路径配置

### 性能问题

1. 打包的 Python 可能比系统 Python 慢
2. 考虑使用系统 Python 作为回退方案
3. 优化 Python 代码执行逻辑

## 📚 相关资源

- [Python 嵌入版文档](https://docs.python.org/3/using/windows.html#embedded-distribution)
- [Tauri 资源打包](https://tauri.app/v1/guides/building/resources)
- [pyo3 文档](https://pyo3.rs/)

## 🔄 更新 Python 版本

要更新 Python 版本：

1. 删除旧的 `python-runtime` 目录
2. 运行打包脚本，指定新版本
3. 重新构建应用

```powershell
Remove-Item -Recurse -Force python-runtime
.\scripts\setup-python-runtime.ps1 -PythonVersion "3.13.1"
npm run tauri:build
```

