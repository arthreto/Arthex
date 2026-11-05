use std::{
    path::PathBuf,
    sync::{PoisonError, RwLock},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{AppError, Result, hotkey::Hotkey, storage};

const VERSION: u32 = 2;
const HOME: &str = "home";
const MAX_TAB_BAR: usize = 6;

/// What a module brings to the overlay. Each one has its own switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Page,
    Widget,
    Search,
    Tool,
}

impl Element {
    pub const ALL: [Self; 4] = [Self::Page, Self::Widget, Self::Search, Self::Tool];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Widget => "widget",
            Self::Search => "search",
            Self::Tool => "tool",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|element| element.as_str() == name)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
}

/// Mirrors `src/core/settings/types.ts`.
// Independent switches of the settings file, not a state machine.
#[allow(clippy::struct_excessive_bools)]
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
    /// Turned-off elements, as `<module id>:<element>`.
    pub disabled_elements: Vec<String>,
    /// Installs updates found at launch without asking.
    pub auto_update: bool,
}

impl Settings {
    pub fn is_element_enabled(&self, id: &str, element: Element) -> bool {
        let key = format!("{id}:{}", element.as_str());
        !self.disabled_elements.contains(&key)
    }

    /// A module runs while at least one of its elements is on.
    pub fn is_module_active(&self, id: &str, elements: &[Element]) -> bool {
        elements
            .iter()
            .any(|element| self.is_element_enabled(id, *element))
    }
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
            disabled_elements: Vec::new(),
            auto_update: true,
        }
    }
}

impl Settings {
    /// Repairs anything a hand-edited file or a buggy client could break.
    fn normalize(&mut self) {
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
        let mut seen = Vec::with_capacity(self.disabled_elements.len());
        self.disabled_elements.retain(|key| {
            let keep = is_element_key(key) && !seen.contains(key);
            if keep {
                seen.push(key.clone());
            }
            keep
        });

        self.hotkey = Hotkey::parse(&self.hotkey)
            .unwrap_or(Hotkey::DEFAULT)
            .names();
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

pub fn is_module_id(id: &str) -> bool {
    (1..=32).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_element_key(key: &str) -> bool {
    key.split_once(':').is_some_and(|(id, element)| {
        id != HOME && is_module_id(id) && Element::parse(element).is_some()
    })
}

/// Version 1 had one switch per module: a module turned off gets all its elements off.
fn migrate(mut raw: Value) -> Value {
    let Value::Object(fields) = &mut raw else {
        return raw;
    };
    if let Some(Value::Object(modules)) = fields.remove("modules") {
        let disabled: Vec<Value> = modules
            .iter()
            .filter(|(_, enabled)| **enabled == Value::Bool(false))
            .flat_map(|(id, _)| {
                Element::ALL.map(|element| Value::String(format!("{id}:{}", element.as_str())))
            })
            .collect();
        fields.insert("disabledElements".into(), Value::Array(disabled));
    }
    raw
}

fn is_language_code(code: &str) -> bool {
    (2..=12).contains(&code.len())
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

/// Owns `settings.json`. Every write replaces the file atomically.
pub struct SettingsStore {
    path: PathBuf,
    current: RwLock<Settings>,
}

impl SettingsStore {
    /// Never fails: a missing file gives the defaults, an unreadable one is set aside.
    pub fn load(path: PathBuf) -> Self {
        let mut settings: Settings = storage::read_json::<Value>(&path)
            .and_then(|raw| {
                serde_json::from_value(migrate(raw))
                    .inspect_err(|error| tracing::warn!(%error, "settings are unusable"))
                    .ok()
            })
            .unwrap_or_default();
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
        if Hotkey::parse(&next.hotkey).is_none() {
            return Err(AppError::InvalidSettings("unusable hotkey".into()));
        }
        next.normalize();
        storage::write_json(&self.path, &next)?;
        *current = next.clone();
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

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
    fn keeps_valid_element_switches_only() {
        let (_dir, store) = store();
        let saved = store
            .update(&json!({ "disabledElements": [
                "home:page", "notes:widget", "notes:widget", "notes:nope", "Bad Id:page", "clipboard"
            ] }))
            .unwrap();
        assert_eq!(saved.disabled_elements, ["notes:widget"]);
        assert!(!saved.is_element_enabled("notes", Element::Widget));
        assert!(saved.is_element_enabled("notes", Element::Page));
        assert!(saved.is_module_active("notes", &[Element::Page, Element::Widget]));
        assert!(!saved.is_module_active("notes", &[Element::Widget]));
    }

    #[test]
    fn migrates_module_switches_to_elements() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(
            &path,
            r#"{ "version": 1, "theme": "dark", "modules": { "notes": false, "clipboard": true } }"#,
        )
        .unwrap();
        let settings = SettingsStore::load(path).get();
        assert_eq!(settings.version, VERSION);
        assert_eq!(settings.theme, Theme::Dark);
        assert_eq!(
            settings.disabled_elements,
            ["notes:page", "notes:widget", "notes:search", "notes:tool"]
        );
    }

    #[test]
    fn hotkey_is_validated_and_ordered() {
        let (dir, store) = store();
        for bad in [
            json!([]),
            json!(["Shift"]),
            json!(["A"]),
            json!(["Ctrl", "Space", "Ctrl"]),
        ] {
            assert!(store.update(&json!({ "hotkey": bad })).is_err());
        }
        let saved = store
            .update(&json!({ "hotkey": ["Space", "Ctrl"] }))
            .unwrap();
        assert_eq!(saved.hotkey, ["Ctrl", "Space"]);

        fs::write(
            dir.path().join("settings.json"),
            r#"{ "hotkey": ["Nope"] }"#,
        )
        .unwrap();
        let reloaded = SettingsStore::load(dir.path().join("settings.json"));
        assert_eq!(reloaded.get().hotkey, ["Shift", "Win"]);
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
