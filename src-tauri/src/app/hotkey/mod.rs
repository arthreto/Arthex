//! The global hotkey that opens the overlay, and the recording of a new one from Settings.

mod keys;
#[cfg(any(windows, test))]
mod machine;
#[cfg(not(windows))]
mod shortcut;
#[cfg(windows)]
mod windows;

use tauri::AppHandle;

pub use keys::Hotkey;

pub fn start(app: &AppHandle, names: &[String]) {
    let hotkey = Hotkey::parse(names).unwrap_or(Hotkey::DEFAULT);
    #[cfg(windows)]
    {
        let app = app.clone();
        windows::install(
            machine::Machine::new(hotkey),
            Box::new(move |signal| {
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || on_signal(&handle, signal));
            }),
        );
    }
    #[cfg(not(windows))]
    shortcut::register(app, hotkey);
}

pub fn apply(app: &AppHandle, names: &[String]) {
    let hotkey = Hotkey::parse(names).unwrap_or(Hotkey::DEFAULT);
    #[cfg(windows)]
    {
        let _ = app;
        windows::with_machine(|machine| machine.set_hotkey(hotkey));
    }
    #[cfg(not(windows))]
    shortcut::register(app, hotkey);
}

/// Starts or stops recording. Returns `false` where the interface has to read the keys itself.
pub fn record(active: bool) -> bool {
    #[cfg(windows)]
    {
        windows::with_machine(|machine| machine.record(active));
        true
    }
    #[cfg(not(windows))]
    {
        let _ = active;
        false
    }
}

#[cfg(windows)]
fn on_signal(app: &AppHandle, signal: machine::Signal) {
    use machine::Signal;
    use tauri::Emitter;

    use super::{events, overlay};

    let sent = match signal {
        Signal::Fire => {
            overlay::toggle(app);
            Ok(())
        }
        Signal::Held(mods) => app.emit_to(
            overlay::LABEL,
            events::HOTKEY_HELD,
            mods.names().collect::<Vec<_>>(),
        ),
        Signal::Recorded(hotkey) => app.emit_to(
            overlay::LABEL,
            events::HOTKEY_RECORDED,
            hotkey.map(Hotkey::names).unwrap_or_default(),
        ),
    };
    if let Err(error) = sent {
        tracing::warn!(%error, "could not send a hotkey event");
    }
}
