use std::fs;

use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use super::{AppError, Result};

/// Writes what lets the overlay show the desktop through it: Windows may turn the effect off.
#[cfg(windows)]
pub fn log_display() {
    use windows_sys::Win32::{
        System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS},
        UI::WindowsAndMessaging::{GetSystemMetrics, SM_REMOTESESSION},
    };
    use winreg::{
        RegKey,
        enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    };

    let build: Option<String> = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .and_then(|key| key.get_value("CurrentBuildNumber"))
        .ok();
    let transparency: Option<u32> = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|key| key.get_value("EnableTransparency"))
        .ok();
    // SAFETY: plain queries; the status is written to a local the call borrows.
    let (remote, energy_saver) = unsafe {
        let mut power: SYSTEM_POWER_STATUS = std::mem::zeroed();
        let saver = GetSystemPowerStatus(&raw mut power) != 0 && power.SystemStatusFlag == 1;
        (GetSystemMetrics(SM_REMOTESESSION) != 0, saver)
    };
    tracing::info!(
        build = build.as_deref().unwrap_or("unknown"),
        transparency = ?transparency,
        remote,
        energy_saver,
        "display"
    );
}

#[cfg(not(windows))]
pub fn log_display() {}

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
