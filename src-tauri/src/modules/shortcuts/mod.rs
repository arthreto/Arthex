//! Website and software shortcuts shown on Home.

pub mod commands;
mod favicon;
mod site;

use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard, PoisonError},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::app::{AppError, Result, storage};

pub const CHANGED: &str = "shortcuts://changed";
/// One per Alt + 1…9 key.
pub const MAX: usize = 9;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shortcut {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub target: Target,
    /// PNG or SVG data URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Target {
    Site { url: String },
    App { path: String },
}

pub struct Shortcuts {
    path: PathBuf,
    items: Mutex<Vec<Shortcut>>,
}

impl Shortcuts {
    fn load(path: PathBuf) -> Self {
        let mut items: Vec<Shortcut> = storage::read_json(&path).unwrap_or_default();
        items.truncate(MAX);
        Self {
            path,
            items: Mutex::new(items),
        }
    }

    fn items(&self) -> MutexGuard<'_, Vec<Shortcut>> {
        self.items.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn list(&self) -> Vec<Shortcut> {
        self.items().clone()
    }

    fn get(&self, id: &str) -> Option<Shortcut> {
        self.items().iter().find(|item| item.id == id).cloned()
    }

    fn add(&self, shortcut: Shortcut) -> Result<Vec<Shortcut>> {
        let mut items = self.items();
        if items.len() >= MAX {
            return Err(AppError::Invalid(format!("at most {MAX} shortcuts")));
        }
        items.push(shortcut);
        storage::write_json(&self.path, &*items)?;
        Ok(items.clone())
    }

    fn set_icon(&self, id: &str, icon: String) -> Result<Vec<Shortcut>> {
        let mut items = self.items();
        if let Some(item) = items.iter_mut().find(|item| item.id == id) {
            item.icon = Some(icon);
            storage::write_json(&self.path, &*items)?;
        }
        Ok(items.clone())
    }
}

pub fn setup(app: &AppHandle) -> Result<()> {
    let path = app.path().app_data_dir()?.join("shortcuts.json");
    app.manage(Shortcuts::load(path));
    Ok(())
}

fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    format!("{nanos:x}")
}

/// A name typed by the user: trimmed, at most 40 characters, `None` when empty.
fn clean_name(name: Option<String>) -> Option<String> {
    let name: String = name?.trim().chars().take(40).collect();
    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(id: &str) -> Shortcut {
        Shortcut {
            id: id.into(),
            name: id.into(),
            target: Target::Site {
                url: format!("https://{id}.com/"),
            },
            icon: None,
        }
    }

    #[test]
    fn keeps_at_most_nine_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shortcuts.json");
        let store = Shortcuts::load(path.clone());
        for index in 0..MAX {
            store.add(site(&index.to_string())).unwrap();
        }
        assert!(store.add(site("ten")).is_err());

        store
            .set_icon("3", "data:image/png;base64,AA".into())
            .unwrap();
        let reloaded = Shortcuts::load(path);
        assert_eq!(reloaded.list().len(), MAX);
        assert_eq!(
            reloaded.get("3").unwrap().icon.as_deref(),
            Some("data:image/png;base64,AA")
        );
    }

    #[test]
    fn stores_the_target_kind_in_the_json() {
        let json = serde_json::to_value(site("yt")).unwrap();
        assert_eq!(json["kind"], "site");
        assert_eq!(json["url"], "https://yt.com/");
    }

    #[test]
    fn cleans_names() {
        assert_eq!(clean_name(Some("  Mail  ".into())).as_deref(), Some("Mail"));
        assert_eq!(clean_name(Some("   ".into())), None);
        assert_eq!(clean_name(Some("x".repeat(80))).unwrap().len(), 40);
    }
}
