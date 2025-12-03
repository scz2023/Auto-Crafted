# Debug script to run FeedFlow with console output visible
# Usage: .\scripts\run-debug.ps1

$ErrorActionPreference = "Continue"

$rootDir = Split-Path -Parent $PSScriptRoot
$portableDir = Join-Path $rootDir "dist-portable"
$exePath = Join-Path $portableDir "feedflow.exe"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "FeedFlow Debug Mode" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path $exePath)) {
    Write-Host "Error: Executable not found: $exePath" -ForegroundColor Red
    Write-Host "Please build the application first:" -ForegroundColor Yellow
    Write-Host "  npm run tauri:build:portable:ps1" -ForegroundColor Yellow
    exit 1
}

Write-Host "Running: $exePath" -ForegroundColor Green
Write-Host "Console output will be displayed below:" -ForegroundColor Green
Write-Host "Press Ctrl+C to stop the application" -ForegroundColor Yellow
Write-Host ""
Write-Host "----------------------------------------" -ForegroundColor Gray
Write-Host ""

# Run the executable
Set-Location $portableDir
& $exePath

Write-Host ""
Write-Host "----------------------------------------" -ForegroundColor Gray
Write-Host "Application exited." -ForegroundColor Cyan

