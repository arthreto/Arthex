//! An index of every file and folder on the drives, kept up to date in the background. File
//! search and `FileSize` read it; it runs while either of them is on.

mod config;
mod index;
mod scan;
mod store;
mod usage;
mod watch;

use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    sync::{
        Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use notify::RecommendedWatcher;
use tauri::{AppHandle, Emitter, Manager};

use serde::Serialize;

#[cfg(windows)]
pub use self::config::space;
use self::{config::Exclusions, index::Item, scan::Progress};
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
/// Sent while the first scan runs, with a `Building`, then once with `null` when it ends.
pub const BUILDING: &str = "file-index://building";
const BUILDING_EVERY: Duration = Duration::from_millis(500);
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
    /// Set while there is no index to search yet: no saved one, first scan pending or running.
    building: AtomicBool,
    progress: Progress,
    /// Bytes used on the indexed drives, which the first scan is expected to go through.
    expected: AtomicU64,
}

/// How far the first scan has gone.
#[derive(Debug, Clone, Serialize)]
pub struct Building {
    pub items: u64,
    /// An estimate, from the bytes gone through; `None` when the drives' usage is unknown.
    pub percent: Option<u8>,
}

impl FileIndex {
    pub fn building(&self) -> Option<Building> {
        if !self.building.load(Ordering::SeqCst) {
            return None;
        }
        Some(Building {
            items: self.progress.items.load(Ordering::Relaxed),
            percent: estimate(
                self.progress.bytes.load(Ordering::Relaxed),
                self.expected.load(Ordering::Relaxed),
            ),
        })
    }

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
        self.find_in(&self.read(), path)
    }

    /// Whether an indexed item may go to the Recycle Bin: `None` when it is not indexed.
    /// Finding and checking under one lock keeps a scan from swapping the entry in between.
    pub fn may_remove(&self, path: &Path) -> Option<bool> {
        let index = self.read();
        let at = self.find_in(&index, path)?;
        Some(!index.is_protected(at))
    }

    /// Hides an item sent to the Recycle Bin, and everything inside it, until the next scan.
    pub fn remove(&self, path: &Path) {
        let mut index = self.index.write().unwrap_or_else(PoisonError::into_inner);
        if let Some(at) = self.find_in(&index, path) {
            index.remove(at);
        }
    }

    /// Whether the user's exclusions leave this folder out of the index.
    pub fn excludes(&self, path: &Path) -> bool {
        Exclusions::new(&self.config().exclusions).excludes(path)
    }

    /// Whether a folder lies under one of the indexed roots.
    pub fn covers(&self, path: &Path) -> bool {
        config::roots(&self.config())
            .iter()
            .any(|root| path.starts_with(root))
    }

    fn find_in(&self, index: &Index, path: &Path) -> Option<u32> {
        let folder = self.folders().get(&key(path)).copied();
        if let Some(folder) = folder.filter(|&at| same_path(index, at, path)) {
            return Some(folder);
        }
        let parent_path = path.parent()?;
        let parent = self.folders().get(&key(parent_path)).copied()?;
        if !same_path(index, parent, parent_path) {
            return None;
        }
        let name = path.file_name()?.to_string_lossy().to_lowercase();
        index
            .entries()
            .iter()
            .enumerate()
            .skip(parent as usize + 1)
            .find(|&(position, entry)| {
                entry.parent == parent
                    && !entry.is_removed()
                    && u32::try_from(position).is_ok_and(|at| index.name(at).to_lowercase() == name)
            })
            .and_then(|(position, _)| u32::try_from(position).ok())
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

    fn write(&self) -> RwLockWriteGuard<'_, Index> {
        self.index.write().unwrap_or_else(PoisonError::into_inner)
    }

    /// Brings changed paths up to date: what still exists is added or refreshed, what is gone
    /// is hidden. One pass over the index serves the whole batch, so an editor that saves
    /// through a temporary file never leaves duplicates.
    fn apply(&self, mut paths: Vec<PathBuf>) {
        let exclusions = Exclusions::new(&self.config().exclusions);
        // Parents before their content, so a new folder is placed (and scanned) first.
        paths.sort_by_key(|path| path.components().count());
        let known = self.known(&paths);
        let mut scanned: Vec<PathBuf> = Vec::new();
        for path in paths {
            if scanned.iter().any(|folder| path.starts_with(folder)) {
                continue;
            }
            let meta = fs::symlink_metadata(&path)
                .ok()
                .filter(|meta| !meta.file_type().is_symlink());
            match (known.get(&path), meta) {
                (Some(&at), Some(meta)) if meta.is_file() => {
                    self.write().update(at, meta.len(), scan::modified(&meta));
                }
                (Some(&at), None) => self.write().remove(at),
                (None, Some(meta)) => {
                    let placed = self.place(&path, &meta, &exclusions);
                    if placed && meta.is_dir() {
                        scanned.push(path);
                    }
                }
                _ => {}
            }
        }
    }

    /// The live entries of the given paths, found in one pass.
    fn known(&self, paths: &[PathBuf]) -> HashMap<PathBuf, u32> {
        let mut wanted: HashMap<(u32, String), &PathBuf> = HashMap::new();
        for path in paths {
            if let (Some(parent), Some(name)) = (path.parent(), path.file_name())
                && let Some(parent) = self.folders().get(&key(parent)).copied()
            {
                wanted.insert((parent, name.to_string_lossy().to_lowercase()), path);
            }
        }
        let parents: HashSet<u32> = wanted.keys().map(|(parent, _)| *parent).collect();
        let index = self.read();
        let mut found = HashMap::new();
        for (position, entry) in index.entries().iter().enumerate() {
            if entry.is_removed() || !parents.contains(&entry.parent) {
                continue;
            }
            let Ok(at) = u32::try_from(position) else {
                break;
            };
            if let Some(path) = wanted.get(&(entry.parent, index.name(at).to_lowercase())) {
                found.insert((*path).clone(), at);
            }
        }
        found
    }

    /// Adds a new file, or a new folder with its content, under its indexed parent.
    fn place(&self, path: &Path, meta: &fs::Metadata, exclusions: &Exclusions) -> bool {
        let (Some(parent_path), Some(name)) = (path.parent(), path.file_name()) else {
            return false;
        };
        let Some(parent) = self.folders().get(&key(parent_path)).copied() else {
            return false;
        };
        let name = name.to_string_lossy();
        if !meta.is_dir() {
            self.write().push(&Item {
                name: &name,
                parent,
                is_dir: false,
                summary: false,
                size: meta.len(),
                modified: scan::modified(meta),
            });
            return true;
        }
        let part = scan::subtree(path, &name, exclusions);
        let start = {
            let mut index = self.write();
            let start = index.len();
            index.append(part, parent);
            start
        };
        let index = self.read();
        for at in start..index.len() {
            let Ok(at) = u32::try_from(at) else {
                break;
            };
            if index
                .entry(at)
                .is_some_and(|entry| entry.is_dir() && !entry.is_summary())
            {
                self.folders().insert(key(&index.path(at)), at);
            }
        }
        true
    }
}

