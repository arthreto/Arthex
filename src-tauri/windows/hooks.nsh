; Removes what Arthex added to the right-click menu, while its program is still there.
!macro NSIS_HOOK_PREUNINSTALL
  ExecWait '"$INSTDIR\arthex.exe" --clear-context-menu'
!macroend

; Removes the startup entry written by tauri-plugin-autostart when Arthex is uninstalled.
!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Arthex"
!macroend
