//! JSON files in the app folders, replaced atomically so a crash never leaves half a file.

use std::{
    fs::{self, File},
    io::{ErrorKind, Write},
    path::Path,
};

use serde::{Serialize, de::DeserializeOwned};

use super::Result;

/// A missing file gives `None`. A corrupt one is set aside as `.json.corrupt` and gives `None`.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            if error.kind() != ErrorKind::NotFound {
                tracing::warn!(%error, path = %path.display(), "could not read a data file");
            }
            return None;
        }
    };
    serde_json::from_slice(&bytes)
        .inspect_err(|error| {
            tracing::warn!(%error, path = %path.display(), "data file is corrupt, setting it aside");
            let _ = fs::rename(path, path.with_extension("json.corrupt"));
        })
        .ok()
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    let mut file = File::create(&tmp)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    // On disk before the rename, so a power cut never leaves an empty file in its place.
    file.sync_all()?;
    drop(file);
    fs::rename(&tmp, path)?;
    Ok(())
}
