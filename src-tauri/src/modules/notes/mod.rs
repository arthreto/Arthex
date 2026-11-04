//! Markdown notes, saved in `notes.json`.

pub mod commands;

use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard, PoisonError},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::app::{AppError, Result, storage};

const MAX_NOTES: usize = 500;
const MAX_TITLE: usize = 200;
const MAX_TEXT: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: String,
    pub text: String,
    /// Milliseconds since the Unix epoch.
    pub updated: u64,
}

impl Note {
    fn check(&self) -> Result<()> {
        let id_ok = !self.id.is_empty()
            && self.id.len() <= 64
            && self
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
        if !id_ok {
            return Err(AppError::Invalid("bad note id".into()));
        }
        if self.title.chars().count() > MAX_TITLE || self.text.len() > MAX_TEXT {
            return Err(AppError::Invalid("this note is too long".into()));
        }
        Ok(())
    }
}

pub struct Notes {
    path: PathBuf,
    notes: Mutex<Vec<Note>>,
}

impl Notes {
    fn notes(&self) -> MutexGuard<'_, Vec<Note>> {
        self.notes.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The most recently changed first.
    pub fn list(&self) -> Vec<Note> {
        let mut notes = self.notes().clone();
        notes.sort_by_key(|note| std::cmp::Reverse(note.updated));
        notes
    }

    /// Adds the note, or replaces the one with the same id, stamped with the current time.
    pub fn save(&self, mut note: Note) -> Result<Note> {
        note.check()?;
        note.updated = now();
        let mut notes = self.notes();
        if let Some(existing) = notes.iter_mut().find(|other| other.id == note.id) {
            existing.clone_from(&note);
        } else if notes.len() >= MAX_NOTES {
            return Err(AppError::Invalid("too many notes".into()));
        } else {
            notes.push(note.clone());
        }
        storage::write_json(&self.path, &*notes)?;
        Ok(note)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let mut notes = self.notes();
        notes.retain(|note| note.id != id);
        storage::write_json(&self.path, &*notes)
    }
}

pub fn setup(app: &AppHandle) -> Result<()> {
    let path = app.path().app_data_dir()?.join("notes.json");
    let notes: Vec<Note> = storage::read_json(&path).unwrap_or_default();
    let notes = notes
        .into_iter()
        .filter(|note| note.check().is_ok())
        .take(MAX_NOTES)
        .collect();
    app.manage(Notes {
        path,
        notes: Mutex::new(notes),
    });
    Ok(())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(id: &str, text: &str) -> Note {
        Note {
            id: id.into(),
            title: "Courses".into(),
            text: text.into(),
            updated: 0,
        }
    }

    #[test]
    fn saves_replaces_and_deletes() {
        let dir = tempfile::tempdir().unwrap();
        let notes = Notes {
            path: dir.path().join("notes.json"),
            notes: Mutex::default(),
        };
        notes.save(note("a-1", "- [ ] pain")).unwrap();
        notes.save(note("a-1", "- [x] pain")).unwrap();
        assert_eq!(notes.list().len(), 1);
        assert_eq!(notes.list()[0].text, "- [x] pain");
        let saved: Vec<Note> = storage::read_json(&notes.path).unwrap();
        assert_eq!(saved.len(), 1);
        notes.delete("a-1").unwrap();
        assert_eq!(notes.list(), Vec::<Note>::new());
    }

    #[test]
    fn refuses_bad_ids_and_huge_notes() {
        assert!(note("../x", "").check().is_err());
        assert!(note("ok", &"x".repeat(MAX_TEXT + 1)).check().is_err());
        assert!(note("0b9e-77", "").check().is_ok());
    }
}
