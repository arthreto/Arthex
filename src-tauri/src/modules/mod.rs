//! Rust side of the modules in `src/modules/`, one folder per module id.

pub mod shortcuts;

use tauri::AppHandle;

use crate::app::Result;

pub fn setup(app: &AppHandle) -> Result<()> {
    shortcuts::setup(app)
}
