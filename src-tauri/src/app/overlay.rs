use std::{
    sync::{
        Mutex, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use tauri::{AppHandle, Emitter, Manager, WebviewWindow, Window, WindowEvent};

use super::{events, hotkey};

pub const LABEL: &str = "main";

/// Windows may report a focus loss right after the window is shown, and clicking the tray icon
/// blurs the overlay just before the click arrives. Both are ignored within this delay.
const GRACE: Duration = Duration::from_millis(400);

#[derive(Default)]
struct Timeline {
    shown_at: Mutex<Option<Instant>>,
    hidden_at: Mutex<Option<Instant>>,
    /// Open system dialogs: they take the focus, which must not close the overlay.
    held: AtomicUsize,
}

impl Timeline {
    fn mark(slot: &Mutex<Option<Instant>>) {
        *slot.lock().unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
    }

    fn within_grace(slot: &Mutex<Option<Instant>>) -> bool {
        slot.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some_and(|at| at.elapsed() < GRACE)
    }
}

/// Keeps the overlay open while it lives, for system dialogs that take the focus.
pub struct Hold(AppHandle);

impl Drop for Hold {
    fn drop(&mut self) {
        if let Some(timeline) = self.0.try_state::<Timeline>() {
            timeline.held.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

pub fn hold(app: &AppHandle) -> Hold {
    if let Some(timeline) = app.try_state::<Timeline>() {
        timeline.held.fetch_add(1, Ordering::SeqCst);
    }
    Hold(app.clone())
}

pub fn prepare(app: &AppHandle) {
    app.manage(Timeline::default());
}

pub fn show(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    if let Some(timeline) = app.try_state::<Timeline>() {
        Timeline::mark(&timeline.shown_at);
    }
    if let Err(error) = present(&window) {
        tracing::warn!(%error, "could not show the overlay");
        return;
    }
    let _ = app.emit_to(LABEL, events::OVERLAY_SHOWN, ());
}

pub fn hide(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    // A recording left running would keep hiding every key press from the whole system.
    hotkey::record(false);
    if let Some(timeline) = app.try_state::<Timeline>() {
        Timeline::mark(&timeline.hidden_at);
    }
    if let Err(error) = window.hide() {
        tracing::warn!(%error, "could not hide the overlay");
    }
}

pub fn toggle(app: &AppHandle) {
    let visible = app
        .get_webview_window(LABEL)
        .is_some_and(|window| window.is_visible().unwrap_or(false));
    let just_hidden = app
        .try_state::<Timeline>()
        .is_some_and(|timeline| Timeline::within_grace(&timeline.hidden_at));
    if visible || just_hidden {
        hide(app);
    } else {
        show(app);
    }
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != LABEL {
        return;
    }
    let app = window.app_handle();
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            hide(app);
        }
        WindowEvent::Focused(false) => {
            let keep = app.try_state::<Timeline>().is_some_and(|timeline| {
                Timeline::within_grace(&timeline.shown_at)
                    || timeline.held.load(Ordering::SeqCst) > 0
            });
            if !keep {
                hide(app);
            }
        }
        _ => {}
    }
}

/// Covers the work area of the monitor under the cursor, then takes the focus.
fn present(window: &WebviewWindow) -> tauri::Result<()> {
    let cursor = window.cursor_position()?;
    let monitor = match window.monitor_from_point(cursor.x, cursor.y)? {
        Some(monitor) => Some(monitor),
        None => window.primary_monitor()?,
    };
    if let Some(monitor) = monitor {
        let area = monitor.work_area();
        window.set_position(area.position)?;
        window.set_size(area.size)?;
    }
    window.show()?;
    window.set_focus()
}
