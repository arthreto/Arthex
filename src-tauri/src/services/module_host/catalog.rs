//! The catalog: a signed `catalog.json` published next to the packages in the releases of the
//! modules repository. It is fetched when the Catalog tab opens or the user checks for updates,
//! and once at launch to update installed modules when updates install automatically. The last one
//! is kept on disk, so that the tab shows it at once while the network answers.

use std::{collections::BTreeMap, fs, path::PathBuf, thread, time::Duration};

use base64::{Engine, engine::general_purpose::STANDARD};
use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use super::{
    HOST_API, Installed, ModuleHost,
    context_menu::Action,
    package::{self, MAX_PACKAGE_BYTES, Texts, current_platform, is_safe_path},
    program, public_key, shared,
};
use crate::app::{self, AppError, Result, is_module_id};

const BASE: &str = "https://github.com/arthreto/arthex-modules/releases/latest/download/";
const MAX_CATALOG_BYTES: u64 = 4 * 1024 * 1024;
const MAX_PREVIEW_BYTES: u64 = 2 * 1024 * 1024;
const CATALOG_FILE: &str = "catalog.json";
const SIGNATURE_FILE: &str = "catalog.json.sig";
/// A package is large, and a slow connection must not fail it.
const PACKAGE_TIMEOUT: Duration = Duration::from_secs(1800);

/// More would not be categories any more.
const MAX_TAGS: usize = 8;

#[derive(Debug, Clone, Deserialize)]
struct PackageFile {
    file: String,
    size: u64,
}

/// An image of the module, next to the packages. The catalog is signed, so its hash is enough.
#[derive(Debug, Clone, Deserialize)]
struct PreviewFile {
    file: String,
    sha256: String,
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
    /// Its actions in the right-click menu, to show before it is installed.
    #[serde(default)]
    context_menu: Vec<Action>,
    /// The shared parts it needs.
    #[serde(default)]
    uses: Vec<String>,
    /// The day it last changed, as `2026-11-05`.
    #[serde(default)]
    updated: Option<String>,
    #[serde(default)]
    preview: Option<PreviewFile>,
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
            && self.packages.values().all(is_package_file)
            && self.uses.iter().all(|id| is_module_id(id))
            && self.preview.as_ref().is_none_or(|preview| {
                is_safe_path(&preview.file)
                    && !preview.file.contains('/')
                    && preview_type(&preview.file).is_some()
            })
    }
}

fn preview_type(file: &str) -> Option<&'static str> {
    match file.rsplit('.').next()? {
        "webp" => Some("image/webp"),
        "png" => Some("image/png"),
        "jpg" => Some("image/jpeg"),
        _ => None,
    }
}

/// A part several modules use, published next to them.
#[derive(Debug, Clone, Deserialize)]
struct SharedEntry {
    id: String,
    name: String,
    version: String,
    packages: BTreeMap<String, PackageFile>,
}

impl SharedEntry {
    fn package(&self) -> Option<&PackageFile> {
        self.packages
            .get(&current_platform())
            .or_else(|| self.packages.get("any"))
    }

    fn is_valid(&self) -> bool {
        is_module_id(&self.id)
            && Version::parse(&self.version).is_ok()
            && self.name.len() <= 40
            && self.packages.values().all(is_package_file)
    }
}

fn is_package_file(package: &PackageFile) -> bool {
    is_safe_path(&package.file) && !package.file.contains('/') && package.file.ends_with(".arthex")
}

#[derive(Debug, Default, Deserialize)]
struct Catalog {
    modules: Vec<Entry>,
    /// An older catalog has none.
    #[serde(default)]
    shared: Vec<SharedEntry>,
}

/// A shared part a module needs, for what the details say before it is installed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedOffer {
    name: String,
    size: u64,
    installed: bool,
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
    /// Where it runs: `any`, or systems such as `windows-x86_64`.
    platforms: Vec<String>,
    context_menu: Vec<Action>,
    updated: Option<String>,
    has_preview: bool,
    shared: Vec<SharedOffer>,
}

fn agent() -> ureq::Agent {
    agent_with_timeout(Duration::from_secs(60))
}

