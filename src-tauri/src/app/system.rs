use std::fs;

use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use super::{AppError, Result};

pub fn open_data_dir(app: &AppHandle) -> Result<()> {
    let dir = app.path().app_data_dir()?;
    fs::create_dir_all(&dir)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Failed(error.to_string()))
}

pub fn open_log_dir(app: &AppHandle) -> Result<()> {
    let dir = app.path().app_log_dir()?;
    fs::create_dir_all(&dir)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Failed(error.to_string()))
}

/// Starts the uninstaller registered by the NSIS installer, then quits so it can remove the files.
#[cfg(windows)]
pub fn uninstall(app: &AppHandle) -> Result<()> {
    use std::{path::PathBuf, process::Command};
    use winreg::{
        RegKey,
        enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    };

    let key = format!(
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{}",
        app.package_info().name
    );
    let command: String = [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE]
        .into_iter()
        .find_map(|hive| {
            RegKey::predef(hive)
                .open_subkey(&key)
                .ok()?
                .get_value("UninstallString")
                .ok()
        })
        .ok_or(AppError::Unsupported)?;

    let uninstaller = PathBuf::from(command.trim().trim_matches('"'));
    let is_exe = uninstaller
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
    if !uninstaller.is_absolute() || !is_exe || !uninstaller.is_file() {
        return Err(AppError::Failed(format!(
            "unexpected uninstaller: {}",
            uninstaller.display()
        )));
    }

    Command::new(&uninstaller).spawn()?;
    tracing::info!("uninstaller started");
    app.exit(0);
    Ok(())
}

#[cfg(not(windows))]
pub fn uninstall(_app: &AppHandle) -> Result<()> {
    Err(AppError::Unsupported)
}
