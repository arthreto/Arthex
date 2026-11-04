//! Fast file and folder search over the shared file index.

pub mod commands;

use std::fs;

use serde::Serialize;

use crate::services::{
    file_index::{FileIndex, location},
    text::words,
};

pub const ID: &str = "file-search";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHit {
    pub path: String,
    pub name: String,
    pub location: String,
    pub is_dir: bool,
    pub size: u64,
    /// Seconds since the Unix epoch.
    pub modified: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The fixed drives, always indexed.
    pub drives: Vec<String>,
    /// Folders added by the user.
    pub folders: Vec<String>,
    pub exclusions: Vec<String>,
}

/// The best matches that still exist; the index may lag behind a deletion.
pub fn search(files: &FileIndex, query: &str, limit: usize) -> Vec<FileHit> {
    let index = files.read();
    index
        .search(&words(query), limit * 3)
        .into_iter()
        .filter_map(|hit| {
            let path = index.path(hit.index);
            fs::symlink_metadata(&path).ok()?;
            let entry = index.entry(hit.index)?;
            Some(FileHit {
                name: index.name(hit.index).to_owned(),
                location: location(&path),
                path: path.to_string_lossy().into_owned(),
                is_dir: entry.is_dir(),
                size: entry.size,
                modified: entry.modified,
            })
        })
        .take(limit)
        .collect()
}

pub fn status(files: &FileIndex) -> Status {
    let config = files.current_config();
    Status {
        drives: crate::services::file_index::drives()
            .iter()
            .map(|drive| drive.to_string_lossy().into_owned())
            .collect(),
        folders: config.folders,
        exclusions: config.exclusions,
    }
}
