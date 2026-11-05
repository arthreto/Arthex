//! Shortcuts to a site, an app, a folder or file, or a command, shown on Home, in the search, or
//! both, with the user's own name, icon and keyword.

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
pub const MAX_ON_HOME: usize = 9;
pub const MAX: usize = 200;
/// Mirrors `TINTS` in `src/core/modules/manifest.ts`.
const TINTS: [&str; 9] = [
    "blue", "amber", "green", "violet", "sky", "pink", "mint", "orange", "red",
];

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
    /// A pictogram of the interface, in place of an image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<Symbol>,
    /// Typed alone in the search, it puts this shortcut first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(default = "shown")]
    pub on_home: bool,
    #[serde(default = "shown")]
    pub in_search: bool,
}

fn shown() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Target {
    Site {
        url: String,
    },
    App {
        path: String,
    },
    /// A folder or file, opened with its usual app.
    Path {
        path: String,
    },
    /// A command line, run as typed.
    Command {
        line: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub tint: String,
}

impl Symbol {
    fn is_valid(&self) -> bool {
        (1..=24).contains(&self.name.len())
            && self
                .name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
            && TINTS.contains(&self.tint.as_str())
    }
}

pub struct Shortcuts {
    path: PathBuf,
    items: Mutex<Vec<Shortcut>>,
}

impl Shortcuts {
    fn load(path: PathBuf) -> Self {
        let mut items: Vec<Shortcut> = storage::read_json(&path).unwrap_or_default();
        items.truncate(MAX);
        let mut on_home = 0;
        for item in &mut items {
            on_home += usize::from(item.on_home);
            item.on_home &= on_home <= MAX_ON_HOME;
        }
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
        fits(&items, &shortcut)?;
        if let Some(existing) = items.iter_mut().find(|item| item.id == shortcut.id) {
            *existing = shortcut;
        } else {
            items.push(shortcut);
        }
        self.write(&items)
    }

    /// Puts a removed shortcut back at its place (undo).
    fn insert(&self, index: usize, shortcut: Shortcut) -> Result<Vec<Shortcut>> {
        let mut items = self.items();
        if items.iter().any(|item| item.id == shortcut.id) {
            return Err(AppError::Invalid("cannot restore this shortcut".into()));
        }
        fits(&items, &shortcut)?;
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

/// The limits a new or edited shortcut must keep: Alt + 1…9 for Home, one shortcut per keyword.
fn fits(items: &[Shortcut], shortcut: &Shortcut) -> Result<()> {
    let others = || items.iter().filter(|item| item.id != shortcut.id);
    if !shortcut.on_home && !shortcut.in_search {
        return Err(AppError::Invalid(
            "a shortcut shows on Home, in the search or both".into(),
        ));
    }
    if others().count() >= MAX {
        return Err(AppError::Invalid(format!("at most {MAX} shortcuts")));
    }
    if shortcut.on_home && others().filter(|item| item.on_home).count() >= MAX_ON_HOME {
        return Err(AppError::Invalid(format!(
            "at most {MAX_ON_HOME} shortcuts on Home"
        )));
    }
    if shortcut.keyword.is_some() && others().any(|item| item.keyword == shortcut.keyword) {
        return Err(AppError::Invalid(
            "another shortcut has this keyword".into(),
        ));
    }
    Ok(())
}

/// A keyword typed by the user: one lowercase word of at most 24 characters, `None` when empty.
fn clean_keyword(keyword: Option<String>) -> Option<String> {
    let keyword = keyword?.trim().to_lowercase();
    (!keyword.is_empty() && keyword.chars().count() <= 24 && !keyword.contains(char::is_whitespace))
        .then_some(keyword)
}

/// A command line typed by the user: one line, at most 2000 characters.
fn clean_line(line: &str) -> Option<String> {
    let line = line.trim();
    (!line.is_empty() && line.len() <= 2000 && !line.contains(['\n', '\r']))
        .then(|| line.to_string())
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
            symbol: None,
            keyword: None,
            on_home: true,
            in_search: true,
        }
    }

    #[test]
    fn keeps_at_most_nine_on_home_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shortcuts.json");
        let store = Shortcuts::load(path.clone());
        for index in 0..MAX_ON_HOME {
            store.save(site(&index.to_string())).unwrap();
        }
        assert!(store.save(site("ten")).is_err());
        let mut search_only = site("ten");
        search_only.on_home = false;
        store.save(search_only).unwrap();

        store
            .set_icon("3", "data:image/png;base64,AA".into())
            .unwrap();
        let reloaded = Shortcuts::load(path);
        assert_eq!(reloaded.list().len(), MAX_ON_HOME + 1);
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
    fn keeps_keywords_unique_and_a_place_to_show() {
        let dir = tempfile::tempdir().unwrap();
        let store = Shortcuts::load(dir.path().join("shortcuts.json"));
        let mut mail = site("mail");
        mail.keyword = Some("mail".into());
        store.save(mail.clone()).unwrap();
        store.save(mail.clone()).unwrap();
        let mut other = site("other");
        other.keyword = Some("mail".into());
        assert!(store.save(other).is_err());
        let mut hidden = site("hidden");
        hidden.on_home = false;
        hidden.in_search = false;
        assert!(store.save(hidden).is_err());
    }

    #[test]
    fn reads_shortcuts_saved_before_keywords() {
        let json = r#"[{ "id": "a", "name": "A", "kind": "app", "path": "C:\\a.exe" }]"#;
        let items: Vec<Shortcut> = serde_json::from_str(json).unwrap();
        assert!(items[0].on_home && items[0].in_search && items[0].keyword.is_none());
    }

    #[test]
    fn cleans_keywords_commands_and_symbols() {
        assert_eq!(
            clean_keyword(Some(" Mail ".into())).as_deref(),
            Some("mail")
        );
        assert_eq!(clean_keyword(Some("two words".into())), None);
        assert_eq!(clean_keyword(Some(String::new())), None);
        assert_eq!(
            clean_line("  code C:\\Projets ").as_deref(),
            Some("code C:\\Projets")
        );
        assert_eq!(clean_line("a\nb"), None);
        let symbol = |name: &str, tint: &str| Symbol {
            name: name.into(),
            tint: tint.into(),
        };
        assert!(symbol("terminal", "green").is_valid());
        assert!(!symbol("Terminal", "green").is_valid());
        assert!(!symbol("terminal", "gold").is_valid());
    }

    #[test]
    fn cleans_names() {
        assert_eq!(clean_name(Some("  Mail  ".into())).as_deref(), Some("Mail"));
        assert_eq!(clean_name(Some("   ".into())), None);
        assert_eq!(clean_name(Some("x".repeat(80))).unwrap().len(), 40);
    }
}
