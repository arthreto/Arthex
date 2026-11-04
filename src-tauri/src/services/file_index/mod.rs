//! An index of every file and folder on the drives, kept up to date in the background. File
//! search and `FileSize` read it; it runs while either of them is on.

mod config;
mod index;
mod scan;
mod store;
mod usage;
mod watch;

use std::{
    collections::HashMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    sync::{
        Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use notify::RecommendedWatcher;
use tauri::{AppHandle, Emitter, Manager};

use self::{config::Exclusions, index::Item};
pub use self::{
    config::{Config, drives},
    index::{Index, location},
    usage::Category,
};
use crate::{
    app::{Result, autostart, storage},
    services::text::fold,
};
/// Sent when a scan finishes, so views built on the index refresh.
pub const UPDATED: &str = "file-index://updated";
/// A start at login waits before scanning, to stay out of the way while Windows settles.
const DELAY_AT_LOGIN: Duration = Duration::from_secs(30);
const DELAY: Duration = Duration::from_secs(3);
/// A full scan also runs regularly, which drops removed files from the index.
const RESCAN_EVERY: Duration = Duration::from_secs(6 * 60 * 60);

pub struct FileIndex {
    config_path: PathBuf,
    cache_path: PathBuf,
    config: Mutex<Config>,
    index: RwLock<Index>,
    /// Folders by `key` of their path, to find them and place new files.
    folders: Mutex<HashMap<u64, u32>>,
    watcher: Mutex<Option<RecommendedWatcher>>,
    /// Bumped when the module stops, so its background thread ends.
    generation: AtomicU64,
    running: AtomicBool,
    cancel: AtomicBool,
    rescan: AtomicBool,
}

impl FileIndex {
    fn config(&self) -> MutexGuard<'_, Config> {
        self.config.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn folders(&self) -> MutexGuard<'_, HashMap<u64, u32>> {
        self.folders.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn read(&self) -> RwLockReadGuard<'_, Index> {
        self.index.read().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn current_config(&self) -> Config {
        self.config().clone()
    }

    /// The entry of an indexed path.
    pub fn find(&self, path: &Path) -> Option<u32> {
        let folder = self.folders().get(&key(path)).copied();
        if let Some(folder) = folder.filter(|&at| key_matches(&self.read(), at, path)) {
            return Some(folder);
        }
        let parent_path = path.parent()?;
        let parent = *self.folders().get(&key(parent_path))?;
        let name = fold(&path.file_name()?.to_string_lossy());
        let index = self.read();
        if !key_matches(&index, parent, parent_path) {
            return None;
        }
        index
            .entries()
            .iter()
            .enumerate()
            .skip(parent as usize + 1)
            .find(|(_, entry)| {
                entry.parent == parent
                    && !entry.is_removed()
                    && index.folded_name(entry) == name.as_bytes()
            })
            .and_then(|(position, _)| u32::try_from(position).ok())
    }

    /// Hides an entry sent to the Recycle Bin until the next scan.
    pub fn remove(&self, index: u32) {
        self.index
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(index);
    }

    /// Asks the background thread for a full scan now.
    pub fn rescan(&self) {
        self.rescan.store(true, Ordering::Relaxed);
    }

    fn replace_index(&self, index: Index) {
        let folders = (0..u32::try_from(index.len()).unwrap_or(u32::MAX))
            .filter(|&at| {
                index
                    .entry(at)
                    .is_some_and(|entry| entry.is_dir() && !entry.is_summary())
            })
            .map(|at| (key(&index.path(at)), at))
            .collect();
        *self.folders() = folders;
        *self.index.write().unwrap_or_else(PoisonError::into_inner) = index;
    }

    pub fn update_config(&self, change: impl FnOnce(&mut Config)) -> Result<()> {
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
                    summary: false,
                    size: if meta.is_dir() { 0 } else { meta.len() },
                    modified: scan::modified(&meta),
                });
            if meta.is_dir() {
                self.folders().insert(key(&path), at);
            }
        }
    }
}

/// Folders are found by a hash of their folded path, which takes far less memory than the
/// paths themselves; `find` checks the path, so a collision can never return the wrong entry.
fn key(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    fold(&path.to_string_lossy()).hash(&mut hasher);
    hasher.finish()
}

fn key_matches(index: &Index, at: u32, path: &Path) -> bool {
    fold(&index.path(at).to_string_lossy()) == fold(&path.to_string_lossy())
}

pub fn setup(app: &AppHandle) -> Result<()> {
    let config_path = app.path().app_config_dir()?.join("file-search.json");
    let cache_path = app.path().app_local_data_dir()?.join("index.bin");
    app.manage(FileIndex {
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

/// Starts the index when a module needs it, and frees it when none does.
pub fn set_wanted(app: &AppHandle, wanted: bool) {
    let files = app.state::<FileIndex>();
    if wanted {
        if !files.running.swap(true, Ordering::SeqCst) {
            files.cancel.store(false, Ordering::SeqCst);
            let generation = files.generation.fetch_add(1, Ordering::SeqCst) + 1;
            let app = app.clone();
            thread::Builder::new()
                .name("file-index".into())
                .spawn(move || keep_up_to_date(&app, generation))
                .inspect_err(|error| tracing::error!(%error, "could not start the file index"))
                .ok();
        }
    } else if files.running.swap(false, Ordering::SeqCst) {
        files.generation.fetch_add(1, Ordering::SeqCst);
        files.cancel.store(true, Ordering::SeqCst);
        *files.watcher.lock().unwrap_or_else(PoisonError::into_inner) = None;
        files.replace_index(Index::default());
    }
}

fn keep_up_to_date(app: &AppHandle, generation: u64) {
    let files = app.state::<FileIndex>();
    let current = || files.generation.load(Ordering::SeqCst) == generation;
    if let Some(cached) = store::load(&files.cache_path) {
        files.replace_index(cached);
        app.emit(UPDATED, ()).ok();
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
        let config = files.config().clone();
        let roots = config::roots(&config);
        let started = Instant::now();
        let scanned = scan::scan(&roots, &Exclusions::new(&config.exclusions), &files.cancel);
        let Some(index) = scanned.filter(|_| current()) else {
            return;
        };
        tracing::info!(
            entries = index.len(),
            seconds = started.elapsed().as_secs(),
            "file index ready"
        );
        if let Err(error) = store::save(&files.cache_path, &index) {
            tracing::warn!(%error, "could not save the file index");
        }
        files.replace_index(index);
        app.emit(UPDATED, ()).ok();
        let handle = app.clone();
        *files.watcher.lock().unwrap_or_else(PoisonError::into_inner) =
            watch::start(&roots, move |paths| handle.state::<FileIndex>().add(paths));
        if !wait(RESCAN_EVERY, current, || {
            files.rescan.swap(false, Ordering::Relaxed)
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
