use serde_json::json;
use tauri::{
    AppHandle, Emitter, Wry,
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
};

use super::{Result, events, i18n::Translator, overlay};

const ID: &str = "main";

pub fn create(app: &AppHandle) -> Result<()> {
    let mut builder = TrayIconBuilder::with_id(ID)
        .tooltip("Arthex")
        .menu(&menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_event);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Rebuilds the menu after a language or startup change.
pub fn refresh(app: &AppHandle) -> Result<()> {
    if let Some(tray) = app.tray_by_id(ID) {
        tray.set_menu(Some(menu(app)?))?;
    }
    Ok(())
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let settings = super::settings(app);
    let text = Translator::new(settings.language.as_deref());
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "open", text.t("tray.open"), true, None::<&str>)?,
            &MenuItem::with_id(app, "settings", text.t("tray.settings"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &CheckMenuItem::with_id(
                app,
                "autostart",
                text.t("tray.autostart"),
                true,
                settings.autostart,
                None::<&str>,
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                "uninstall",
                text.t("tray.uninstall"),
                cfg!(windows),
                None::<&str>,
            )?,
            &MenuItem::with_id(app, "quit", text.t("tray.quit"), true, None::<&str>)?,
        ],
    )
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "open" => overlay::show(app),
        "settings" => {
            overlay::show(app);
            let _ = app.emit_to(overlay::LABEL, events::NAVIGATE, "settings");
        }
        "autostart" => {
            let enabled = super::settings(app).autostart;
            if let Err(error) = super::update_settings(app, &json!({ "autostart": !enabled })) {
                tracing::warn!(%error, "could not toggle the startup entry");
            }
        }
        "uninstall" => {
            overlay::show(app);
            let _ = app.emit_to(overlay::LABEL, events::CONFIRM_UNINSTALL, ());
        }
        "quit" => app.exit(0),
        _ => {}
    }
}

fn on_tray_event(tray: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        overlay::toggle(tray.app_handle());
    }
}
