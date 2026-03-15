@echo off
echo ====================================
echo Installing OpenClaw 2026.3.12
echo ====================================
echo.

REM 检查 Node.js 是否安装
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js is not installed. Please install Node.js first.
    pause
    exit /b 1
)

REM 检查 Git 是否安装
where git >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Git is not installed. Please install Git first.
    pause
    exit /b 1
)

REM 获取脚本所在目录
set "OPENCLAW_DIR=%~dp0"
set "TARBALL=%OPENCLAW_DIR%\openclaw-2026.3.12.tgz"

REM 本地安装 OpenClaw
echo Installing OpenClaw from local package...
cd /d "%OPENCLAW_DIR%"
call npm install "%TARBALL%" --save-exact

REM 启动 OpenClaw 网关
echo Starting OpenClaw Gateway...
call openclaw gateway start

REM 检查网关状态
echo Checking Gateway status...
call openclaw gateway status

if %errorlevel% neq 0 (
    echo ERROR: Installation failed!
    pause
    exit /b 1
)

echo.
echo ====================================
echo Installation complete!
echo ====================================
echo.
pause
