@echo off
REM Debug script to run FeedFlow with console output visible
REM Usage: scripts\run-debug.bat

cd /d "%~dp0\.."
cd dist-portable

if not exist "feedflow.exe" (
    echo Error: feedflow.exe not found in dist-portable directory
    echo Please build the application first:
    echo   npm run tauri:build:portable:ps1
    pause
    exit /b 1
)

echo ========================================
echo FeedFlow Debug Mode
echo ========================================
echo.
echo Running: feedflow.exe
echo Console output will be displayed below:
echo Press Ctrl+C to stop the application
echo.
echo ----------------------------------------
echo.

feedflow.exe

echo.
echo ----------------------------------------
echo Application exited.
pause

