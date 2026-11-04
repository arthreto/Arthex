//! `FileSize`: what takes up space on the drives, read from the shared file index.

pub mod commands;
#[cfg(windows)]
mod windows;

use std::path::Path;

use serde::Serialize;

use crate::services::file_index::{Category, FileIndex, location};

pub const ID: &str = "filesize";
/// Items listed per folder; the folder size still counts all of them.
const MAX_ITEMS: usize = 200;
const MAX_LARGEST: usize = 50;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Drive {
    pub path: String,
    /// Bytes, 0 when the system does not say.
    pub total: u64,
    pub free: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Share {
    pub category: Category,
    pub size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub category: Category,
    pub has_children: bool,
    pub protected: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub path: String,
    pub size: u64,
    /// Largest share first, empty categories left out.
    pub shares: Vec<Share>,
    /// Largest first.
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LargeFile {
    pub name: String,
    pub path: String,
    pub location: String,
    pub size: u64,
    pub protected: bool,
}

/// `None` while the folder is not indexed yet.
pub fn folder(files: &FileIndex, path: &Path) -> Option<Folder> {
    let at = files.find(path)?;
    let index = files.read();
    let usage = index.usage(at);
    let mut shares: Vec<Share> = Category::ALL
        .into_iter()
        .zip(usage.breakdown)
        .filter(|(_, size)| *size > 0)
        .map(|(category, size)| Share { category, size })
        .collect();
    shares.sort_unstable_by_key(|share| std::cmp::Reverse(share.size));
    let items = usage
        .children
        .iter()
        .take(MAX_ITEMS)
        .filter_map(|child| {
            let entry = index.entry(child.index)?;
            Some(Item {
                name: index.name(child.index).to_owned(),
                path: index.path(child.index).to_string_lossy().into_owned(),
                is_dir: entry.is_dir(),
                size: child.size,
                category: child.category(),
                has_children: child.has_children,
                protected: child.protected,
            })
        })
        .collect();
    Some(Folder {
        path: index.path(at).to_string_lossy().into_owned(),
        size: usage.size,
        shares,
        items,
    })
}

pub fn largest(files: &FileIndex, path: &Path) -> Vec<LargeFile> {
    let Some(at) = files.find(path) else {
        return Vec::new();
    };
    let index = files.read();
    index
        .largest(at, MAX_LARGEST)
        .into_iter()
        .map(|file| {
            let path = index.path(file.index);
            LargeFile {
                name: index.name(file.index).to_owned(),
                location: location(&path),
                path: path.to_string_lossy().into_owned(),
                size: file.size,
                protected: file.protected,
            }
        })
        .collect()
}

pub fn drives() -> Vec<Drive> {
    crate::services::file_index::drives()
        .into_iter()
        .map(|drive| {
            #[cfg(windows)]
            let (total, free) = windows::space(&drive).unwrap_or_default();
            #[cfg(not(windows))]
            let (total, free) = (0, 0);
            Drive {
                path: drive.to_string_lossy().into_owned(),
                total,
                free,
            }
        })
        .collect()
}
