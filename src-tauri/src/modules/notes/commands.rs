use tauri::{AppHandle, Manager};

use super::{Note, Notes};
use crate::app::Result;

#[tauri::command]
pub fn notes_list(app: AppHandle) -> Vec<Note> {
    app.state::<Notes>().list()
}

/// Creates or updates a note; also restores a deleted one.
#[tauri::command]
pub fn notes_save(app: AppHandle, note: Note) -> Result<Note> {
    app.state::<Notes>().save(note)
}

#[tauri::command]
pub fn notes_delete(app: AppHandle, id: String) -> Result<()> {
    app.state::<Notes>().delete(&id)
}
