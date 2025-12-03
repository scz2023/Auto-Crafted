# 烛龙图标生成脚本
# 使用 Tauri CLI 从 SVG 生成所有需要的图标格式

Write-Host "🕯️ 开始生成烛龙应用图标..." -ForegroundColor Cyan

# 检查 Tauri CLI 是否安装
$tauriCli = Get-Command tauri -ErrorAction SilentlyContinue
if (-not $tauriCli) {
    Write-Host "❌ Tauri CLI 未找到，请先安装: npm install -g @tauri-apps/cli" -ForegroundColor Red
    exit 1
}

# 检查 SVG 源文件
$svgPath = "src-tauri\icons\icon.svg"
if (-not (Test-Path $svgPath)) {
    Write-Host "❌ SVG 源文件不存在: $svgPath" -ForegroundColor Red
    exit 1
}

# 临时创建 PNG（如果 SVG 转换工具不可用，使用在线工具或手动转换）
# 这里我们直接使用 Tauri 的 icon 命令，它支持 PNG 输入

# 首先需要将 SVG 转换为 PNG（1024x1024）
# 可以使用 Inkscape、ImageMagick 或在线工具
Write-Host "📝 提示：需要先将 SVG 转换为 1024x1024 的 PNG 文件" -ForegroundColor Yellow
Write-Host "   可以使用以下方法之一：" -ForegroundColor Yellow
Write-Host "   1. 在线工具：https://convertio.co/svg-png/" -ForegroundColor Yellow
Write-Host "   2. ImageMagick: magick convert icon.svg -resize 1024x1024 icon.png" -ForegroundColor Yellow
Write-Host "   3. Inkscape: inkscape icon.svg --export-filename=icon.png --export-width=1024 --export-height=1024" -ForegroundColor Yellow

# 检查是否有 PNG 文件
$pngPath = "src-tauri\icons\icon-1024.png"
if (Test-Path $pngPath) {
    Write-Host "✅ 找到 PNG 源文件，开始生成图标..." -ForegroundColor Green
    
    # 使用 Tauri CLI 生成图标
    Set-Location $PSScriptRoot\..
    npm run tauri icon $pngPath
    
    Write-Host "✅ 图标生成完成！" -ForegroundColor Green
} else {
    Write-Host "⚠️  未找到 PNG 源文件 ($pngPath)" -ForegroundColor Yellow
    Write-Host "   请先创建 1024x1024 的 PNG 文件，然后重新运行此脚本" -ForegroundColor Yellow
}