/// Folders are found by a hash of their folded path, which takes far less memory than the
/// paths themselves; lookups check the path, so a collision can never return the wrong entry.
fn key(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    fold(&path.to_string_lossy()).hash(&mut hasher);
    hasher.finish()
}

/// Windows paths ignore case but not accents: "été" and "ete" are two files.
fn same_path(index: &Index, at: u32, path: &Path) -> bool {
    index.path(at).to_string_lossy().to_lowercase() == path.to_string_lossy().to_lowercase()
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
        building: AtomicBool::new(false),
        progress: Progress::default(),
        expected: AtomicU64::new(0),
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
        files.building.store(false, Ordering::SeqCst);
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
    } else {
        files.progress.reset();
        files.building.store(true, Ordering::SeqCst);
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
        if files.building.load(Ordering::SeqCst) {
            files.progress.reset();
            files.expected.store(used_space(), Ordering::Relaxed);
            report_building(app, generation);
        }
        let scanned = scan::scan(
            &roots,
            &Exclusions::new(&config.exclusions),
            &files.cancel,
            &files.progress,
        );
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
        // Turning the module off during the save must not bring the index and watcher back.
        if !current() {
            return;
        }
        files.replace_index(index);
        files.building.store(false, Ordering::SeqCst);
        app.emit(UPDATED, ()).ok();
        let handle = app.clone();
        if !current() {
            return;
        }
        *files.watcher.lock().unwrap_or_else(PoisonError::into_inner) =
            watch::start(&roots, move |paths| {
                handle.state::<FileIndex>().apply(paths);
            });
        if !wait(RESCAN_EVERY, current, || {
            files.rescan.swap(false, Ordering::Relaxed)
        }) && !current()
        {
            return;
        }
    }
}

