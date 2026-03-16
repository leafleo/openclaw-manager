!include "${NSISDIR}\Include\MUI2.nsh"

SetCompressor lzma
SetCompressorDictSize 64

; 包含默认页面
; 欢迎页面
!define MUI_PAGE_CUSTOMFUNCTION_PRE WelcomePagePre
!define MUI_PAGE_CUSTOMFUNCTION_LEAVE WelcomePageLeave
!insertmacro MUI_PAGE_WELCOME

; 许可证页面
!insertmacro MUI_PAGE_LICENSE "LICENSE.txt"

; 目录选择页面
!insertmacro MUI_PAGE_DIRECTORY

; 安装页面
!define MUI_PAGE_CUSTOMFUNCTION_PRE InstallPagePre
!define MUI_PAGE_CUSTOMFUNCTION_LEAVE InstallPageLeave
!insertmacro MUI_PAGE_INSTFILES

; 完成页面
!define MUI_PAGE_CUSTOMFUNCTION_PRE FinishPagePre
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"

; 自定义函数
Function WelcomePagePre
  ; 欢迎页面加载前执行
  DetailPrint "Welcome page pre-function executed"
FunctionEnd

Function WelcomePageLeave
  ; 欢迎页面离开时执行（点击下一步）
  DetailPrint "Welcome page leave function executed - Next button clicked"
  
  ; 示例：检查系统环境
  DetailPrint "Checking system environment..."
  
  ; 示例：显示自定义消息
  MessageBox MB_OK "欢迎使用 OpenClaw Manager 安装向导！\n\n点击确定继续安装过程。"
FunctionEnd

Function InstallPagePre
  ; 安装页面加载前执行
  DetailPrint "Install page pre-function executed"
  
  ; 示例：准备安装资源
  DetailPrint "Preparing installation resources..."
FunctionEnd

Function InstallPageLeave
  ; 安装页面离开时执行（安装完成后点击下一步）
  DetailPrint "Install page leave function executed - Next button clicked after installation"
  
  ; 示例：配置环境变量
  DetailPrint "Configuring environment variables..."
  
  ; 示例：注册服务
  DetailPrint "Registering services..."
FunctionEnd

Function FinishPagePre
  ; 完成页面加载前执行
  DetailPrint "Finish page pre-function executed"
  
  ; 示例：准备完成页面内容
  DetailPrint "Preparing finish page..."
FunctionEnd

; 安装完成后执行
Function .onInstSuccess
  ; 安装成功后的逻辑
  DetailPrint "Installation successful!"
  
  ; 示例：创建快捷方式
  DetailPrint "Creating shortcuts..."
  
  ; 示例：启动应用程序
  DetailPrint "Preparing to launch application..."
FunctionEnd

; 安装失败时执行
Function .onInstFailed
  ; 安装失败时的逻辑
  DetailPrint "Installation failed!"
  
  ; 示例：显示错误信息
  MessageBox MB_OK "安装过程中出现错误，请重试或联系技术支持。"
FunctionEnd

; 自定义函数：配置环境变量
Function ConfigureEnvironmentVariables
  ; 在这里添加配置环境变量的逻辑
  DetailPrint "Configuring environment variables..."
  
  ; 示例：添加 Node.js 到 PATH
  ; 注意：实际实现需要根据安装路径动态调整
  ; StrCpy $0 "$INSTDIR\bundles\windows\node"
  ; nsExec::ExecToStack "setx PATH "$PATH;$0" /M"
FunctionEnd

; 自定义函数：安装 OpenClaw
Function InstallOpenClaw
  ; 在这里添加安装 OpenClaw 的逻辑
  DetailPrint "Installing OpenClaw..."
  
  ; 示例：执行安装脚本
  ; StrCpy $0 "$INSTDIR\bundles\common\openclaw\install-openclaw.bat"
  ; nsExec::ExecToStack "$0"
FunctionEnd
