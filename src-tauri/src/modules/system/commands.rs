use tauri::{AppHandle, Manager};

use super::{Command, Planner, Schedule, planner, recycle_bin_size};
use crate::app::{Result, overlay};

/// Runs one of the system commands. The overlay closes first, so it is not in the way after a
/// lock or a wake from sleep.
#[tauri::command]
pub fn system_run(app: AppHandle, command: Command) -> Result<()> {
    overlay::hide(&app);
    super::run(&app, command)
}

#[tauri::command]
pub fn system_recycle_bin_size() -> u64 {
    recycle_bin_size()
}

#[tauri::command]
pub fn system_schedule(app: AppHandle, schedule: Schedule) -> Result<Schedule> {
    planner::plan(&app, schedule)
}

#[tauri::command]
pub fn system_schedule_cancel(app: AppHandle) {
    planner::cancel(&app);
}

#[tauri::command]
pub fn system_schedule_get(app: AppHandle) -> Option<Schedule> {
    app.state::<Planner>().current()
}
