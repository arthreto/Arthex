use std::{
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command as Process, Stdio},
    ptr,
};

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use windows_sys::Win32::{
    System::{Power::SetSuspendState, Shutdown::LockWorkStation},
    UI::Shell::{
        SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND, SHEmptyRecycleBinW, SHQUERYRBINFO,
        SHQueryRecycleBinW,
    },
};

use super::Command;
use crate::app::{AppError, Result};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn run(app: &AppHandle, command: Command) -> Result<()> {
    match command {
        // SAFETY: plain Win32 calls without arguments to check.
        Command::Lock => check(unsafe { LockWorkStation() } != 0),
        Command::Sleep => check(unsafe { SetSuspendState(false, false, false) }),
        Command::Restart => system_program("shutdown.exe", &["/r", "/t", "0"]),
        Command::Shutdown => system_program("shutdown.exe", &["/s", "/t", "0"]),
        Command::SignOut => system_program("shutdown.exe", &["/l"]),
        Command::EmptyRecycleBin => {
            // SAFETY: null window and root mean "every drive"; Arthex already asked for confirmation.
            // An empty bin returns an error code, which is not a failure here.
            unsafe {
                SHEmptyRecycleBinW(
                    ptr::null_mut(),
                    ptr::null(),
                    SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
                );
            }
            Ok(())
        }
        Command::TaskManager => system_program("taskmgr.exe", &[]),
        Command::WindowsSettings => app
            .opener()
            .open_url("ms-settings:", None::<&str>)
            .map_err(|error| AppError::Failed(error.to_string())),
    }
}

pub fn recycle_bin_size() -> u64 {
    let mut info = SHQUERYRBINFO {
        cbSize: u32::try_from(std::mem::size_of::<SHQUERYRBINFO>()).unwrap_or_default(),
        i64Size: 0,
        i64NumItems: 0,
    };
    // SAFETY: `info` has its size set, as the call requires.
    let result = unsafe { SHQueryRecycleBinW(ptr::null(), &raw mut info) };
    if result < 0 {
        0
    } else {
        u64::try_from(info.i64Size).unwrap_or(0)
    }
}

fn check(ok: bool) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().into())
    }
}

/// Starts a program of `System32` by its full path, so nothing on the PATH can stand in for it.
fn system_program(name: &str, args: &[&str]) -> Result<()> {
    let root = std::env::var_os("SystemRoot").ok_or(AppError::Unsupported)?;
    let program = PathBuf::from(root).join("System32").join(name);
    Process::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    Ok(())
}