fn agent_with_timeout(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
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

fn fetch(app: &AppHandle, agent: &ureq::Agent) -> Result<Catalog> {
    let key = public_key(app)?;
    let (catalog, signature) = thread::scope(|scope| {
        let signature = scope.spawn(|| download(agent, SIGNATURE_FILE, 4096));
        (
            download(agent, CATALOG_FILE, MAX_CATALOG_BYTES),
            signature.join(),
        )
    });
    let Some(catalog) = catalog? else {
        return Ok(Catalog::default());
    };
    let signature = signature
        .map_err(|_| AppError::Failed("the download of the signature stopped".into()))??
        .ok_or_else(|| AppError::Invalid("the catalog is not signed".into()))?;
    let read = read_signed(&catalog, &signature, &key)?;
    keep(app, &catalog, &signature);
    Ok(read)
}

fn read_signed(catalog: &[u8], signature: &[u8], key: &str) -> Result<Catalog> {
    package::verify_signature(catalog, signature, key)?;
    let mut catalog: Catalog = serde_json::from_slice(catalog)?;
    catalog.modules.retain(Entry::is_valid);
    catalog.shared.retain(SharedEntry::is_valid);
    Ok(catalog)
}

fn kept_dir(app: &AppHandle) -> Result<PathBuf> {
    Ok(app.path().app_local_data_dir()?.join("module-catalog"))
}

fn keep(app: &AppHandle, catalog: &[u8], signature: &[u8]) {
    let kept = kept_dir(app).and_then(|dir| {
        fs::create_dir_all(&dir)?;
        fs::write(dir.join(CATALOG_FILE), catalog)?;
        fs::write(dir.join(SIGNATURE_FILE), signature)?;
        Ok(())
    });
    if let Err(error) = kept {
        tracing::warn!(%error, "could not keep the module catalog");
    }
}

/// The last catalog read, if any. It is checked like a download: any program can write to its folder.
pub fn kept_offers(app: &AppHandle) -> Result<Option<Vec<Offer>>> {
    let dir = kept_dir(app)?;
    let (Ok(catalog), Ok(signature)) = (
        fs::read(dir.join(CATALOG_FILE)),
        fs::read(dir.join(SIGNATURE_FILE)),
    ) else {
        return Ok(None);
    };
    match read_signed(&catalog, &signature, &public_key(app)?) {
        Ok(catalog) => Ok(Some(to_offers(app, catalog))),
        Err(error) => {
            tracing::warn!(%error, "the kept module catalog is unusable");
            Ok(None)
        }
    }
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
    Ok(to_offers(app, fetch(app, &agent())?))
}

fn to_offers(app: &AppHandle, catalog: Catalog) -> Vec<Offer> {
    let host = app.state::<ModuleHost>();
    let parts = catalog.shared;
    catalog
        .modules
        .into_iter()
        .filter_map(|entry| {
            let size = entry.package()?.size;
            let needed = entry
                .uses
                .iter()
                .filter_map(|id| {
                    let part = parts.iter().find(|part| part.id == *id)?;
                    Some(SharedOffer {
                        name: part.name.clone(),
                        size: part.package()?.size,
                        installed: shared::folder_of(&host, id).is_some(),
                    })
                })
                .collect();
            Some(Offer {
                shared: needed,
                state: state(&entry, host.get(&entry.id).as_ref()),
                id: entry.id,
                version: entry.version,
                icon: entry.icon,
                tint: entry.tint,
                elements: entry.elements,
                tags: entry.tags,
                texts: entry.texts,
                size,
                platforms: entry.packages.into_keys().collect(),
                context_menu: entry
                    .context_menu
                    .into_iter()
                    .filter(Action::is_valid)
                    .collect(),
                updated: entry.updated,
                has_preview: entry.preview.is_some(),
            })
        })
        .collect()
}

/// The image of a module, as a data address: the interface cannot load a remote one.
pub fn preview(app: &AppHandle, id: &str) -> Result<Option<String>> {
    let dir = kept_dir(app)?;
    let (catalog, signature) = (
        fs::read(dir.join(CATALOG_FILE))?,
        fs::read(dir.join(SIGNATURE_FILE))?,
    );
    let Some(preview) = read_signed(&catalog, &signature, &public_key(app)?)?
        .modules
        .into_iter()
        .find(|entry| entry.id == id)
        .and_then(|entry| entry.preview)
    else {
        return Ok(None);
    };
    let bytes = download(&agent(), &preview.file, MAX_PREVIEW_BYTES)?
        .ok_or_else(|| AppError::Failed(format!("{} is missing", preview.file)))?;
    if package::sha256(&bytes) != preview.sha256 {
        return Err(AppError::Invalid(format!("{} was altered", preview.file)));
    }
    Ok(preview_type(&preview.file)
        .map(|kind| format!("data:{kind};base64,{}", STANDARD.encode(bytes))))
}

fn install_shared(app: &AppHandle, part: &SharedEntry) -> Result<()> {
    let file = part.package().ok_or(AppError::Unsupported)?;
    let bytes = download(
        &agent_with_timeout(PACKAGE_TIMEOUT),
        &file.file,
        MAX_PACKAGE_BYTES,
    )?
    .ok_or_else(|| AppError::Failed(format!("{} is missing", file.file)))?;
    shared::install(
        &app.state::<ModuleHost>(),
        &bytes,
        &part.id,
        &public_key(app)?,
    )
}

/// What is wanted and not there, or older than what the catalog holds.
fn to_fetch<'a>(host: &ModuleHost, catalog: &'a Catalog, uses: &[String]) -> Vec<&'a SharedEntry> {
    catalog
        .shared
        .iter()
        .filter(|part| uses.contains(&part.id))
        .filter(|part| match shared::folder_of(host, &part.id) {
            None => true,
            Some(_) => shared::version_of(host, &part.id)
                .is_some_and(|version| is_newer(&part.version, &version)),
        })
        .collect()
}

