!include "${NSISDIR}\Include\MUI2.nsh"
!include "custom.nsh"

; 继承默认的 Tauri NSIS 脚本
!include "${TAURI_DIR}\nsis\common.nsh"

; 自定义安装页面
!define MUI_PAGE_CUSTOMFUNCTION_PRE WelcomePagePre
!define MUI_PAGE_CUSTOMFUNCTION_LEAVE WelcomePageLeave

; 安装过程页面
!define MUI_PAGE_CUSTOMFUNCTION_PRE InstallPagePre
!define MUI_PAGE_CUSTOMFUNCTION_LEAVE InstallPageLeave

; 完成页面
!define MUI_PAGE_CUSTOMFUNCTION_PRE FinishPagePre

; 包含默认页面
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${TAURI_DIR}\nsis\LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"

; 自定义函数
Function WelcomePagePre
  ; 欢迎页面加载前执行
  Call CustomWelcomePagePre
FunctionEnd

Function WelcomePageLeave
  ; 欢迎页面离开时执行（点击下一步）
  Call CustomWelcomePageLeave
FunctionEnd

Function InstallPagePre
  ; 安装页面加载前执行
  Call CustomInstallPagePre
FunctionEnd

Function InstallPageLeave
  ; 安装页面离开时执行（安装完成后点击下一步）
  Call CustomInstallPageLeave
FunctionEnd

Function FinishPagePre
  ; 完成页面加载前执行
  Call CustomFinishPagePre
FunctionEnd

; 安装完成后执行
Function .onInstSuccess
  Call CustomOnInstSuccess
FunctionEnd

; 安装失败时执行
Function .onInstFailed
  Call CustomOnInstFailed
FunctionEnd
