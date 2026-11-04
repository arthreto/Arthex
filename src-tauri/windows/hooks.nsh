; Removes the startup entry written by tauri-plugin-autostart when Arthex is uninstalled.
!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Arthex"
!macroend
