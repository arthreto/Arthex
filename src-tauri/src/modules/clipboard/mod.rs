//! Clipboard history: text only, kept in memory and never written to disk or to the logs.

pub mod commands;
#[cfg(windows)]
mod watch;

use std::{
    sync::{
        Mutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};

pub const ID: &str = "clipboard";
/// Sent when the history changes, so the page and the Home card refresh.
#[cfg_attr(not(windows), allow(dead_code))]
pub const CHANGED: &str = "clipboard://changed";
/// Entries kept besides the pinned ones.
const MAX: usize = 50;
/// Larger copies (a whole document, a log) are not kept.
const MAX_BYTES: usize = 100 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    }

    pub fn remove(&self, id: u64) {
        self.history().clips.retain(|clip| clip.id != id);
    }

    /// Clears the history but keeps what is pinned.
    pub fn clear(&self) {
        self.history().clips.retain(|clip| clip.pinned);
    }
}

pub fn setup(app: &AppHandle) {
    app.manage(Clipboard {
        history: Mutex::default(),
        enabled: AtomicBool::new(false),
        paused: AtomicBool::new(false),
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
    }
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
