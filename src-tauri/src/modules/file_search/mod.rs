//! Fast file and folder search over an index of the drives, kept up to date in the background.

pub mod commands;
mod config;
mod index;
mod scan;
mod store;
mod watch;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex, MutexGuard, PoisonError, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use notify::RecommendedWatcher;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use self::{
    config::{Config, Exclusions},
    index::{Index, Item},
};
use crate::{
    app::{Result, autostart, storage},
    services::text::{fold, words},
};

pub const ID: &str = "file-search";
/// A start at login waits before scanning, to stay out of the way while Windows settles.
const DELAY_AT_LOGIN: Duration = Duration::from_secs(30);
const DELAY: Duration = Duration::from_secs(3);
/// A full scan also runs regularly, which drops removed files from the index.
const RESCAN_EVERY: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHit {
    pub path: String,
    pub name: String,
    pub location: String,
    pub is_dir: bool,
    pub size: u64,
    /// Seconds since the Unix epoch.
    pub modified: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The fixed drives, always indexed.
    pub drives: Vec<String>,
    /// Folders added by the user.
    pub folders: Vec<String>,
    pub exclusions: Vec<String>,
}

pub struct FileSearch {
    config_path: PathBuf,
    cache_path: PathBuf,
    config: Mutex<Config>,
    index: RwLock<Index>,
    /// Folded folder paths to their entry, to place new files.
    folders: Mutex<HashMap<String, u32>>,
    watcher: Mutex<Option<RecommendedWatcher>>,
    /// Bumped when the module stops, so its background thread ends.
    generation: AtomicU64,
    running: AtomicBool,
    cancel: AtomicBool,
    rescan: AtomicBool,
}

