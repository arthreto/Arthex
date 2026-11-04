//! Signed updates from the project's GitHub Releases. An update is refused unless its signature
//! matches the public key built into the app. The release notes are kept until the next launch,
//! for "What's new".

use std::{
    path::PathBuf,
    sync::{Mutex, PoisonError},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use super::{AppError, Result, settings, storage};

/// Sent when the check at launch finds a version it does not install by itself.
pub const AVAILABLE: &str = "updates://available";
/// Download progress, `{ downloaded, total }` in bytes.
pub const PROGRESS: &str = "updates://progress";
/// The check at launch waits, so it never competes with the start of Windows.
const FIRST_CHECK: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Available {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsNew {
    pub version: String,
    /// The release description: Markdown with an English and a French section.
    pub notes: String,
}

#[derive(Clone, Serialize)]
struct Progress {
    downloaded: u64,
    total: Option<u64>,
}

#[derive(Default)]
pub struct Updates {
    found: Mutex<Option<Update>>,
}

/// Builds without a public key (development, forks) never look for updates.
fn configured(app: &AppHandle) -> bool {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|updater| updater.get("pubkey"))
        .and_then(Value::as_str)
        .is_some_and(|key| !key.trim().is_empty())
}

fn failed(error: impl std::fmt::Display) -> AppError {
    AppError::Failed(error.to_string())
}

fn notes_path(app: &AppHandle) -> Result<PathBuf> {
    Ok(app.path().app_data_dir()?.join("whats-new.json"))
}

pub async fn check(app: &AppHandle) -> Result<Option<Available>> {
    if !configured(app) {
        return Err(AppError::Unsupported);
    }
    let update = app
        .updater()
        .map_err(failed)?
        .check()
        .await
        .map_err(failed)?;
    let available = update.as_ref().map(|update| Available {
        version: update.version.clone(),
    });
    *app.state::<Updates>()
        .found
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = update;
    Ok(available)
}

/// Downloads, checks and installs the update found by the last check, then restarts.
pub async fn install(app: &AppHandle) -> Result<()> {
    let update = app
        .state::<Updates>()
        .found
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
        .ok_or_else(|| AppError::Invalid("no update to install".into()))?;
    storage::write_json(
        &notes_path(app)?,
        &WhatsNew {
            version: update.version.clone(),
            notes: update.body.clone().unwrap_or_default(),
        },
    )?;
    let handle = app.clone();
    let mut downloaded = 0u64;
    update
        .download_and_install(
            move |chunk, total| {
                downloaded += u64::try_from(chunk).unwrap_or(0);
                handle.emit(PROGRESS, Progress { downloaded, total }).ok();
            },
            || {},
        )
        .await
        .map_err(failed)?;
    tracing::info!(version = %update.version, "update installed");
    app.restart()
}

/// The notes of the version now running, once, right after an update.
pub fn whats_new(app: &AppHandle) -> Option<WhatsNew> {
    let path = notes_path(app).ok()?;
    let saved: WhatsNew = storage::read_json(&path)?;
    std::fs::remove_file(&path).ok();
    (saved.version == app.package_info().version.to_string()).then_some(saved)
}

/// Looks for an update a little after launch, and installs it when the user allows it.
pub fn start(app: &AppHandle) {
    app.manage(Updates::default());
    if !configured(app) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK);
        tauri::async_runtime::block_on(async {
            match check(&app).await {
                Ok(Some(_)) if settings(&app).auto_update => {
                    if let Err(error) = install(&app).await {
                        tracing::warn!(%error, "could not install the update");
                    }
                }
                Ok(Some(available)) => {
                    app.emit(AVAILABLE, available).ok();
                }
                Ok(None) => {}
                Err(error) => tracing::warn!(%error, "could not check for updates"),
            }
        });
    });
}
