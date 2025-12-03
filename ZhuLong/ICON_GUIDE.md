# 烛龙应用图标生成指南

## 📋 概述

本项目已包含完整的图标文件，位于 `src-tauri/icons/` 目录。所有图标格式已通过 Tauri CLI 自动生成。

## 🎨 图标设计

图标采用"烛龙"主题设计：
- **背景**：深蓝黑色渐变，体现神秘感
- **主体**：红色龙形图案，象征力量与洞察
- **亮点**：金色火焰效果，代表"烛"的意象
- **文字**：底部显示"烛龙"和"ZhuLong"

## 📁 图标文件

### 必需文件（已生成）

- `icon.ico` - Windows 图标
- `icon.icns` - macOS 图标
- `icon.png` - 通用 PNG 图标
- `32x32.png` - 小尺寸图标
- `128x128.png` - 标准尺寸图标
- `128x128@2x.png` - 高分辨率图标

### 源文件

- `icon.svg` - SVG 矢量源文件（1024x1024）

## 🔄 重新生成图标

如果需要重新生成图标（例如修改了 SVG 设计），请按以下步骤操作：

### 方法 1：使用 Tauri CLI（推荐）

1. **将 SVG 转换为 PNG**（1024x1024）：
   - 使用在线工具：https://convertio.co/svg-png/
   - 或使用 ImageMagick：`magick convert icon.svg -resize 1024x1024 icon-1024.png`
   - 或使用 Inkscape：`inkscape icon.svg --export-filename=icon-1024.png --export-width=1024 --export-height=1024`

2. **生成所有格式**：
   ```bash
   cd ZhuLong
   npx @tauri-apps/cli icon src-tauri/icons/icon-1024.png
   ```

   或者如果已有 1024x1024 的 PNG：
   ```bash
   npx @tauri-apps/cli icon src-tauri/icons/icon.png
   ```

### 方法 2：使用 Python 脚本

1. **安装依赖**：
   ```bash
   pip install cairosvg pillow
   ```

2. **运行转换脚本**：
   ```bash
   python scripts/create-icon-png.py
   ```

3. **生成图标**：
   ```bash
   npx @tauri-apps/cli icon src-tauri/icons/icon-1024.png
   ```

### 方法 3：使用 PowerShell 脚本

```powershell
.\scripts\generate-icons.ps1
```

## ✏️ 自定义图标

### 修改 SVG 设计

1. 编辑 `src-tauri/icons/icon.svg`
2. 使用上述方法重新生成所有格式

### 使用自己的图标

1. 准备 1024x1024 的 PNG 图标文件
2. 放置到 `src-tauri/icons/` 目录
3. 运行：`npx @tauri-apps/cli icon src-tauri/icons/your-icon.png`

## ✅ 验证图标

生成后，检查以下文件是否存在：

- ✅ `src-tauri/icons/icon.ico` (Windows)
- ✅ `src-tauri/icons/icon.icns` (macOS)
- ✅ `src-tauri/icons/icon.png`
- ✅ `src-tauri/icons/32x32.png`
- ✅ `src-tauri/icons/128x128.png`
- ✅ `src-tauri/icons/128x128@2x.png`

## 📝 配置说明

图标配置在 `src-tauri/tauri.conf.json` 中：

```json
{
  "bundle": {
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

## 🎯 图标设计建议

- **尺寸**：源文件建议 1024x1024 或更大
- **格式**：使用 PNG 或 SVG
- **背景**：建议使用透明或纯色背景
- **细节**：确保在小尺寸（32x32）下仍然清晰可辨
- **颜色**：使用高对比度颜色，确保在各种背景下都清晰

## 🐛 常见问题

### 图标显示不正确

1. 检查 `tauri.conf.json` 中的路径是否正确
2. 确保所有必需的图标文件都存在
3. 重新构建应用：`npm run tauri:build`

### 图标模糊

1. 使用更高分辨率的源文件（至少 1024x1024）
2. 确保 SVG 转换为 PNG 时使用高质量设置

### 图标未更新

1. 清除构建缓存：删除 `src-tauri/target/` 目录
2. 重新生成图标
3. 重新构建应用

