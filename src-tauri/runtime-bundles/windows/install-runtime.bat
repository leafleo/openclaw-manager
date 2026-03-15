@echo off

setlocal

echo ====================================
echo Installing OpenClaw Runtime (Windows)
echo ====================================
echo.

rem 获取脚本所在目录
set "SCRIPT_DIR=%~dp0"
set "ROOT_DIR=%SCRIPT_DIR%...."

rem 解压 Node.js
set "NODE_ZIP=%SCRIPT_DIR%node
ode-v18.20.4-win-x64.zip"
set "NODE_DIR=%ROOT_DIR%untime
ode"

echo Extracting Node.js...
if not exist "%NODE_DIR%" mkdir "%NODE_DIR%"
powershell -Command "Expand-Archive -Path '%NODE_ZIP%' -DestinationPath '%NODE_DIR%' -Force"

rem 解压 Git
set "GIT_EXE=%SCRIPT_DIR%gitPortableGit-2.43.0-64-bit.7z.exe"
set "GIT_DIR=%ROOT_DIR%untimegit"

echo Extracting Git...
if not exist "%GIT_DIR%" mkdir "%GIT_DIR%"
"%GIT_EXE%" -y -o"%GIT_DIR%"

rem 配置环境变量
set "NODE_BIN=%NODE_DIR%
ode-v18.20.4-win-x64in"
set "GIT_BIN=%GIT_DIR%in"
set "PATH=%NODE_BIN%;%GIT_BIN%;%PATH%"

echo Configuring environment variables...

rem 保存环境变量到系统
setx NODE_HOME "%NODE_DIR%
ode-v18.20.4-win-x64" /M
setx GIT_HOME "%GIT_DIR%" /M
setx PATH "%NODE_BIN%;%GIT_BIN%;%PATH%" /M

echo.
echo ====================================
echo Installation complete!
echo ====================================
echo.
echo Node.js: %NODE_BIN%
ode.exe
echo Git: %GIT_BIN%git.exe
echo.
echo You can now run: node --version
echo You can now run: git --version
echo.

endlocal
