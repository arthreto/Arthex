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

    /// Replaces the shortcut with the same id, or adds it at the end.
    fn save(&self, shortcut: Shortcut) -> Result<Vec<Shortcut>> {
        let mut items = self.items();
        if let Some(existing) = items.iter_mut().find(|item| item.id == shortcut.id) {
            *existing = shortcut;
        } else if items.len() >= MAX {
            return Err(AppError::Invalid(format!("at most {MAX} shortcuts")));
        } else {
            items.push(shortcut);
        }
        self.write(&items)
    }

    /// Puts a removed shortcut back at its place (undo).
    fn insert(&self, index: usize, shortcut: Shortcut) -> Result<Vec<Shortcut>> {
        let mut items = self.items();
        if items.len() >= MAX || items.iter().any(|item| item.id == shortcut.id) {
            return Err(AppError::Invalid("cannot restore this shortcut".into()));
        }
        let index = index.min(items.len());
        items.insert(index, shortcut);
        self.write(&items)
    }

    fn remove(&self, id: &str) -> Result<Vec<Shortcut>> {
        let mut items = self.items();
        items.retain(|item| item.id != id);
        self.write(&items)
    }

    /// Applies a new order, given as every id exactly once.
    fn reorder(&self, ids: &[String]) -> Result<Vec<Shortcut>> {
        let mut items = self.items();
        let is_permutation = ids.len() == items.len()
            && items
                .iter()
                .all(|item| ids.iter().filter(|id| **id == item.id).count() == 1);
        if !is_permutation {
            return Err(AppError::Invalid(
                "the new order must list every shortcut once".into(),
            ));
        }
        items.sort_by_key(|item| ids.iter().position(|id| *id == item.id));
        self.write(&items)
    }

    fn write(&self, items: &[Shortcut]) -> Result<Vec<Shortcut>> {
        storage::write_json(&self.path, &items)?;
        Ok(items.to_vec())
    }

    fn set_icon(&self, id: &str, icon: String) -> Result<Vec<Shortcut>> {
        let mut items = self.items();
        if let Some(item) = items.iter_mut().find(|item| item.id == id) {
            item.icon = Some(icon);
        }
        self.write(&items)
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
            store.save(site(&index.to_string())).unwrap();
        }
        assert!(store.save(site("ten")).is_err());

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
    fn edits_in_place_and_restores_at_the_same_place() {
        let dir = tempfile::tempdir().unwrap();
        let store = Shortcuts::load(dir.path().join("shortcuts.json"));
        for id in ["a", "b", "c"] {
            store.save(site(id)).unwrap();
        }
        let mut renamed = site("b");
        renamed.name = "Bee".into();
        let list = store.save(renamed).unwrap();
        assert_eq!(list[1].name, "Bee");

        let removed = store.get("b").unwrap();
        store.remove("b").unwrap();
        let list = store.insert(1, removed).unwrap();
        assert_eq!(ids(&list), ["a", "b", "c"]);
    }

    #[test]
    fn reorders_only_with_every_id() {
        let dir = tempfile::tempdir().unwrap();
        let store = Shortcuts::load(dir.path().join("shortcuts.json"));
        for id in ["a", "b", "c"] {
            store.save(site(id)).unwrap();
        }
        let list = store
            .reorder(&["c".into(), "a".into(), "b".into()])
            .unwrap();
        assert_eq!(ids(&list), ["c", "a", "b"]);
        assert!(store.reorder(&["c".into(), "a".into()]).is_err());
        assert!(
            store
                .reorder(&["c".into(), "a".into(), "x".into()])
                .is_err()
        );
        assert_eq!(ids(&store.list()), ["c", "a", "b"]);
    }

    fn ids(list: &[Shortcut]) -> Vec<&str> {
        list.iter().map(|item| item.id.as_str()).collect()
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
