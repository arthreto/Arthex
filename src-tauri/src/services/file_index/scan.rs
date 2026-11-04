//! Walks the indexed folders in parallel, on low-priority threads so the PC stays responsive.

use std::{
    fs::{self, Metadata},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::UNIX_EPOCH,
};

use rayon::prelude::*;

use super::{
    config::Exclusions,
    index::{Index, Item, NO_PARENT},
};

/// Folders this close to a root are scanned in parallel; deeper ones go on sequentially.
const PARALLEL_DEPTH: u8 = 2;

pub fn scan(roots: &[PathBuf], exclusions: &Exclusions, stop: &AtomicBool) -> Option<Index> {
    let threads = std::thread::available_parallelism().map_or(2, |n| (n.get() / 2).max(2));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|n| format!("file-index-{n}"))
        .start_handler(|_| lower_priority())
        .build()
        .ok()?;
    let parts: Vec<Index> = pool.install(|| {
        roots
            .par_iter()
            .map(|root| scan_folder(root, &root.to_string_lossy(), 0, exclusions, stop))
            .collect()
    });
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    let mut index = Index::default();
    for part in parts {
        index.append(part, NO_PARENT);
    }
    index.shrink_to_fit();
    Some(index)
}

/// A folder that appeared while the index runs, as a part to append under its parent.
pub fn subtree(path: &Path, name: &str, exclusions: &Exclusions) -> Index {
    let stop = AtomicBool::new(false);
    if exclusions.excludes(path) {
        let mut part = Index::default();
        part.push(&Item {
            name,
            parent: NO_PARENT,
            is_dir: true,
            summary: true,
            size: folder_size(path, &stop),
            modified: fs::metadata(path).map_or(0, |meta| modified(&meta)),
        });
        return part;
    }
    scan_folder(path, name, PARALLEL_DEPTH, exclusions, &stop)
}

/// Returns a part whose first entry is this folder.
fn scan_folder(
    path: &Path,
    name: &str,
    depth: u8,
    exclusions: &Exclusions,
    stop: &AtomicBool,
) -> Index {
    let mut part = Index::default();
    let me = part.push(&Item {
        name,
        parent: NO_PARENT,
        is_dir: true,
        summary: false,
        size: 0,
        modified: fs::metadata(path).map_or(0, |meta| modified(&meta)),
    });
    if depth < PARALLEL_DEPTH {
        let folders = read_into(&mut part, me, path, exclusions, stop);
        let children: Vec<Index> = folders
            .into_par_iter()
            .map(|(folder, name, _)| scan_folder(&folder, &name, depth + 1, exclusions, stop))
            .collect();
        for child in children {
            part.append(child, me);
        }
    } else {
        walk(&mut part, me, path, exclusions, stop);
    }
    part
}

fn walk(part: &mut Index, parent: u32, path: &Path, exclusions: &Exclusions, stop: &AtomicBool) {
    if stop.load(Ordering::Relaxed) {
        return;
    }
    for (folder, name, time) in read_into(part, parent, path, exclusions, stop) {
        let me = part.push(&Item {
            name: &name,
            parent,
            is_dir: true,
            summary: false,
            size: 0,
            modified: time,
        });
        walk(part, me, &folder, exclusions, stop);
    }
}

/// Adds the files of `path` and returns its folders to visit. Links and junctions are skipped,
/// so the walk never loops or leaves the indexed folders.
fn read_into(
    part: &mut Index,
    parent: u32,
    path: &Path,
    exclusions: &Exclusions,
    stop: &AtomicBool,
) -> Vec<(PathBuf, String, u32)> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };
    let mut folders = Vec::new();
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let meta = entry.metadata().ok();
        let time = meta.as_ref().map_or(0, modified);
        if kind.is_dir() {
            let folder = entry.path();
            if exclusions.excludes(&folder) {
                part.push(&Item {
                    name: &name,
                    parent,
                    is_dir: true,
                    summary: true,
                    size: folder_size(&folder, stop),
                    modified: time,
                });
            } else {
                folders.push((folder, name, time));
            }
        } else {
            part.push(&Item {
                name: &name,
                parent,
                is_dir: false,
                summary: false,
                size: meta.as_ref().map_or(0, Metadata::len),
                modified: time,
            });
        }
    }
    folders
}

/// The total size of the files in a folder, for excluded folders that `FileSize` still counts.
fn folder_size(path: &Path, stop: &AtomicBool) -> u64 {
    if stop.load(Ordering::Relaxed) {
        return 0;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() && !kind.is_symlink() => folder_size(&entry.path(), stop),
            Ok(kind) if kind.is_file() => entry.metadata().map_or(0, |meta| meta.len()),
            _ => 0,
        })
        .sum()
}

/// Seconds since the Unix epoch, which fit in 32 bits until 2106 and keep entries small.
pub fn modified(meta: &Metadata) -> u32 {
    meta.modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |elapsed| {
            u32::try_from(elapsed.as_secs()).unwrap_or(u32::MAX)
        })
}

#[cfg(windows)]
fn lower_priority() {
    use windows_sys::Win32::System::Threading::{
        GetCurrentThread, SetThreadPriority, THREAD_MODE_BACKGROUND_BEGIN,
    };
    // SAFETY: lowers the CPU and disk priority of the calling pool thread only.
    unsafe { SetThreadPriority(GetCurrentThread(), THREAD_MODE_BACKGROUND_BEGIN) };
}

#[cfg(not(windows))]
fn lower_priority() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_files_and_counts_excluded_folders_as_a_whole() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("docs/deep/deeper")).unwrap();
        fs::create_dir_all(dir.path().join("node_modules/pkg")).unwrap();
        fs::write(dir.path().join("docs/deep/deeper/report.pdf"), b"12345").unwrap();
        fs::write(dir.path().join("node_modules/pkg/index.js"), b"x").unwrap();

        let exclusions = Exclusions::new(&["node_modules".into()]);
        let index = scan(
            &[dir.path().to_path_buf()],
            &exclusions,
            &AtomicBool::new(false),
        )
        .unwrap();
        let hits = index.search(&["report".into()], 5);
        assert_eq!(hits.len(), 1);
        let path = index.path(hits[0].index);
        assert_eq!(path, dir.path().join("docs/deep/deeper/report.pdf"));
        assert_eq!(index.entry(hits[0].index).unwrap().size, 5);
        assert!(index.search(&["index.js".into()], 5).is_empty());
        assert!(index.search(&["node_modules".into()], 5).is_empty());
        let excluded = index.entries().iter().find(|entry| entry.is_summary());
        assert_eq!(excluded.map(|entry| entry.size), Some(1));
    }
}
