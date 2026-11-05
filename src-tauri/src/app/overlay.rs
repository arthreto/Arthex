use std::{
    sync::{
        Mutex, PoisonError,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use tauri::{
    AppHandle, Emitter, Manager, PhysicalSize, Webview, WebviewWindow, Window, WindowEvent,
    webview::{PageLoadEvent, PageLoadPayload},
};

use super::{events, hotkey};

pub const LABEL: &str = "main";

/// Windows may report a focus loss right after the window is shown, and clicking the tray icon
/// blurs the overlay just before the click arrives. Both are ignored within this delay.
const GRACE: Duration = Duration::from_millis(400);
/// A focus loss is checked once the foreground window has settled: focus moving into the page
/// itself also reports one.
const SETTLE: Duration = Duration::from_millis(100);
/// How long the page gets to clear itself before the overlay is hidden anyway.
const CLEARING: Duration = Duration::from_millis(300);

#[derive(Default)]
struct Timeline {
    shown_at: Mutex<Option<Instant>>,
    hidden_at: Mutex<Option<Instant>>,
    /// Open system dialogs: they take the focus, which must not close the overlay.
    held: AtomicUsize,
    /// Counts the openings, so a late hide never closes a newer opening.
    openings: AtomicU64,
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

/// A silent start leaves the overlay hidden: once its page has loaded, it can rest.
pub fn on_page_load(webview: &Webview, payload: &PageLoadPayload<'_>) {
    if payload.event() != PageLoadEvent::Finished {
        return;
    }
    if let Some(window) = webview.app_handle().get_webview_window(LABEL)
        && !window.is_visible().unwrap_or(true)
    {
        set_resting(&window, true);
    }
}

pub fn show(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    if let Some(timeline) = app.try_state::<Timeline>() {
        Timeline::mark(&timeline.shown_at);
        timeline.openings.fetch_add(1, Ordering::SeqCst);
    }
    if let Err(error) = present(&window) {
        tracing::warn!(%error, "could not show the overlay");
        return;
    }
    tracing::debug!("overlay shown");
    let _ = app.emit_to(LABEL, events::OVERLAY_SHOWN, ());
    // Windows may refuse the foreground to an app the user is not typing in: ask once more.
    let app = app.clone();
    thread::spawn(move || {
        thread::sleep(SETTLE);
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            if let Some(window) = handle.get_webview_window(LABEL)
                && window.is_visible().unwrap_or(false)
                && !is_foreground(&window)
            {
                tracing::debug!("the overlay did not get the focus, asking again");
                let _ = window.set_focus();
            }
        });
    });
}

/// `WebView2` shows its last frame when the overlay opens again. The page is asked to clear
/// itself first and then calls `conceal`, so every opening starts on the enter animation.
pub fn hide(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    if !window.is_visible().unwrap_or(false) {
        return;
    }
    tracing::debug!("overlay hiding");
    leave(app);
    let Some(timeline) = app.try_state::<Timeline>() else {
        conceal(app);
        return;
    };
    let opening = timeline.openings.load(Ordering::SeqCst);
    if app.emit_to(LABEL, events::OVERLAY_HIDING, ()).is_err() {
        conceal(app);
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        thread::sleep(CLEARING);
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            let current = handle
                .try_state::<Timeline>()
                .is_some_and(|timeline| timeline.openings.load(Ordering::SeqCst) == opening);
            let visible = handle
                .get_webview_window(LABEL)
                .is_some_and(|window| window.is_visible().unwrap_or(false));
            if current && visible {
                conceal(&handle);
            }
        });
    });
}

/// Hides the overlay and returns once the screen really shows what is under it.
pub fn hide_until_gone(app: &AppHandle) {
    /// The overlay clears its page first: it takes a moment, and rarely more than a few frames.
    const HIDING_TRIES: u32 = 40;
    const HIDING_STEP: Duration = Duration::from_millis(20);

    set_fading(app, false);
    hide(app);
    let visible = || {
        app.get_webview_window(LABEL)
            .is_some_and(|window| window.is_visible().unwrap_or(false))
    };
    for _ in 0..HIDING_TRIES {
        if !visible() {
            break;
        }
        thread::sleep(HIDING_STEP);
    }
    if visible() {
        conceal(app);
    }
    wait_for_screen();
}

/// Shows the overlay again after `hide_until_gone`.
pub fn come_back(app: &AppHandle) {
    set_fading(app, true);
    show(app);
}