impl FileSearch {
    fn config(&self) -> MutexGuard<'_, Config> {
        self.config.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn folders(&self) -> MutexGuard<'_, HashMap<String, u32>> {
        self.folders.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn replace_index(&self, index: Index) {
        let folders = (0..u32::try_from(index.len()).unwrap_or(u32::MAX))
            .filter(|&at| index.entry(at).is_some_and(index::Entry::is_dir))
            .map(|at| (key(&index.path(at)), at))
            .collect();
        *self.folders() = folders;
        *self.index.write().unwrap_or_else(PoisonError::into_inner) = index;
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<FileHit> {
        let words = words(query);
        let index = self.index.read().unwrap_or_else(PoisonError::into_inner);
        index
            .search(&words, limit * 3)
            .into_iter()
            .filter_map(|hit| {
                let path = index.path(hit.index);
                fs::symlink_metadata(&path).ok()?;
                let entry = index.entry(hit.index)?;
                Some(FileHit {
                    name: index.name(hit.index).to_owned(),
                    location: index::location(&path),
                    path: path.to_string_lossy().into_owned(),
                    is_dir: entry.is_dir(),
                    size: entry.size,
                    modified: entry.modified,
                })
            })
            .take(limit)
            .collect()
    }

    pub fn status(&self) -> Status {
        let config = self.config().clone();
        Status {
            drives: config::drives()
                .iter()
                .map(|drive| drive.to_string_lossy().into_owned())
                .collect(),
            folders: config.folders,
            exclusions: config.exclusions,
        }
    }

    fn update_config(&self, change: impl FnOnce(&mut Config)) -> Result<()> {
        let mut config = self.config();
        change(&mut config);
        storage::write_json(&self.config_path, &*config)?;
        drop(config);
        self.rescan.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Places files that just appeared under their folder, if it is indexed.
    fn add(&self, paths: Vec<PathBuf>) {
        let exclusions = Exclusions::new(&self.config().exclusions);
        for path in paths {
            let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
                continue;
            };
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink()
                || exclusions.excludes(parent)
                || (meta.is_dir() && exclusions.excludes(&path))
            {
                continue;
            }
            let Some(&parent) = self.folders().get(&key(parent)) else {
                continue;
            };
            let at = self
                .index
                .write()
                .unwrap_or_else(PoisonError::into_inner)
                .push(&Item {
                    name: &name.to_string_lossy(),
                    parent,
                    is_dir: meta.is_dir(),
                    size: if meta.is_dir() { 0 } else { meta.len() },
                    modified: scan::modified(&meta),
                });
            if meta.is_dir() {
                self.folders().insert(key(&path), at);
            }
        }
    }
}

fn key(path: &Path) -> String {
    fold(&path.to_string_lossy())
}

pub fn setup(app: &AppHandle) -> Result<()> {
    let config_path = app.path().app_config_dir()?.join("file-search.json");
    let cache_path = app.path().app_local_data_dir()?.join("index.bin");
    app.manage(FileSearch {
        config: Mutex::new(storage::read_json(&config_path).unwrap_or_default()),
        config_path,
        cache_path,
        index: RwLock::default(),
        folders: Mutex::default(),
        watcher: Mutex::default(),
        generation: AtomicU64::new(0),
        running: AtomicBool::new(false),
        cancel: AtomicBool::new(false),
        rescan: AtomicBool::new(false),
    });
    Ok(())
}

/// Starts the index when the module is on, and frees it when it is turned off.
pub fn set_enabled(app: &AppHandle, enabled: bool) {
    let search = app.state::<FileSearch>();
    if enabled {
        if !search.running.swap(true, Ordering::SeqCst) {
            search.cancel.store(false, Ordering::SeqCst);
            let generation = search.generation.fetch_add(1, Ordering::SeqCst) + 1;
            let app = app.clone();
            thread::Builder::new()
                .name("file-index".into())
                .spawn(move || keep_up_to_date(&app, generation))
                .inspect_err(|error| tracing::error!(%error, "could not start the file index"))
                .ok();
        }
    } else if search.running.swap(false, Ordering::SeqCst) {
        search.generation.fetch_add(1, Ordering::SeqCst);
        search.cancel.store(true, Ordering::SeqCst);
        *search
            .watcher
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = None;
        search.replace_index(Index::default());
    }
}

fn keep_up_to_date(app: &AppHandle, generation: u64) {
    let search = app.state::<FileSearch>();
    let current = || search.generation.load(Ordering::SeqCst) == generation;
    if let Some(cached) = store::load(&search.cache_path) {
        search.replace_index(cached);
    }
    let delay = if autostart::launched_at_startup() {
        DELAY_AT_LOGIN
    } else {
        DELAY
    };
    if !wait(delay, current, || false) {
        return;
    }
    loop {
        let config = search.config().clone();
        let roots = config::roots(&config);
        let started = Instant::now();
        let scanned = scan::scan(&roots, &Exclusions::new(&config.exclusions), &search.cancel);
        let Some(index) = scanned.filter(|_| current()) else {
            return;
        };
        tracing::info!(
            entries = index.len(),
            seconds = started.elapsed().as_secs(),
            "file index ready"
        );
        if let Err(error) = store::save(&search.cache_path, &index) {
            tracing::warn!(%error, "could not save the file index");
        }
        search.replace_index(index);
        let handle = app.clone();
        *search
            .watcher
            .lock()
            .unwrap_or_else(PoisonError::into_inner) =
            watch::start(&roots, move |paths| handle.state::<FileSearch>().add(paths));
        if !wait(RESCAN_EVERY, current, || {
            search.rescan.swap(false, Ordering::Relaxed)
        }) && !current()
        {
            return;
        }
    }
}

/// Sleeps up to `duration`. Returns `true` when the time is up or `wake` asks for it, `false` when
/// `current` says the module stopped.
fn wait(duration: Duration, current: impl Fn() -> bool, wake: impl Fn() -> bool) -> bool {
    let end = Instant::now() + duration;
    while Instant::now() < end {
        if !current() {
            return false;
        }
        if wake() {
            return true;
        }
        thread::sleep(Duration::from_millis(500));
    }
    current()
}
