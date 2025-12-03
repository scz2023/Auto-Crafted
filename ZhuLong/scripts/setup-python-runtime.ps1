# ZhuLong - Python Runtime Setup Script
# This script downloads and packages Python interpreter for the application

param(
    [string]$PythonVersion = "3.13.0",
    [string]$Platform = "windows-x64"
)

# Set output encoding to UTF-8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
chcp 65001 | Out-Null

Write-Host "ZhuLong - Python Runtime Setup Tool" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan

$ErrorActionPreference = "Stop"

# 配置
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$RuntimeDir = Join-Path $ProjectRoot "python-runtime"
$PythonDir = Join-Path $RuntimeDir "python"

# 检查平台
if ($Platform -eq "windows-x64") {
    $PythonUrl = "https://www.python.org/ftp/python/$PythonVersion/python-$PythonVersion-embed-amd64.zip"
    $PythonExe = "python.exe"
} elseif ($Platform -eq "windows-x86") {
    $PythonUrl = "https://www.python.org/ftp/python/$PythonVersion/python-$PythonVersion-embed-win32.zip"
    $PythonExe = "python.exe"
} else {
    Write-Host "[ERROR] Unsupported platform: $Platform" -ForegroundColor Red
    Write-Host "   Supported platforms: windows-x64, windows-x86" -ForegroundColor Yellow
    exit 1
}

# 创建目录
Write-Host "`n[INFO] Creating directory structure..." -ForegroundColor Yellow
if (Test-Path $RuntimeDir) {
    Write-Host "   Directory exists, cleaning: $RuntimeDir" -ForegroundColor Yellow
    Remove-Item -Path $RuntimeDir -Recurse -Force
}
New-Item -ItemType Directory -Path $PythonDir -Force | Out-Null

# 下载 Python
$ZipFile = Join-Path $RuntimeDir "python.zip"
Write-Host "`n[INFO] Downloading Python $PythonVersion ($Platform)..." -ForegroundColor Yellow
Write-Host "   Source: $PythonUrl" -ForegroundColor Gray

try {
    Invoke-WebRequest -Uri $PythonUrl -OutFile $ZipFile -UseBasicParsing
    Write-Host "   [OK] Download completed" -ForegroundColor Green
} catch {
    Write-Host "   [ERROR] Download failed: $_" -ForegroundColor Red
    exit 1
}

# 解压
Write-Host "`n[INFO] Extracting Python..." -ForegroundColor Yellow
try {
    Expand-Archive -Path $ZipFile -DestinationPath $PythonDir -Force
    Write-Host "   [OK] Extraction completed" -ForegroundColor Green
} catch {
    Write-Host "   [ERROR] Extraction failed: $_" -ForegroundColor Red
    exit 1
}

# 清理
Remove-Item -Path $ZipFile -Force

# 配置 Python
Write-Host "`n[INFO] Configuring Python..." -ForegroundColor Yellow

# 1. 重命名 python313._pth 为 python313.pth（如果存在）
$PthFiles = Get-ChildItem -Path $PythonDir -Filter "python*.pth" -ErrorAction SilentlyContinue
foreach ($PthFile in $PthFiles) {
    if ($PthFile.Name -match "\._pth$") {
        $NewName = $PthFile.Name -replace "\._pth$", ".pth"
        $NewPath = Join-Path $PythonDir $NewName
        if (-not (Test-Path $NewPath)) {
            Copy-Item -Path $PthFile.FullName -Destination $NewPath
            Write-Host "   [OK] Created $NewName" -ForegroundColor Green
        }
    }
}

# 2. 修改 .pth 文件以启用标准库
$PthFile = Get-ChildItem -Path $PythonDir -Filter "python*.pth" | Select-Object -First 1
if ($PthFile) {
    $PthContent = Get-Content -Path $PthFile.FullName -Raw
    # 取消注释 import site 行（如果存在）
    $PthContent = $PthContent -replace "#import site", "import site"
    Set-Content -Path $PthFile.FullName -Value $PthContent -NoNewline
    Write-Host "   [OK] Configured .pth file" -ForegroundColor Green
}

# 3. 创建 get-pip.py（用于安装 pip）
$GetPipUrl = "https://bootstrap.pypa.io/get-pip.py"
$GetPipPath = Join-Path $PythonDir "get-pip.py"
Write-Host "`n[INFO] Downloading get-pip.py..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $GetPipUrl -OutFile $GetPipPath -UseBasicParsing
    Write-Host "   [OK] Download completed" -ForegroundColor Green
} catch {
    Write-Host "   [WARN] Download get-pip.py failed (optional): $_" -ForegroundColor Yellow
}

# 创建说明文件
$ReadmePath = Join-Path $RuntimeDir "README.md"
$Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

$ReadmeContent = @'
# Python Runtime

This directory contains the bundled Python {0} interpreter.

## Directory Structure

python/ - Python interpreter files
  python.exe - Python executable
  python*.dll - Python dynamic link libraries
  python*.pth - Python path configuration files
  get-pip.py - pip installation script (optional)

## Install pip (Optional)

If you need to use pip to install packages, you can run:

python-runtime/python/python.exe python-runtime/python/get-pip.py

## Notes

1. This is an embedded Python, does not include the full standard library
2. Some packages that require compilation may not be directly installable
3. It is recommended to use pre-compiled wheel packages

## Version Information

Python Version: {0}
Platform: {1}
Packaged Time: {2}
'@ -f $PythonVersion, $Platform, $Timestamp

Set-Content -Path $ReadmePath -Value $ReadmeContent -Encoding UTF8

Write-Host "`n[OK] Python runtime setup completed!" -ForegroundColor Green
Write-Host "`n[INFO] Next steps:" -ForegroundColor Cyan
Write-Host "   1. Check if python-runtime/python/python.exe exists" -ForegroundColor Yellow
Write-Host "   2. Run the app, Python will automatically use the bundled interpreter" -ForegroundColor Yellow
$pipCmd = "python-runtime\python\python.exe python-runtime\python\get-pip.py"
Write-Host "   3. To install pip, run: $pipCmd" -ForegroundColor Yellow