/// Windows fades a window out after it is hidden: anything reading the screen meanwhile would see
/// the overlay instead of what is under it.
#[cfg(windows)]
fn set_fading(app: &AppHandle, fading: bool) {
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_TRANSITIONS_FORCEDISABLED, DwmSetWindowAttribute,
    };

    let Some(Ok(hwnd)) = app.get_webview_window(LABEL).map(|window| window.hwnd()) else {
        return;
    };
    let disabled = i32::from(!fading);
    // SAFETY: a DWM call on our own window, with a value that outlives it.
    let _ = unsafe {
        DwmSetWindowAttribute(
            hwnd.0,
            DWMWA_TRANSITIONS_FORCEDISABLED.cast_unsigned(),
            (&raw const disabled).cast(),
            u32::try_from(size_of_val(&disabled)).unwrap_or_default(),
        )
    };
}

#[cfg(not(windows))]
fn set_fading(_app: &AppHandle, _fading: bool) {}

#[cfg(windows)]
fn wait_for_screen() {
    use windows_sys::Win32::Graphics::Dwm::DwmFlush;

    const FRAMES: u32 = 3;
    const SETTLING: Duration = Duration::from_millis(60);

    for _ in 0..FRAMES {
        // SAFETY: waits for the compositor to present its next frame.
        let _ = unsafe { DwmFlush() };
    }
    thread::sleep(SETTLING);
}

#[cfg(not(windows))]
fn wait_for_screen() {}

pub fn conceal(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    leave(app);
    if let Err(error) = window.hide() {
        tracing::warn!(%error, "could not hide the overlay");
    }
    set_resting(&window, true);
    tracing::debug!("overlay hidden");
}

fn leave(app: &AppHandle) {
    // A recording left running would keep hiding every key press from the whole system.
    hotkey::record(false);
    if let Some(timeline) = app.try_state::<Timeline>() {
        Timeline::mark(&timeline.hidden_at);
    }
}

pub fn toggle(app: &AppHandle) {
    let visible = app
        .get_webview_window(LABEL)
        .is_some_and(|window| window.is_visible().unwrap_or(false));
    let just_hidden = app
        .try_state::<Timeline>()
        .is_some_and(|timeline| Timeline::within_grace(&timeline.hidden_at));
    tracing::debug!(visible, just_hidden, "the hotkey toggles the overlay");
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
            let app = app.clone();
            thread::spawn(move || {
                thread::sleep(SETTLE);
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || on_focus_lost(&handle));
            });
        }
        _ => {}
    }
}

fn on_focus_lost(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    let keep = app.try_state::<Timeline>().is_some_and(|timeline| {
        Timeline::within_grace(&timeline.shown_at) || timeline.held.load(Ordering::SeqCst) > 0
    });
    let foreground = is_foreground(&window);
    tracing::debug!(keep, foreground, "the overlay lost the focus");
    if !keep && !foreground {
        hide(app);
    }
}

#[cfg(windows)]
fn is_foreground(window: &WebviewWindow) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    // SAFETY: only reads which top-level window is in the foreground.
    let foreground = unsafe { GetForegroundWindow() };
    window.hwnd().is_ok_and(|hwnd| hwnd.0 == foreground)
}

#[cfg(not(windows))]
fn is_foreground(window: &WebviewWindow) -> bool {
    window.is_focused().unwrap_or(false)
}

/// Covers the work area of the monitor under the cursor, then takes the focus.
fn present(window: &WebviewWindow) -> tauri::Result<()> {
    set_resting(window, false);
    see_through(window);
    let cursor = window.cursor_position()?;
    let monitor = match window.monitor_from_point(cursor.x, cursor.y)? {
        Some(monitor) => Some(monitor),
        None => window.primary_monitor()?,
    };
    if let Some(monitor) = monitor {
        let area = monitor.work_area();
        let size = fitted(area.size, *monitor.size());
        tracing::debug!(
            width = size.width,
            height = size.height,
            screen_width = monitor.size().width,
            screen_height = monitor.size().height,
            "overlay placed"
        );
        window.set_position(area.position)?;
        window.set_size(size)?;
    }
    window.show()?;
    window.set_focus()
}

/// A topmost window covering a whole screen is shown by Windows like a full-screen game, straight
/// to the display: the desktop is no longer composed behind it and the transparent page turns
/// black. That happens when the taskbar hides itself, so the overlay then stops one pixel short.
fn fitted(area: PhysicalSize<u32>, screen: PhysicalSize<u32>) -> PhysicalSize<u32> {
    if area == screen {
        PhysicalSize::new(area.width, area.height.saturating_sub(1))
    } else {
        area
    }
}

