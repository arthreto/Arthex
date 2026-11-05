//! The catalog: a signed `catalog.json` published next to the packages in the releases of the
//! modules repository. It is fetched when the Catalog tab opens or the user checks for updates,
//! and once at launch to update installed modules when updates install automatically.

use std::{collections::BTreeMap, time::Duration};

use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use super::{
    HOST_API, Installed, ModuleHost,
    package::{self, MAX_PACKAGE_BYTES, Texts, current_platform, is_safe_path},
    public_key,
};
use crate::app::{self, AppError, Result, is_module_id};

const BASE: &str = "https://github.com/arthreto/arthex-modules/releases/latest/download/";
const MAX_CATALOG_BYTES: u64 = 4 * 1024 * 1024;

/// More would not be categories any more.
const MAX_TAGS: usize = 8;

#[derive(Debug, Clone, Deserialize)]
struct PackageFile {
    file: String,
    size: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    id: String,
    version: String,
    api: u32,
    icon: String,
    tint: String,
    elements: Vec<String>,
    /// Categories, among the fixed list the interface knows.
    #[serde(default)]
    tags: Vec<String>,
    /// Name and description per language code.
    texts: BTreeMap<String, Texts>,
    /// Per platform (`windows-x86_64`), or `any` for a module without a program.
    packages: BTreeMap<String, PackageFile>,
}

impl Entry {
    fn package(&self) -> Option<&PackageFile> {
        self.packages
            .get(&current_platform())
            .or_else(|| self.packages.get("any"))
    }

    fn is_valid(&self) -> bool {
        is_module_id(&self.id)
            && Version::parse(&self.version).is_ok()
            && self.icon.len() <= 2000
            && self.tags.len() <= MAX_TAGS
            && self.tags.iter().all(|tag| tag.len() <= 32)
            && self.packages.values().all(|package| {
                is_safe_path(&package.file)
                    && !package.file.contains('/')
                    && package.file.ends_with(".arthex")
            })
    }
}

#[derive(Debug, Deserialize)]
struct Catalog {
    modules: Vec<Entry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OfferState {
    Available,
    Installed,
    Update,
    /// Built for a newer version of Arthex.
    Outdated,
}

/// A module of the catalog that runs on this system, as the interface lists it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    id: String,
    version: String,
    icon: String,
    tint: String,
    elements: Vec<String>,
    tags: Vec<String>,
    texts: BTreeMap<String, Texts>,
    size: u64,
    state: OfferState,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .max_redirects(5)
        .user_agent(concat!("Arthex/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

/// `None` when the file is not published (no catalog yet).
fn download(agent: &ureq::Agent, file: &str, limit: u64) -> Result<Option<Vec<u8>>> {
    match agent.get(format!("{BASE}{file}")).call() {
        Ok(mut response) => response
            .body_mut()
            .with_config()
            .limit(limit)
            .read_to_vec()
            .map(Some)
            .map_err(|error| AppError::Failed(error.to_string())),
        Err(ureq::Error::StatusCode(404)) => Ok(None),
        Err(error) => Err(AppError::Failed(error.to_string())),
    }
}

fn fetch(app: &AppHandle, agent: &ureq::Agent) -> Result<Vec<Entry>> {
    let key = public_key(app)?;
    let Some(catalog) = download(agent, "catalog.json", MAX_CATALOG_BYTES)? else {
        return Ok(Vec::new());
    };
    let signature = download(agent, "catalog.json.sig", 4096)?
        .ok_or_else(|| AppError::Invalid("the catalog is not signed".into()))?;
    package::verify_signature(&catalog, &signature, &key)?;
    let catalog: Catalog = serde_json::from_slice(&catalog)?;
    Ok(catalog
        .modules
        .into_iter()
        .filter(Entry::is_valid)
        .collect())
}

fn is_newer(candidate: &str, current: &str) -> bool {
    match (Version::parse(candidate), Version::parse(current)) {
        (Ok(candidate), Ok(current)) => candidate > current,
        _ => false,
    }
}

fn state(entry: &Entry, installed: Option<&Installed>) -> OfferState {
    match installed {
        Some(module) if !is_newer(&entry.version, &module.version) => OfferState::Installed,
        _ if entry.api > HOST_API => OfferState::Outdated,
        Some(_) => OfferState::Update,
        None => OfferState::Available,
    }
}

pub fn offers(app: &AppHandle) -> Result<Vec<Offer>> {
    let entries = fetch(app, &agent())?;
    let host = app.state::<ModuleHost>();
    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let size = entry.package()?.size;
            Some(Offer {
                state: state(&entry, host.get(&entry.id).as_ref()),
                id: entry.id,
                version: entry.version,
                icon: entry.icon,
                tint: entry.tint,
                elements: entry.elements,
                tags: entry.tags,
                texts: entry.texts,
                size,
            })
        })
        .collect())
}

