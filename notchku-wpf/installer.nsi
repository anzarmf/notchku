; NotchKu 0.2.0 — NSIS installer (WPF, .NET Framework 4.8, no WebView, no extra runtime)
!include "MUI2.nsh"

Name "NotchKu"
OutFile "..\NotchKu_0.2.0_x64-setup.exe"
InstallDir "$PROGRAMFILES64\NotchKu"
RequestExecutionLevel admin
Unicode true

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetOutPath "$INSTDIR"
  File "bin\Release\net48\NotchKu.exe"
  File "bin\Release\net48\Newtonsoft.Json.dll"
  CreateShortcut "$DESKTOP\NotchKu.lnk" "$INSTDIR\NotchKu.exe"
  CreateDirectory "$SMPROGRAMS\NotchKu"
  CreateShortcut "$SMPROGRAMS\NotchKu\NotchKu.lnk" "$INSTDIR\NotchKu.exe"
  CreateShortcut "$SMPROGRAMS\NotchKu\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotchKu" "DisplayName" "NotchKu"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotchKu" "DisplayVersion" "0.2.0"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotchKu" "Publisher" "Zarski"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotchKu" "UninstallString" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\NotchKu.exe"
  Delete "$INSTDIR\Newtonsoft.Json.dll"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$DESKTOP\NotchKu.lnk"
  Delete "$SMPROGRAMS\NotchKu\NotchKu.lnk"
  Delete "$SMPROGRAMS\NotchKu\Uninstall.lnk"
  RMDir "$SMPROGRAMS\NotchKu"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotchKu"
SectionEnd