/// A borderless window lets the desktop show through only once the DWM frame covers all of it;
/// without that, Windows fills what the page leaves transparent with black. Windows 11 then puts
/// its acrylic behind it: the plain blur alone leaves the window black there. Older versions,
/// which lack that backdrop, get the plain blur instead. With the `translucent` setting off, the
/// page is opaque and all of this is removed.
#[cfg(windows)]
pub fn see_through(window: &WebviewWindow) {
    use tauri::{
        utils::config::WindowEffectsConfig,
        window::{Effect, EffectsBuilder},
    };
    use windows_sys::Win32::{
        Graphics::Dwm::{
            DWMSBT_NONE, DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
            DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
        },
        UI::Controls::MARGINS,
    };

    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    let translucent = super::settings(window.app_handle()).translucent;
    if !translucent {
        let _ = window.set_effects(None::<WindowEffectsConfig>);
    }
    let margin = if translucent { -1 } else { 0 };
    let margins = MARGINS {
        cxLeftWidth: margin,
        cxRightWidth: margin,
        cyTopHeight: margin,
        cyBottomHeight: margin,
    };
    let backdrop = if translucent {
        DWMSBT_TRANSIENTWINDOW
    } else {
        DWMSBT_NONE
    };
    // SAFETY: DWM calls on our own window, with values that outlive the calls.
    let (frame, acrylic) = unsafe {
        (
            DwmExtendFrameIntoClientArea(hwnd.0, &raw const margins),
            DwmSetWindowAttribute(
                hwnd.0,
                DWMWA_SYSTEMBACKDROP_TYPE.cast_unsigned(),
                (&raw const backdrop).cast(),
                u32::try_from(size_of_val(&backdrop)).unwrap_or_default(),
            ),
        )
    };
    tracing::debug!(translucent, frame, acrylic, "overlay backdrop");
    if translucent && acrylic != 0 {
        let _ = window.set_effects(EffectsBuilder::new().effect(Effect::Blur).build());
    }
}

#[cfg(not(windows))]
pub fn see_through(_window: &WebviewWindow) {}

/// While the overlay is hidden, `WebView2` is made invisible, suspended (its page stops running)
/// and asked to give memory back. Showing it again resumes it.
#[cfg(windows)]
fn set_resting(window: &WebviewWindow, resting: bool) {
    use webview2_com::{
        Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_COLOR, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL, ICoreWebView2_2, ICoreWebView2_3,
            ICoreWebView2_19, ICoreWebView2Controller2,
        },
        TrySuspendCompletedHandler, take_pwstr,
    };
    use windows::core::Interface;

    let level = if resting {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
    } else {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
    };
    let result = window.with_webview(move |webview| {
        // SAFETY: COM calls on the WebView2 objects Tauri hands out, on the thread it calls us on.
        // Older WebView2 runtimes lack the newer interfaces and simply keep running as usual.
        unsafe {
            let controller = webview.controller();
            let _ = controller.SetIsVisible(!resting);
            // What the page leaves transparent shows this colour: it must stay fully transparent,
            // or the desktop never shows through.
            if !resting && let Ok(controller) = controller.cast::<ICoreWebView2Controller2>() {
                let mut color = COREWEBVIEW2_COLOR::default();
                let _ = controller.DefaultBackgroundColor(&raw mut color);
                tracing::debug!(alpha = color.A, "webview background");
                if color.A != 0 {
                    let _ = controller.SetDefaultBackgroundColor(COREWEBVIEW2_COLOR::default());
                }
            }
            let Ok(core) = controller.CoreWebView2() else {
                return;
            };
            if !resting
                && let Ok(environment) = core
                    .cast::<ICoreWebView2_2>()
                    .and_then(|core| core.Environment())
            {
                let mut version = windows::core::PWSTR::null();
                if environment.BrowserVersionString(&raw mut version).is_ok() {
                    tracing::debug!(version = take_pwstr(version), "webview runtime");
                }
            }
            if let Ok(core) = core.cast::<ICoreWebView2_19>() {
                let _ = core.SetMemoryUsageTargetLevel(level);
            }
            // Suspending cuts the socket of the Vite dev server, which then reloads the whole page
            // at every opening.
            if resting
                && !cfg!(debug_assertions)
                && let Ok(core) = core.cast::<ICoreWebView2_3>()
            {
                let _ =
                    core.TrySuspend(&TrySuspendCompletedHandler::create(Box::new(|_, _| Ok(()))));
            }
        }
    });
    if let Err(error) = result {
        tracing::warn!(%error, "could not change the WebView2 state");
    }
}

#[cfg(not(windows))]
fn set_resting(_window: &WebviewWindow, _resting: bool) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_covers_a_whole_screen() {
        let screen = PhysicalSize::new(1920, 1080);
        assert_eq!(fitted(screen, screen), PhysicalSize::new(1920, 1079));
        let above_taskbar = PhysicalSize::new(1920, 1032);
        assert_eq!(fitted(above_taskbar, screen), above_taskbar);
    }
}