/// Installs or updates a module from the catalog. The package is checked like any other.
pub fn install(app: &AppHandle, id: &str) -> Result<Installed> {
    let agent = agent();
    let entries = fetch(app, &agent)?;
    let entry = entries
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| AppError::Invalid(format!("{id} is not in the catalog")))?;
    if entry.api > HOST_API {
        return Err(AppError::Outdated);
    }
    let file = entry.package().ok_or(AppError::Unsupported)?;
    let bytes = download(&agent, &file.file, MAX_PACKAGE_BYTES)?
        .ok_or_else(|| AppError::Failed(format!("{} is missing", file.file)))?;
    super::install(app, &bytes, Some(id))
}

pub fn update_at_launch(app: &AppHandle) {
    let installed: Vec<Installed> = app
        .state::<ModuleHost>()
        .list()
        .into_iter()
        .filter(|module| !module.dev)
        .collect();
    if installed.is_empty() || !app::settings(app).auto_update {
        return;
    }
    let entries = match fetch(app, &agent()) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(%error, "could not read the module catalog");
            return;
        }
    };
    for module in installed {
        let Some(entry) = entries.iter().find(|entry| entry.id == module.id) else {
            continue;
        };
        if state(entry, Some(&module)) == OfferState::Update
            && let Err(error) = install(app, &module.id)
        {
            tracing::warn!(module = %module.id, %error, "could not update a module");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(version: &str, api: u32) -> Entry {
        Entry {
            id: "timer".into(),
            version: version.into(),
            api,
            icon: String::new(),
            tint: "green".into(),
            elements: vec!["tool".into()],
            tags: vec!["time".into()],
            texts: BTreeMap::new(),
            packages: BTreeMap::from([(
                "any".into(),
                PackageFile {
                    file: "timer-1.2.0.arthex".into(),
                    size: 1,
                },
            )]),
        }
    }

    fn installed(version: &str) -> Installed {
        Installed {
            id: "timer".into(),
            version: version.into(),
            url: String::new(),
            style: false,
            dev: false,
            dir: std::path::PathBuf::new(),
            info: package::ModuleInfo {
                id: "timer".into(),
                version: version.into(),
                api: 1,
                platform: "any".into(),
                ..package::ModuleInfo::default()
            },
        }
    }

    #[test]
    fn tells_what_can_be_done_with_a_module() {
        assert_eq!(state(&entry("1.2.0", 1), None), OfferState::Available);
        assert_eq!(
            state(&entry("1.2.0", 1), Some(&installed("1.2.0"))),
            OfferState::Installed
        );
        assert_eq!(
            state(&entry("1.10.0", 1), Some(&installed("1.9.0"))),
            OfferState::Update
        );
        assert_eq!(
            state(&entry("2.0.0", HOST_API + 1), Some(&installed("1.0.0"))),
            OfferState::Outdated
        );
    }

    #[test]
    fn keeps_valid_entries_only() {
        assert!(entry("1.2.0", 1).is_valid());
        assert!(!entry("one", 1).is_valid());
        let mut escaping = entry("1.2.0", 1);
        escaping.packages.insert(
            "any".into(),
            PackageFile {
                file: "../timer.arthex".into(),
                size: 1,
            },
        );
        assert!(!escaping.is_valid());
        let mut tagged = entry("1.2.0", 1);
        tagged.tags = vec!["time".into(); MAX_TAGS + 1];
        assert!(!tagged.is_valid());
    }
}
