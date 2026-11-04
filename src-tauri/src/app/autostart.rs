use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Passed by the startup entry, so a launch at login stays silent.
pub const LAUNCH_ARG: &str = "--autostart";

pub fn launched_at_startup() -> bool {
    std::env::args().any(|arg| arg == LAUNCH_ARG)
}

/// Keeps the system startup entry in line with the setting. Debug builds never register one,
/// so a development binary does not end up launching at login.
pub fn apply(app: &AppHandle, enabled: bool) {
    if cfg!(debug_assertions) {
        return;
    }
    let launcher = app.autolaunch();
    let result = match (enabled, launcher.is_enabled()) {
        (true, Ok(false)) => launcher.enable(),
        (false, Ok(true)) => launcher.disable(),
        (_, Err(error)) => Err(error),
        _ => Ok(()),
    };
    if let Err(error) = result {
        tracing::warn!(%error, enabled, "could not update the startup entry");
    }
}
