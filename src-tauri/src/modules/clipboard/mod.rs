//! Clipboard history: text only, never written to the logs. It stays in memory, unless the user
//! asks to keep it across restarts.

pub mod commands;
mod saved;
#[cfg(windows)]
mod watch;

use std::{
    path::PathBuf,
    sync::{
        Mutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

pub const ID: &str = "clipboard";
/// Sent when the history changes, so the page and the Home card refresh.
#[cfg_attr(not(windows), allow(dead_code))]
pub const CHANGED: &str = "clipboard://changed";
/// Entries kept besides the pinned ones.
const MAX: usize = 50;
/// Larger copies (a whole document, a log) are not kept.
const MAX_BYTES: usize = 100 * 1024;
const SAVED_FILE: &str = "clipboard.bin";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: u64,
    pub text: String,
    pub pinned: bool,
    /// Milliseconds since the Unix epoch.
    pub copied_at: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    /// Pinned first, then the latest.
    pub clips: Vec<Clip>,
    pub paused: bool,
}

#[derive(Debug, Default)]
struct History {
    clips: Vec<Clip>,
    next_id: u64,
}

impl History {
    /// Puts the text on top; copying it again moves it up instead of repeating it.
    fn add(&mut self, text: String, now: u64) {
        let pinned = match self.clips.iter().position(|clip| clip.text == text) {
            Some(position) => self.clips.remove(position).pinned,
            None => false,
        };
        self.next_id += 1;
        self.clips.insert(
            0,
            Clip {
                id: self.next_id,
                text,
                pinned,
                copied_at: now,
            },
        );
        let mut kept = 0;
        self.clips.retain(|clip| {
            kept += usize::from(!clip.pinned);
            clip.pinned || kept <= MAX
        });
    }

    /// Only into an empty history, at launch: ids go on from the saved ones.
    fn restore(&mut self, clips: Vec<Clip>) {
        self.next_id = clips.iter().map(|clip| clip.id).max().unwrap_or(0);
        self.clips = clips;
    }

    fn sorted(&self) -> Vec<Clip> {
        let mut clips = self.clips.clone();
        clips.sort_by_key(|clip| !clip.pinned);
        clips
    }
}

pub struct Clipboard {
    history: Mutex<History>,
    enabled: AtomicBool,
    paused: AtomicBool,
    keep: AtomicBool,
    saved_file: Option<PathBuf>,
}

impl Clipboard {
    fn history(&self) -> MutexGuard<'_, History> {
        self.history.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether a new copy may be read at all: nothing is read while off or paused.
    fn listening(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) && !self.paused.load(Ordering::Relaxed)
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    fn record(&self, text: String) -> bool {
        if !self.listening() || text.trim().is_empty() || text.len() > MAX_BYTES {
            return false;
        }
        self.history().add(text, now());
        self.save();
        true
    }

    pub fn state(&self) -> State {
        State {
            clips: self.history().sorted(),
            paused: self.paused.load(Ordering::Relaxed),
        }
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }

    pub fn pin(&self, id: u64, pinned: bool) {
        if let Some(clip) = self.history().clips.iter_mut().find(|clip| clip.id == id) {
            clip.pinned = pinned;
        }
        self.save();
    }

    pub fn remove(&self, id: u64) {
        self.history().clips.retain(|clip| clip.id != id);
        self.save();
    }

    /// Clears the history but keeps what is pinned.
    pub fn clear(&self) {
        self.history().clips.retain(|clip| clip.pinned);
        self.save();
    }

    fn save(&self) {
        if !self.keep.load(Ordering::Relaxed) {
            return;
        }
        let Some(file) = &self.saved_file else {
            return;
        };
        let clips = self.history().clips.clone();
        if let Err(error) = saved::save(file, &clips) {
            tracing::warn!(%error, "could not save the clipboard history");
        }
    }

    fn delete_saved(&self) {
        if let Some(file) = &self.saved_file {
            saved::delete(file);
        }
    }
}

pub fn setup(app: &AppHandle) {
    app.manage(Clipboard {
        history: Mutex::default(),
        enabled: AtomicBool::new(false),
        paused: AtomicBool::new(false),
        keep: AtomicBool::new(false),
        saved_file: app
            .path()
            .app_data_dir()
            .ok()
            .map(|dir| dir.join(SAVED_FILE)),
    });
    #[cfg(windows)]
    {
        use tauri::Emitter;

        let reader = app.clone();
        let handle = app.clone();
        watch::start(
            move || reader.state::<Clipboard>().listening(),
            move |text| {
                if handle.state::<Clipboard>().record(text) {
                    handle.emit(CHANGED, ()).ok();
                }
            },
        );
    }
}

/// Turning the module off forgets everything, pinned entries included.
pub fn set_enabled(app: &AppHandle, enabled: bool) {
    let clipboard = app.state::<Clipboard>();
    clipboard.enabled.store(enabled, Ordering::Relaxed);
    if !enabled {
        clipboard.history().clips.clear();
        clipboard.delete_saved();
    }
}

/// Turning it on brings back what was saved, which is how the history survives a restart. Turning
/// it off leaves nothing on disk.
pub fn set_keep(app: &AppHandle, keep: bool) {
    let clipboard = app.state::<Clipboard>();
    if clipboard.keep.swap(keep, Ordering::Relaxed) == keep {
        return;
    }
    if !keep {
        clipboard.delete_saved();
        return;
    }
    let saved = clipboard.saved_file.as_deref().and_then(saved::load);
    if let Some(clips) = saved {
        let mut history = clipboard.history();
        if history.clips.is_empty() {
            history.restore(clips);
        }
    }
    clipboard.save();
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

    fn texts(history: &History) -> Vec<&str> {
        history
            .clips
            .iter()
            .map(|clip| clip.text.as_str())
            .collect()
    }

    #[test]
    fn moves_a_repeated_copy_to_the_top_and_keeps_its_pin() {
        let mut history = History::default();
        history.add("a".into(), 1);
        history.add("b".into(), 2);
        history.clips[1].pinned = true;
        history.add("a".into(), 3);
        assert_eq!(texts(&history), ["a", "b"]);
        assert!(history.clips[0].pinned);
        assert_eq!(history.clips[0].copied_at, 3);
    }

    #[test]
    fn numbers_new_copies_after_the_restored_ones() {
        let mut history = History::default();
        history.restore(vec![Clip {
            id: 7,
            text: "saved".into(),
            pinned: true,
            copied_at: 1,
        }]);
        history.add("new".into(), 2);
        assert_eq!(texts(&history), ["new", "saved"]);
        assert_eq!(history.clips[0].id, 8);
    }

    #[test]
    fn keeps_the_latest_fifty_besides_pinned_ones() {
        let mut history = History::default();
        history.add("pinned".into(), 0);
        history.clips[0].pinned = true;
        for index in 0..60 {
            history.add(index.to_string(), index);
        }
        assert_eq!(history.clips.len(), MAX + 1);
        assert_eq!(history.clips[0].text, "59");
        assert!(history.clips.iter().any(|clip| clip.text == "pinned"));
        assert_eq!(history.sorted()[0].text, "pinned");
    }
}
