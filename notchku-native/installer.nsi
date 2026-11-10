; NotchKu installer (NSIS). No WebView, no extra runtime to install.
; Built on GitHub Actions (windows-latest, makensis preinstalled).
; Expects NotchKu.exe next to this script (cargo build --release output).

!include "MUI2.nsh"

Name "NotchKu"
OutFile "NotchKu_0.2.0_x64-setup.exe"
InstallDir "$PROGRAMFILES64\NotchKu"
RequestExecutionLevel admin
SetCompressor /SOLID lzma

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Indonesian"

Section "Install"
  SetOutPath $INSTDIR
  File "NotchKu.exe"
  CreateShortcut "$DESKTOP\NotchKu.lnk" "$INSTDIR\NotchKu.exe"
  CreateDirectory "$SMPROGRAMS\NotchKu"
  CreateShortcut "$SMPROGRAMS\NotchKu\NotchKu.lnk" "$INSTDIR\NotchKu.exe"
  CreateShortcut "$SMPROGRAMS\NotchKu\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotchKu" "DisplayName" "NotchKu"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotchKu" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotchKu" "DisplayVersion" "0.2.0"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\NotchKu.exe"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$DESKTOP\NotchKu.lnk"
  Delete "$SMPROGRAMS\NotchKu\NotchKu.lnk"
  Delete "$SMPROGRAMS\NotchKu\Uninstall.lnk"
  RMDir "$SMPROGRAMS\NotchKu"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotchKu"
SectionEnd
