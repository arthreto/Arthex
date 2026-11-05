//! Lets the user take a colour from anywhere on the screen.
//!
//! The overlay goes away and the screen is copied once, before anything of ours is on it: the
//! colour read is the real one, without the fade that shows the picker is active. That copy stays
//! in memory for as long as the picker is open, and is never saved or logged.

#[cfg(windows)]
mod paint;
#[cfg(windows)]
mod picker;

use tauri::AppHandle;

/// The colour the user clicked, as `#RRGGBB`. `None` when they cancelled.
#[cfg(windows)]
pub fn pick(app: &AppHandle) -> Option<String> {
    use std::{thread, time::Duration};

    use tauri::Manager;
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_TRANSITIONS_FORCEDISABLED, DwmFlush, DwmSetWindowAttribute,
    };

    use crate::app::overlay;

    /// The overlay clears its page first: it takes a moment, and rarely more than a few frames.
    const HIDING_TRIES: u32 = 40;
    const HIDING_STEP: Duration = Duration::from_millis(20);
    /// Frames to wait for the screen to show what is under the overlay.
    const SETTLING_FRAMES: u32 = 3;
    const SETTLING_TIME: Duration = Duration::from_millis(60);

    // Windows fades a window out after it is hidden. Copying the screen during that fade would
    // read the overlay instead of what is under it, so it goes at once this time.
    let set_transitions_off = |off: i32| {
        if let Some(Ok(hwnd)) = app
            .get_webview_window(overlay::LABEL)
            .map(|window| window.hwnd())
        {
            // SAFETY: a DWM call on our own window, with a value that outlives it.
            let _ = unsafe {
                DwmSetWindowAttribute(
                    hwnd.0,
                    DWMWA_TRANSITIONS_FORCEDISABLED.cast_unsigned(),
                    (&raw const off).cast(),
                    u32::try_from(size_of_val(&off)).unwrap_or_default(),
                )
            };
        }
    };

    set_transitions_off(1);
    overlay::hide(app);
    let visible = || {
        app.get_webview_window(overlay::LABEL)
            .is_some_and(|window| window.is_visible().unwrap_or(false))
    };
    for _ in 0..HIDING_TRIES {
        if !visible() {
            break;
        }
        thread::sleep(HIDING_STEP);
    }
    if visible() {
        overlay::conceal(app);
    }
    for _ in 0..SETTLING_FRAMES {
        // SAFETY: waits for the compositor to present its next frame.
        let _ = unsafe { DwmFlush() };
    }
    thread::sleep(SETTLING_TIME);

    let picked = picker::run();
    tracing::debug!(picked = picked.is_some(), "screen colour picker closed");
    set_transitions_off(0);
    overlay::show(app);
    picked
}

#[cfg(not(windows))]
pub fn pick(_app: &AppHandle) -> Option<String> {
    None
}
