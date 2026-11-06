//! Shared parts: what several modules use, such as `FFmpeg`, signed like a module and kept once in
//! `<local data>/shared/<id>/`. A module names them in its `uses`; they are fetched when it is
//! installed, given to its program when it starts, and removed with the last module using them.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use super::{
    ModuleHost,
    package::{self, SHARED_INFO, SharedInfo},
    replace_folder,
};
use crate::app::{self, AppError, Result};

/// In development, `arthex build-shared` puts them in `<ARTHEX_DEV_MODULES>/shared/`.
const DEV_FOLDER: &str = "shared";

fn dev_folder(id: &str) -> Option<PathBuf> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let dev = Path::new(&std::env::var_os("ARTHEX_DEV_MODULES")?)
        .join(DEV_FOLDER)
        .join(id);
    dev.join(SHARED_INFO).is_file().then_some(dev)
}

/// Where a shared part is, if it is there.
pub fn folder_of(host: &ModuleHost, id: &str) -> Option<PathBuf> {
    let folder = host.shared.join(id);
    if folder.join(SHARED_INFO).is_file() {
        Some(folder)
    } else {
        dev_folder(id)
    }
}

pub fn version_of(host: &ModuleHost, id: &str) -> Option<String> {
    let info: SharedInfo = app::storage::read_json(&host.shared.join(id).join(SHARED_INFO))?;
    Some(info.version)
}

/// Where each part a program uses is, for the message that starts it.
pub fn folders(host: &ModuleHost, uses: &[String]) -> BTreeMap<String, PathBuf> {
    uses.iter()
        .filter_map(|id| Some((id.clone(), folder_of(host, id)?)))
        .collect()
}

/// Checks a package like a module's, and puts it in place once the programs using it have stopped.
pub fn install(host: &ModuleHost, bytes: &[u8], expected: &str, key: &str) -> Result<()> {
    let package = package::open_shared(bytes, key)?;
    if package.info.id != expected {
        return Err(AppError::Invalid(format!(
            "the package holds {}",
            package.info.id
        )));
    }
    for module in host.list() {
        if module.info.uses.iter().any(|id| id == expected) {
            host.programs.stop(&module.id);
        }
    }
    replace_folder(&host.shared, expected, &package.files)?;
    tracing::info!(shared = %expected, version = %package.info.version, "shared part installed");
    Ok(())
}

/// What is there and that nothing uses any more.
fn unused(present: &BTreeSet<String>, used: &BTreeSet<&str>) -> Vec<String> {
    present
        .iter()
        .filter(|id| !used.contains(id.as_str()))
        .cloned()
        .collect()
}

/// A leftover is harmless: what cannot be removed now goes at the next uninstall.
pub fn prune(host: &ModuleHost) {
    let Ok(entries) = fs::read_dir(&host.shared) else {
        return;
    };
    let present: BTreeSet<String> = entries
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    let modules = host.list();
    let used: BTreeSet<&str> = modules
        .iter()
        .flat_map(|module| module.info.uses.iter().map(String::as_str))
        .collect();
    for id in unused(&present, &used) {
        if let Err(error) = fs::remove_dir_all(host.shared.join(&id)) {
            tracing::warn!(shared = %id, %error, "could not remove a shared part");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_what_nothing_uses_goes_away() {
        let present: BTreeSet<String> = ["ffmpeg".to_string(), "old".to_string()].into();
        let used: BTreeSet<&str> = ["ffmpeg"].into();
        assert_eq!(unused(&present, &used), ["old"]);
        assert_eq!(unused(&present, &BTreeSet::new()).len(), 2);
    }
}
