//! Every installed app, desktop and Store alike, for the search.

pub mod commands;

use std::{
    path::Path,
    sync::{
        Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};

/// Installed apps change rarely; an older list is rebuilt in the background.
const FRESH_FOR: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub id: String,
    pub name: String,
    /// The program file (`Code.exe`), unknown for Store apps.
    pub file: Option<String>,
    pub icon: Option<String>,
}

#[derive(Default)]
pub struct Apps {
    cache: Mutex<Option<(Instant, Vec<App>)>>,
    refreshing: AtomicBool,
}

impl Apps {
    /// The known apps at once. Only the very first call waits for the shell, which reads the icons:
    /// call it off the interface thread. A list that got old is rebuilt in the background.
    pub fn list(app: &AppHandle) -> Vec<App> {
        let apps = app.state::<Self>();
        let cached = apps
            .cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let Some((built, list)) = cached else {
            return apps.rebuild();
        };
        if built.elapsed() >= FRESH_FOR && !apps.refreshing.swap(true, Ordering::SeqCst) {
            let handle = app.clone();
            let spawned = thread::Builder::new().name("apps".into()).spawn(move || {
                let apps = handle.state::<Self>();
                apps.rebuild();
                apps.refreshing.store(false, Ordering::SeqCst);
            });
            if spawned.is_err() {
                apps.refreshing.store(false, Ordering::SeqCst);
            }
        }
        list
    }

    fn rebuild(&self) -> Vec<App> {
        let list = build();
        *self.cache.lock().unwrap_or_else(PoisonError::into_inner) =
            Some((Instant::now(), list.clone()));
        list
    }

    fn knows(&self, id: &str) -> bool {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .is_some_and(|(_, apps)| apps.iter().any(|app| app.id == id))
    }
}

pub fn setup(app: &AppHandle) {
    app.manage(Apps::default());
}

#[cfg(windows)]
fn build() -> Vec<App> {
    use crate::services::{icons, shell};

    #[allow(clippy::cast_possible_wrap)]
    let mut apps: Vec<App> = shell::apps(icons::SIZE as i32)
        .into_iter()
        .filter(|app| is_launchable(&app.id) && !is_uninstaller(&app.name))
        .map(|app| App {
            file: program_file(&app.id),
            icon: app
                .icon
                .and_then(|(width, height, pixels)| icons::rgba_data_url(width, height, pixels)),
            id: app.id,
            name: app.name,
        })
        .collect();
    apps.sort_by_key(|app| app.name.to_lowercase());
    apps.dedup_by_key(|app| app.name.to_lowercase());
    apps
}

#[cfg(not(windows))]
fn build() -> Vec<App> {
    Vec::new()
}

/// Store apps (`Publisher.App_id!App`) and programs; not help files, documents or web links.
#[cfg_attr(not(windows), allow(dead_code))]
fn is_launchable(id: &str) -> bool {
    if id.contains('!') {
        return true;
    }
    Path::new(id)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe") || ext.eq_ignore_ascii_case("msc"))
}

#[cfg_attr(not(windows), allow(dead_code))]
fn is_uninstaller(name: &str) -> bool {
    let name = name.to_lowercase();
    ["uninstall", "désinstall", "deinstall"]
        .iter()
        .any(|word| name.contains(word))
}

#[cfg_attr(not(windows), allow(dead_code))]
fn program_file(id: &str) -> Option<String> {
    (!id.contains('!')).then(|| id.rsplit(['\\', '/']).next().unwrap_or(id).to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_programs_and_store_apps_only() {
        assert!(is_launchable(
            "Microsoft.WindowsCalculator_8wekyb3d8bbwe!App"
        ));
        assert!(is_launchable(
            r"{6D809377-6AF0-444B-8957-A3773F02200E}\Microsoft VS Code\Code.exe"
        ));
        assert!(is_launchable(r"C:\Windows\system32\diskmgmt.msc"));
        assert!(!is_launchable(r"C:\Program Files\App\Help.chm"));
        assert!(!is_launchable("https://example.org/"));
        assert!(is_uninstaller("Uninstall Spotify"));
    }

    #[test]
    fn names_the_program_file() {
        assert_eq!(
            program_file(r"{6D809377}\Microsoft VS Code\Code.exe").as_deref(),
            Some("Code.exe")
        );
        assert_eq!(
            program_file("Microsoft.WindowsCalculator_8wekyb3d8bbwe!App"),
            None
        );
    }
}