/// Fetches what a module needs and is not there yet, for one that is about to be installed. The
/// parts already there stay as they are, so this needs no network then.
pub fn ensure_shared(app: &AppHandle, uses: &[String]) -> Result<()> {
    let host = app.state::<ModuleHost>();
    if uses.iter().all(|id| shared::folder_of(&host, id).is_some()) {
        return Ok(());
    }
    let catalog = fetch(app, &agent())?;
    for id in uses {
        if shared::folder_of(&host, id).is_some() {
            continue;
        }
        let part = catalog
            .shared
            .iter()
            .find(|part| part.id == *id)
            .ok_or_else(|| AppError::Invalid(format!("{id} is not in the catalog")))?;
        install_shared(app, part)?;
    }
    Ok(())
}

/// Installs or updates a module from the catalog. The package is checked like any other.
pub fn install(app: &AppHandle, id: &str) -> Result<Installed> {
    let agent = agent_with_timeout(PACKAGE_TIMEOUT);
    let catalog = fetch(app, &agent)?;
    let entry = catalog
        .modules
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| AppError::Invalid(format!("{id} is not in the catalog")))?;
    if entry.api > HOST_API {
        return Err(AppError::Outdated);
    }
    for part in to_fetch(&app.state::<ModuleHost>(), &catalog, &entry.uses) {
        install_shared(app, part)?;
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
    let catalog = match fetch(app, &agent()) {
        Ok(catalog) => catalog,
        Err(error) => {
            tracing::warn!(%error, "could not read the module catalog");
            return;
        }
    };
    for module in installed {
        let Some(entry) = catalog.modules.iter().find(|entry| entry.id == module.id) else {
            continue;
        };
        if state(entry, Some(&module)) == OfferState::Update
            && let Err(error) = install(app, &module.id)
        {
            tracing::warn!(module = %module.id, %error, "could not update a module");
        }
    }
    update_shared(app, &catalog);
}

/// A part updates once for all the modules using it, whose programs start again afterwards.
fn update_shared(app: &AppHandle, catalog: &Catalog) {
    let host = app.state::<ModuleHost>();
    let uses: Vec<String> = host
        .list()
        .iter()
        .flat_map(|module| module.info.uses.clone())
        .collect();
    let outdated = to_fetch(&host, catalog, &uses);
    if outdated.is_empty() {
        return;
    }
    for part in outdated {
        if let Err(error) = install_shared(app, part) {
            tracing::warn!(shared = %part.id, %error, "could not update a shared part");
        }
    }
    program::sync(app, &app::settings(app));
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
            context_menu: Vec::new(),
            uses: Vec::new(),
            updated: None,
            preview: None,
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
        let mut sharing = entry("1.2.0", 1);
        sharing.uses = vec!["../ffmpeg".into()];
        assert!(!sharing.is_valid());
    }

    #[test]
    fn keeps_valid_shared_parts_only() {
        let part = |id: &str, file: &str| SharedEntry {
            id: id.into(),
            name: "FFmpeg".into(),
            version: "8.1.3".into(),
            packages: BTreeMap::from([(
                "any".into(),
                PackageFile {
                    file: file.into(),
                    size: 1,
                },
            )]),
        };
        assert!(part("ffmpeg", "ffmpeg-8.1.3.arthex").is_valid());
        assert!(!part("../ffmpeg", "ffmpeg-8.1.3.arthex").is_valid());
        assert!(!part("ffmpeg", "../ffmpeg.arthex").is_valid());
        assert!(!part("ffmpeg", "ffmpeg.exe").is_valid());
    }

    #[test]
    fn an_older_catalog_has_no_shared_parts() {
        let catalog: Catalog = serde_json::from_str(r#"{"modules":[]}"#).unwrap();
        assert_eq!(catalog.shared.len(), 0);
    }
}
