# 便携版构建脚本

这些脚本用于自动构建 FeedFlow 的便携版（免安装版本）。

## 使用方法

### 方法 1: 使用 Node.js 脚本（推荐）

首先安装依赖：
```bash
npm install
```

然后运行：
```bash
npm run tauri:build:portable
```

### 方法 2: 使用 PowerShell 脚本（Windows）

直接运行 PowerShell 脚本（无需额外依赖）：
```bash
npm run tauri:build:portable:ps1
```

或者直接运行：
```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-portable.ps1
```

## 输出

构建完成后，会在项目根目录生成：
- `dist-portable/` - 便携版文件夹（包含所有必需文件）
- `FeedFlow-1.0.0-portable.zip` - 打包好的便携版 ZIP 文件

## 便携版内容

便携版包含：
- `feedflow.exe` - 主程序
- 所有必需的 DLL 文件
- 资源文件（如果有）
- `README.txt` - 使用说明

## 注意事项

1. 构建前请确保已运行 `npm install` 安装所有依赖
2. 首次构建可能需要较长时间（需要编译 Rust 代码）
3. 便携版可以直接解压到任意目录运行，无需安装
4. 数据文件会保存在 Windows 应用数据目录

