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
    if let Some(window) = app.get_webview_window(LABEL) {
        set_low_memory(&window, true);
    }
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
    set_low_memory(&window, true);
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
    set_low_memory(window, false);
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

/// While the overlay is hidden, `WebView2` is asked to give back as much memory as it can.
#[cfg(windows)]
fn set_low_memory(window: &WebviewWindow, low: bool) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
        ICoreWebView2_19,
    };
    use windows::core::Interface;

    let level = if low {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
    } else {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
    };
    let result = window.with_webview(move |webview| {
        // SAFETY: COM calls on the WebView2 objects Tauri hands out, on the thread it calls us on.
        // Older WebView2 runtimes lack ICoreWebView2_19 and simply keep their usual memory.
        unsafe {
            if let Ok(core) = webview.controller().CoreWebView2()
                && let Ok(core) = core.cast::<ICoreWebView2_19>()
            {
                let _ = core.SetMemoryUsageTargetLevel(level);
            }
        }
    });
    if let Err(error) = result {
        tracing::warn!(%error, "could not change the WebView2 memory level");
    }
}

#[cfg(not(windows))]
fn set_low_memory(_window: &WebviewWindow, _low: bool) {}
