//! macOS and Linux register a regular global shortcut. They cannot fire on modifiers alone,
//! so a modifier-only hotkey gets Space added (Shift + Win becomes Shift + Super + Space).

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use super::{super::overlay, keys::Hotkey};

pub fn register(app: &AppHandle, hotkey: Hotkey) {
    let shortcuts = app.global_shortcut();
    if let Err(error) = shortcuts.unregister_all() {
        tracing::warn!(%error, "could not clear the global shortcut");
    }
    let accelerator = accelerator(hotkey);
    let registered = shortcuts.on_shortcut(accelerator.as_str(), |app, _, event| {
        if event.state == ShortcutState::Pressed {
            overlay::toggle(app);
        }
    });
    if let Err(error) = registered {
        tracing::warn!(%error, %accelerator, "could not register the global shortcut");
    }
}

fn accelerator(hotkey: Hotkey) -> String {
    let mut parts: Vec<String> = hotkey
        .names()
        .into_iter()
        .map(|name| match name.as_str() {
            "Ctrl" => "Control".to_owned(),
            "Win" => "Super".to_owned(),
            _ => name,
        })
        .collect();
    if hotkey.key.is_none() {
        parts.push("Space".to_owned());
    }
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_only_hotkeys_get_space() {
        assert_eq!(accelerator(Hotkey::DEFAULT), "Shift+Super+Space");
        let combo = Hotkey::parse(&["Ctrl", "Alt", "K"]).unwrap();
        assert_eq!(accelerator(combo), "Control+Alt+K");
    }
}