/// Never 100 before the end: the estimate cannot know what the scan will miss or count twice.
fn estimate(bytes: u64, expected: u64) -> Option<u8> {
    (expected > 0).then(|| {
        u8::try_from((u128::from(bytes) * 100 / u128::from(expected)).min(99)).unwrap_or(99)
    })
}

/// Sends the progress of the first scan until it ends or the module stops.
fn report_building(app: &AppHandle, generation: u64) {
    let app = app.clone();
    thread::Builder::new()
        .name("file-index-progress".into())
        .spawn(move || {
            let files = app.state::<FileIndex>();
            while files.generation.load(Ordering::SeqCst) == generation {
                // The last event says `null`, so the interface never keeps a stale progress.
                let building = files.building();
                app.emit(BUILDING, &building).ok();
                if building.is_none() {
                    break;
                }
                thread::sleep(BUILDING_EVERY);
            }
        })
        .inspect_err(|error| tracing::warn!(%error, "could not report the file index progress"))
        .ok();
}

#[cfg(windows)]
fn used_space() -> u64 {
    drives()
        .into_iter()
        .filter_map(|drive| space(&drive))
        .map(|(total, free)| total.saturating_sub(free))
        .sum()
}

#[cfg(not(windows))]
fn used_space() -> u64 {
    0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_the_first_scan_without_reaching_the_end_early() {
        assert_eq!(estimate(0, 0), None);
        assert_eq!(estimate(50, 200), Some(25));
        assert_eq!(estimate(300, 200), Some(99));
        assert_eq!(estimate(u64::MAX, u64::MAX), Some(99));
    }

    fn indexed(root: &Path) -> FileIndex {
        let config = Config {
            folders: Vec::new(),
            exclusions: Vec::new(),
        };
        let files = FileIndex {
            config_path: root.join("config.json"),
            cache_path: root.join("index.bin"),
            config: Mutex::new(config),
            index: RwLock::default(),
            folders: Mutex::default(),
            watcher: Mutex::default(),
            generation: AtomicU64::new(0),
            running: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            rescan: AtomicBool::new(false),
            building: AtomicBool::new(false),
            progress: Progress::default(),
            expected: AtomicU64::new(0),
        };
        let index = scan::scan(
            &[root.to_path_buf()],
            &Exclusions::new(&[]),
            &AtomicBool::new(false),
            &Progress::default(),
        )
        .unwrap();
        files.replace_index(index);
        files
    }

    fn live(files: &FileIndex, name: &str) -> Vec<u64> {
        let index = files.read();
        index
            .entries()
            .iter()
            .enumerate()
            .filter(|(at, entry)| {
                !entry.is_removed() && index.name(u32::try_from(*at).unwrap()) == name
            })
            .map(|(_, entry)| entry.size)
            .collect()
    }

    #[test]
    fn follows_saves_deletions_and_new_folders_without_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("report.docx"), b"12").unwrap();
        let files = indexed(root);

        // An editor saving through a temporary file.
        fs::write(root.join("report.tmp"), b"12345").unwrap();
        fs::rename(root.join("report.tmp"), root.join("report.docx")).unwrap();
        files.apply(vec![root.join("report.tmp"), root.join("report.docx")]);
        assert_eq!(live(&files, "report.docx"), [5]);
        assert_eq!(live(&files, "report.tmp"), Vec::<u64>::new());

        fs::create_dir_all(root.join("new/inner")).unwrap();
        fs::write(root.join("new/inner/photo.jpg"), b"123").unwrap();
        files.apply(vec![
            root.join("new/inner/photo.jpg"),
            root.join("new"),
            root.join("new/inner"),
        ]);
        assert_eq!(live(&files, "photo.jpg"), [3]);
        assert_eq!(live(&files, "inner").len(), 1);

        fs::remove_file(root.join("report.docx")).unwrap();
        files.apply(vec![root.join("report.docx")]);
        assert_eq!(live(&files, "report.docx"), Vec::<u64>::new());
    }
}
