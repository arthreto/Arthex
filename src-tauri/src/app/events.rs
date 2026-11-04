//! Event names shared with `src/core/api.ts`.

pub const OVERLAY_SHOWN: &str = "overlay://shown";
pub const SETTINGS_CHANGED: &str = "settings://changed";
pub const NAVIGATE: &str = "app://navigate";
pub const CONFIRM_UNINSTALL: &str = "app://confirm-uninstall";
#[cfg(windows)]
pub const HOTKEY_HELD: &str = "hotkey://held";
#[cfg(windows)]
pub const HOTKEY_RECORDED: &str = "hotkey://recorded";
