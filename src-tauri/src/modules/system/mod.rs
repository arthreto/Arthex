//! System commands and the shutdown planner. Commands are a closed list: the interface can only
//! name one of them, never pass a command line.

pub mod commands;
mod planner;
#[cfg(windows)]
mod windows;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

pub use planner::{Planner, Schedule};

use crate::app::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Command {
    Lock,
    Sleep,
    Restart,
    Shutdown,
    SignOut,
    EmptyRecycleBin,
    TaskManager,
    WindowsSettings,
}

pub fn setup(app: &AppHandle) {
    planner::setup(app);
}

#[cfg(windows)]
pub fn run(app: &AppHandle, command: Command) -> Result<()> {
    windows::run(app, command)
}

#[cfg(not(windows))]
pub fn run(_app: &AppHandle, _command: Command) -> Result<()> {
    Err(crate::app::AppError::Unsupported)
}

/// Bytes waiting in the Recycle Bin, for the confirmation before emptying it.
pub fn recycle_bin_size() -> u64 {
    #[cfg(windows)]
    {
        windows::recycle_bin_size()
    }
    #[cfg(not(windows))]
    {
        0
    }
}
