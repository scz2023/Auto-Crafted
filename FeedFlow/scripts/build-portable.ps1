# FeedFlow 便携版构建脚本
$ErrorActionPreference = "Stop"

$rootDir = Split-Path -Parent $PSScriptRoot
$tauriDir = Join-Path $rootDir "src-tauri"
$releaseDir = Join-Path $tauriDir "target\release"
$portableDir = Join-Path $rootDir "dist-portable"
$productName = "FeedFlow"
$version = "1.0.0"
$exeName = "feedflow.exe"

Write-Host "Starting portable build...`n" -ForegroundColor Cyan

# 1. Clean old portable directory
if (Test-Path $portableDir) {
    Write-Host "Cleaning old portable directory..." -ForegroundColor Yellow
    Remove-Item -Path $portableDir -Recurse -Force
}

# 2. Build frontend
Write-Host "Building frontend..." -ForegroundColor Cyan
Set-Location $rootDir
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Frontend build failed!" -ForegroundColor Red
    exit 1
}

# 3. Build Tauri application
Write-Host "Building Tauri application (Release mode)..." -ForegroundColor Cyan
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Warning: Tauri build had errors, but checking if release version was generated..." -ForegroundColor Yellow
}

# 4. Check release directory
if (-not (Test-Path $releaseDir)) {
    Write-Host "Error: Release directory does not exist: $releaseDir" -ForegroundColor Red
    exit 1
}

# 5. Create portable directory
Write-Host "Creating portable directory..." -ForegroundColor Cyan
New-Item -ItemType Directory -Path $portableDir -Force | Out-Null

# 6. Copy executable file
$exePath = Join-Path $releaseDir $exeName
if (Test-Path $exePath) {
    Write-Host "Copying executable: $exeName" -ForegroundColor Green
    Copy-Item -Path $exePath -Destination (Join-Path $portableDir $exeName)
} else {
    Write-Host "Error: Executable not found: $exePath" -ForegroundColor Red
    exit 1
}

# 7. Copy DLL files and other dependencies
Write-Host "Copying dependencies..." -ForegroundColor Cyan
Get-ChildItem -Path $releaseDir -File | Where-Object {
    $_.Extension -in @('.dll', '.pdb') -or $_.Name -eq 'WebView2Loader.dll'
} | ForEach-Object {
    Copy-Item -Path $_.FullName -Destination (Join-Path $portableDir $_.Name)
    Write-Host "  [OK] $($_.Name)" -ForegroundColor Gray
}

# 7.5. Copy server files (.output/server directory)
$outputServerDir = Join-Path $rootDir ".output\server"
$portableOutputDir = Join-Path $portableDir ".output"
$portableServerDir = Join-Path $portableOutputDir "server"
if (Test-Path $outputServerDir) {
    Write-Host "Copying server files..." -ForegroundColor Cyan
    Write-Host "  Source: $outputServerDir" -ForegroundColor Gray
    Write-Host "  Destination: $portableServerDir" -ForegroundColor Gray
    
    # 确保 .output 目录存在
    if (-not (Test-Path $portableOutputDir)) {
        New-Item -ItemType Directory -Path $portableOutputDir -Force | Out-Null
    }
    
    Copy-Item -Path $outputServerDir -Destination $portableServerDir -Recurse -Force
    Write-Host "  [OK] Server files copied" -ForegroundColor Green
    
    # 验证复制是否成功
    $serverIndex = Join-Path $portableServerDir "index.mjs"
    if (Test-Path $serverIndex) {
        Write-Host "  [OK] Server index.mjs verified" -ForegroundColor Green
    } else {
        Write-Host "  [ERROR] Server index.mjs not found after copy!" -ForegroundColor Red
    }
} else {
    Write-Host "ERROR: Server directory does not exist: $outputServerDir" -ForegroundColor Red
    Write-Host "Please run 'npm run build' first to generate the server files." -ForegroundColor Yellow
    exit 1
}

# 8. Check and copy resource files (if any)
$resourcesDir = Join-Path $releaseDir "_up_"
if (Test-Path $resourcesDir) {
    Write-Host "Copying resource files..." -ForegroundColor Cyan
    Get-ChildItem -Path $resourcesDir -File | ForEach-Object {
        Copy-Item -Path $_.FullName -Destination (Join-Path $portableDir $_.Name)
        Write-Host "  [OK] $($_.Name)" -ForegroundColor Gray
    }
}

# 9. 创建 README 文件
$readmeLines = @(
    "# $productName Portable Version",
    "",
    "Version: $version",
    "",
    "## Usage Instructions",
    "",
    "1. Extract this file to any directory",
    "2. Run $exeName to use",
    "3. No installation required, can run directly",
    "",
    "## Notes",
    "",
    "- First run may take some time to load",
    "- Data files will be saved in the application data directory",
    "- You can delete the entire folder at any time without leaving traces in the system",
    "",
    "## System Requirements",
    "",
    "- Windows 10/11",
    "- WebView2 Runtime (usually pre-installed, will download automatically if not installed)",
    "- Node.js (required for server-side API)"
)
$readmeContent = $readmeLines -join "`r`n"

$readmePath = Join-Path $portableDir "README.txt"
$readmeContent | Out-File -FilePath $readmePath -Encoding UTF8
Write-Host "Created README.txt" -ForegroundColor Green

# 10. Package into ZIP
Write-Host "`nPackaging into ZIP file..." -ForegroundColor Cyan
$zipFileName = "$productName-$version-portable.zip"
$zipPath = Join-Path $rootDir $zipFileName

# 删除旧的 ZIP 文件（如果存在）
if (Test-Path $zipPath) {
    Remove-Item -Path $zipPath -Force
}

# 使用 .NET 压缩功能
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory($portableDir, $zipPath)

$zipSize = (Get-Item $zipPath).Length / 1MB
Write-Host "Portable version packaged successfully!" -ForegroundColor Green
Write-Host "File: $zipFileName" -ForegroundColor Cyan
Write-Host "Size: $([math]::Round($zipSize, 2)) MB" -ForegroundColor Cyan
Write-Host "Location: $zipPath" -ForegroundColor Cyan
Write-Host ""
Write-Host "Portable version is ready!" -ForegroundColor Green

