use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{PoisonError, RwLock},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{AppError, Result};

const VERSION: u32 = 1;
const HOME: &str = "home";
const MAX_TAB_BAR: usize = 6;
const MAX_HOTKEY_KEYS: usize = 4;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
}

/// Mirrors `src/core/settings/types.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub onboarded: bool,
    pub language: Option<String>,
    pub theme: Theme,
    pub reduce_motion: bool,
    pub autostart: bool,
    pub hotkey: Vec<String>,
    pub tab_bar: Vec<String>,
    pub favorites: Vec<String>,
    pub modules: BTreeMap<String, bool>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: VERSION,
            onboarded: false,
            language: None,
            theme: Theme::System,
            reduce_motion: false,
            autostart: true,
            hotkey: vec!["Shift".into(), "Win".into()],
            tab_bar: ["home", "filesize", "notes", "clipboard"]
                .map(String::from)
                .to_vec(),
            favorites: Vec::new(),
            modules: BTreeMap::new(),
        }
    }
}

impl Settings {
    /// Repairs anything a hand-edited file or a buggy client could break.
    fn normalize(&mut self) {
        let defaults = Self::default();
        self.version = VERSION;

        if self
            .language
            .as_deref()
            .is_some_and(|code| !is_language_code(code))
        {
            self.language = None;
        }

        dedupe_ids(&mut self.tab_bar);
        self.tab_bar.retain(|id| id != HOME);
        self.tab_bar.insert(0, HOME.into());
        self.tab_bar.truncate(MAX_TAB_BAR);

        dedupe_ids(&mut self.favorites);
        self.modules.retain(|id, _| id != HOME && is_module_id(id));

        let mut keys = Vec::with_capacity(self.hotkey.len());
        for key in self.hotkey.drain(..) {
            if is_key_name(&key) && !keys.contains(&key) {
                keys.push(key);
            }
        }
        self.hotkey = if keys.is_empty() || keys.len() > MAX_HOTKEY_KEYS {
            defaults.hotkey
        } else {
            keys
        };
    }
}

fn dedupe_ids(ids: &mut Vec<String>) {
    let mut seen = Vec::with_capacity(ids.len());
    ids.retain(|id| {
        let keep = is_module_id(id) && !seen.contains(id);
        if keep {
            seen.push(id.clone());
        }
        keep
    });
}

fn is_module_id(id: &str) -> bool {
    (1..=32).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_language_code(code: &str) -> bool {
    (2..=12).contains(&code.len())
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn is_key_name(key: &str) -> bool {
    (1..=16).contains(&key.len()) && key.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

/// Owns `settings.json`. Every write replaces the file atomically.
pub struct SettingsStore {
    path: PathBuf,
    current: RwLock<Settings>,
}

impl SettingsStore {
    /// Never fails: a missing file gives the defaults, an unreadable one is set aside.
    pub fn load(path: PathBuf) -> Self {
        let mut settings = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|error| {
                tracing::warn!(%error, "settings file is corrupt, using defaults");
                let _ = fs::rename(&path, path.with_extension("json.corrupt"));
                Settings::default()
            }),
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(%error, "could not read the settings file");
                }
                Settings::default()
            }
        };
        settings.normalize();
        Self {
            path,
            current: RwLock::new(settings),
        }
    }

    pub fn get(&self) -> Settings {
        self.current
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Applies a partial update such as `{"theme": "dark"}` and persists the result.
    pub fn update(&self, patch: &Value) -> Result<Settings> {
        let Value::Object(patch) = patch else {
            return Err(AppError::InvalidSettings("expected an object".into()));
        };
        let mut current = self.current.write().unwrap_or_else(PoisonError::into_inner);

        let Value::Object(mut merged) = serde_json::to_value(&*current)? else {
            unreachable!("settings serialize to an object");
        };
        for (key, value) in patch {
            if key == "version" || !merged.contains_key(key) {
                return Err(AppError::InvalidSettings(format!("unknown key `{key}`")));
            }
            merged.insert(key.clone(), value.clone());
        }

        let mut next: Settings = serde_json::from_value(Value::Object(merged))
            .map_err(|error| AppError::InvalidSettings(error.to_string()))?;
        next.normalize();
        write_atomic(&self.path, &serde_json::to_vec_pretty(&next)?)?;
        *current = next.clone();
        Ok(next)
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn store() -> (tempfile::TempDir, SettingsStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::load(dir.path().join("settings.json"));
        (dir, store)
    }

    #[test]
    fn missing_file_gives_defaults() {
        let (_dir, store) = store();
        assert_eq!(store.get(), Settings::default());
    }

    #[test]
    fn update_merges_and_persists() {
        let (dir, store) = store();
        let saved = store
            .update(&json!({ "theme": "dark", "onboarded": true }))
            .unwrap();
        assert_eq!(saved.theme, Theme::Dark);
        assert!(saved.onboarded);

        let reloaded = SettingsStore::load(dir.path().join("settings.json"));
        assert_eq!(reloaded.get(), saved);
    }

    #[test]
    fn rejects_unknown_keys_and_bad_values() {
        let (_dir, store) = store();
        assert!(store.update(&json!({ "nope": 1 })).is_err());
        assert!(store.update(&json!({ "version": 9 })).is_err());
        assert!(store.update(&json!({ "theme": "neon" })).is_err());
        assert!(store.update(&json!([1, 2])).is_err());
        assert_eq!(store.get(), Settings::default());
    }

    #[test]
    fn keeps_home_first_and_caps_the_tab_bar() {
        let (_dir, store) = store();
        let saved = store
            .update(
                &json!({ "tabBar": ["notes", "home", "notes", "a", "b", "c", "d", "e", "Bad Id"] }),
            )
            .unwrap();
        assert_eq!(saved.tab_bar, ["home", "notes", "a", "b", "c", "d"]);
    }

    #[test]
    fn home_cannot_be_disabled() {
        let (_dir, store) = store();
        let saved = store
            .update(&json!({ "modules": { "home": false, "notes": false } }))
            .unwrap();
        assert_eq!(
            saved.modules,
            BTreeMap::from([("notes".to_string(), false)])
        );
    }

    #[test]
    fn invalid_hotkey_falls_back_to_default() {
        let (_dir, store) = store();
        assert_eq!(
            store.update(&json!({ "hotkey": [] })).unwrap().hotkey,
            ["Shift", "Win"]
        );
        assert_eq!(
            store
                .update(&json!({ "hotkey": ["a", "b", "c", "d", "e"] }))
                .unwrap()
                .hotkey,
            ["Shift", "Win"]
        );
        assert_eq!(
            store
                .update(&json!({ "hotkey": ["Ctrl", "Space", "Ctrl"] }))
                .unwrap()
                .hotkey,
            ["Ctrl", "Space"]
        );
    }

    #[test]
    fn corrupt_file_is_set_aside() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, "{ not json").unwrap();

        let store = SettingsStore::load(path.clone());
        assert_eq!(store.get(), Settings::default());
        assert!(path.with_extension("json.corrupt").exists());
    }
}
