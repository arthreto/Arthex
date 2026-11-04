use tauri::{AppHandle, Manager};

use super::{Clipboard, State};

#[tauri::command]
pub fn clipboard_state(app: AppHandle) -> State {
    app.state::<Clipboard>().state()
}

#[tauri::command]
pub fn clipboard_pause(app: AppHandle, paused: bool) -> State {
    let clipboard = app.state::<Clipboard>();
    clipboard.set_paused(paused);
    clipboard.state()
}

#[tauri::command]
pub fn clipboard_pin(app: AppHandle, id: u64, pinned: bool) -> State {
    let clipboard = app.state::<Clipboard>();
    clipboard.pin(id, pinned);
    clipboard.state()
}

#[tauri::command]
pub fn clipboard_remove(app: AppHandle, id: u64) -> State {
    let clipboard = app.state::<Clipboard>();
    clipboard.remove(id);
    clipboard.state()
}

#[tauri::command]
pub fn clipboard_clear(app: AppHandle) -> State {
    let clipboard = app.state::<Clipboard>();
    clipboard.clear();
    clipboard.state()
}
