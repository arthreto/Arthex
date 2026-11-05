//! The apps used last, recorded from the foreground window on Windows.

pub mod commands;
#[cfg(windows)]
mod watch;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Mutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::app::{Result, storage};

pub const ID: &str = "recent-apps";
const MAX: usize = 30;

/// What is saved to `recent.json`: never window titles or content, only the program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub path: String,
    pub name: String,
    /// Milliseconds since the Unix epoch.
    pub last_used: u64,
}

#[derive(Debug, Default)]
struct History(Vec<Entry>);

impl History {
    /// Moves the app to the top. Returns whether the order changed, which is when it is saved.
    fn record(&mut self, path: &str, name: &str, now: u64) -> bool {
        if let Some(first) = self.0.first_mut()
            && first.path.eq_ignore_ascii_case(path)
        {
            first.last_used = now;
            return false;
        }
        self.0
            .retain(|entry| !entry.path.eq_ignore_ascii_case(path));
        self.0.insert(
            0,
            Entry {
                path: path.to_owned(),
                name: name.to_owned(),
                last_used: now,
            },
        );
        self.0.truncate(MAX);
        true
    }
}

pub struct RecentApps {
    path: PathBuf,
    history: Mutex<History>,
    /// Last window of each app (by path, lower case), to bring it back instead of starting it again.
    windows: Mutex<HashMap<String, isize>>,
    enabled: AtomicBool,
}

impl RecentApps {
    fn load(path: PathBuf) -> Self {
        let mut entries: Vec<Entry> = storage::read_json(&path).unwrap_or_default();
        entries.truncate(MAX);
        for entry in &mut entries {
            entry.name = saved_name(&entry.name);
        }
        Self {
            path,
            history: Mutex::new(History(entries)),
            windows: Mutex::default(),
            enabled: AtomicBool::new(false),
        }
    }

    fn history(&self) -> MutexGuard<'_, History> {
        self.history.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn windows(&self) -> MutexGuard<'_, HashMap<String, isize>> {
        self.windows.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn entries(&self) -> Vec<Entry> {
        self.history().0.clone()
    }

    pub fn window_of(&self, path: &str) -> Option<isize> {
        self.windows().get(&path.to_lowercase()).copied()
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    fn record(&self, path: &str, name: &str, window: isize) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        self.windows().insert(path.to_lowercase(), window);
        let mut history = self.history();
        if history.record(path, name, now())
            && let Err(error) = storage::write_json(&self.path, &history.0)
        {
            tracing::warn!(%error, "could not save recent apps");
        }
    }

    pub fn clear(&self) -> Result<()> {
        self.windows().clear();
        let mut history = self.history();
        history.0.clear();
        storage::write_json(&self.path, &history.0)
    }
}

/// Older versions could save what followed the end of a program's name, after a NUL.
fn saved_name(name: &str) -> String {
    name.split('\0').next().unwrap_or(name).trim().to_owned()
}

pub fn setup(app: &AppHandle) -> Result<()> {
    let path = app.path().app_data_dir()?.join("recent.json");
    app.manage(RecentApps::load(path));
    #[cfg(windows)]
    {
        let handle = app.clone();
        watch::start(move |foreground| {
            handle.state::<RecentApps>().record(
                &foreground.path,
                &foreground.name,
                foreground.window,
            );
        });
    }
    Ok(())
}

/// Records only while the module is enabled; the hook itself costs nothing while ignored.
pub fn set_enabled(app: &AppHandle, enabled: bool) {
    app.state::<RecentApps>()
        .enabled
        .store(enabled, Ordering::Relaxed);
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(history: &History) -> Vec<&str> {
        history.0.iter().map(|entry| entry.path.as_str()).collect()
    }

    #[test]
    fn a_saved_name_loses_what_follows_a_nul() {
        assert_eq!(
            saved_name("NVIDIA App\0\08\u{18}\u{1}FileVer"),
            "NVIDIA App"
        );
        assert_eq!(saved_name("Claude"), "Claude");
    }

    #[test]
    fn moves_the_last_app_to_the_top_without_duplicates() {
        let mut history = History::default();
        assert!(history.record(r"C:\a.exe", "A", 1));
        assert!(history.record(r"C:\b.exe", "B", 2));
        assert!(history.record(r"C:\A.EXE", "A", 3));
        assert_eq!(paths(&history), [r"C:\A.EXE", r"C:\b.exe"]);
    }

    #[test]
    fn staying_on_the_same_app_only_refreshes_its_time() {
        let mut history = History::default();
        history.record(r"C:\a.exe", "A", 1);
        assert!(!history.record(r"C:\a.exe", "A", 9));
        assert_eq!(history.0[0].last_used, 9);
    }

    #[test]
    fn keeps_the_thirty_latest() {
        let mut history = History::default();
        for index in 0..40 {
            history.record(&format!(r"C:\{index}.exe"), "x", index);
        }
        assert_eq!(history.0.len(), MAX);
        assert_eq!(history.0[0].path, r"C:\39.exe");
    }
}
