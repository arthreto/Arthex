//! Shuts down, restarts or sleeps at a set time, with a notice five minutes before.

use std::{
    sync::{
        Mutex, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use super::Command;
use crate::app::{self, AppError, Result, i18n::Translator};

pub const CHANGED: &str = "system://schedule";
const NOTICE_BEFORE_MS: u64 = 5 * 60 * 1000;
const MAX_AHEAD_MS: u64 = 7 * 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Shutdown,
    Restart,
    Sleep,
}

impl Action {
    fn command(self) -> Command {
        match self {
            Self::Shutdown => Command::Shutdown,
            Self::Restart => Command::Restart,
            Self::Sleep => Command::Sleep,
        }
    }

    fn notice_key(self) -> &'static str {
        match self {
            Self::Shutdown => "schedule.notice.shutdown",
            Self::Restart => "schedule.notice.restart",
            Self::Sleep => "schedule.notice.sleep",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub action: Action,
    /// Milliseconds since the Unix epoch.
    pub at: u64,
}

#[derive(Default)]
pub struct Planner {
    current: Mutex<Option<Schedule>>,
    /// Bumped on every change, so a waiting timer knows it was replaced or cancelled.
    generation: AtomicU64,
}

impl Planner {
    pub fn current(&self) -> Option<Schedule> {
        *self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn replace(&self, schedule: Option<Schedule>) -> u64 {
        *self.current.lock().unwrap_or_else(PoisonError::into_inner) = schedule;
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn is_current(&self, generation: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == generation
    }
}

pub fn setup(app: &AppHandle) {
    app.manage(Planner::default());
}

pub fn plan(app: &AppHandle, schedule: Schedule) -> Result<Schedule> {
    let now = now_ms();
    if schedule.at <= now || schedule.at > now + MAX_AHEAD_MS {
        return Err(AppError::Invalid(
            "the time must be within the next seven days".into(),
        ));
    }
    let generation = app.state::<Planner>().replace(Some(schedule));
    let _ = app.emit(CHANGED, Some(schedule));

    let app = app.clone();
    thread::spawn(move || {
        let mut noticed = schedule.at.saturating_sub(now) <= NOTICE_BEFORE_MS;
        loop {
            thread::sleep(Duration::from_secs(1));
            let planner = app.state::<Planner>();
            if !planner.is_current(generation) {
                return;
            }
            let left = schedule.at.saturating_sub(now_ms());
            if !noticed && left <= NOTICE_BEFORE_MS {
                noticed = true;
                notify(&app, schedule.action);
            }
            if left == 0 {
                planner.replace(None);
                let _ = app.emit(CHANGED, None::<Schedule>);
                if let Err(error) = super::run(&app, schedule.action.command()) {
                    tracing::warn!(%error, "the planned action failed");
                }
                return;
            }
        }
    });
    Ok(schedule)
}

pub fn cancel(app: &AppHandle) {
    app.state::<Planner>().replace(None);
    let _ = app.emit(CHANGED, None::<Schedule>);
}

fn notify(app: &AppHandle, action: Action) {
    let text = Translator::new(app::settings(app).language.as_deref());
    let shown = app
        .notification()
        .builder()
        .title("Arthex")
        .body(text.t(action.notice_key()))
        .show();
    if let Err(error) = shown {
        tracing::warn!(%error, "could not show the shutdown notice");
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
