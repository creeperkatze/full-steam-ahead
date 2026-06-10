; Add $INSTDIR\resources to the current user's PATH so `fsa` is
; accessible from any terminal without a full path.
; LogicLib.nsh is already included by Tauri's NSIS template.
ReadRegStr $R0 HKCU "Environment" "Path"
${If} $R0 != ""
    WriteRegStr HKCU "Environment" "Path" "$R0;$INSTDIR\resources"
${Else}
    WriteRegStr HKCU "Environment" "Path" "$INSTDIR\resources"
${EndIf}
; Notify running processes (e.g. open terminals) of the PATH change.
SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
