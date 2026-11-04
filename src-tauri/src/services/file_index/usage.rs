//! What takes up space: the size and kind of everything inside a folder, and its largest files,
//! worked out from the index in one pass each time `FileSize` asks.

use std::{cmp::Reverse, collections::BinaryHeap};

use serde::Serialize;

use super::index::{Entry, Index, NO_PARENT};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    Games,
    Personal,
    Videos,
    Apps,
    System,
    Images,
    Other,
}

impl Category {
    pub const ALL: [Self; 7] = [
        Self::Games,
        Self::Personal,
        Self::Videos,
        Self::Apps,
        Self::System,
        Self::Images,
        Self::Other,
    ];
}

/// Bytes per category, in the order of `Category::ALL`.
pub type Breakdown = [u64; 7];

/// What a folder's place says about its content; everything inside inherits it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Zone {
    None,
    System,
    Apps,
    Games,
    Users,
    Personal,
}

const ROOT_SYSTEM: &[&[u8]] = &[
    b"windows",
    b"$recycle.bin",
    b"system volume information",
    b"recovery",
    b"boot",
    b"$windows.~bt",
    b"$windows.~ws",
    b"$winreagent",
];
const ROOT_SYSTEM_FILES: &[&[u8]] = &[
    b"pagefile.sys",
    b"hiberfil.sys",
    b"swapfile.sys",
    b"dumpstack.log",
    b"dumpstack.log.tmp",
];
const ROOT_APPS: &[&[u8]] = &[b"program files", b"program files (x86)", b"programdata"];
const GAMES: &[&[u8]] = &[
    b"steamlibrary",
    b"steamapps",
    b"epic games",
    b"xboxgames",
    b"riot games",
    b"gog games",
];
const VIDEOS: &[&[u8]] = &[
    b"mp4", b"mkv", b"avi", b"mov", b"wmv", b"webm", b"m4v", b"mpg", b"mpeg", b"flv", b"ts",
];
const IMAGES: &[&[u8]] = &[
    b"jpg", b"jpeg", b"png", b"gif", b"webp", b"heic", b"heif", b"bmp", b"tif", b"tiff", b"raw",
    b"cr2", b"nef", b"arw", b"dng", b"psd", b"svg",
];
const PROGRAMS: &[&[u8]] = &[b"exe", b"dll", b"msi", b"sys"];

pub struct Child {
    pub index: u32,
    pub size: u64,
    pub breakdown: Breakdown,
    pub has_children: bool,
    /// System items and the folders that hold programs, profiles or games.
    pub protected: bool,
}

impl Child {
    /// The kind of content that takes the most room.
    pub fn category(&self) -> Category {
        largest_category(&self.breakdown)
    }
}

pub struct Usage {
    pub size: u64,
    pub breakdown: Breakdown,
    /// Largest first.
    pub children: Vec<Child>,
}

pub struct LargeFile {
    pub index: u32,
    pub size: u64,
    pub protected: bool,
}

struct Visit<'a> {
    index: u32,
    entry: &'a Entry,
    zone: Zone,
    parent_zone: Zone,
    /// Position, among the items directly inside the folder, of the one holding this entry.
    top: usize,
    is_top: bool,
}

impl Index {
    pub fn usage(&self, folder: u32) -> Usage {
        let guarded = self.starts_zone(folder);
        let mut children: Vec<Child> = Vec::new();
        self.walk_under(folder, |visit| {
            if visit.is_top {
                children.push(Child {
                    index: visit.index,
                    size: 0,
                    breakdown: [0; 7],
                    has_children: false,
                    protected: guarded || is_protected(visit.entry, visit.zone, visit.parent_zone),
                });
            }
            let Some(child) = children.get_mut(visit.top) else {
                return;
            };
            if !visit.is_top {
                child.has_children = true;
            }
            if !visit.entry.is_dir() || visit.entry.is_summary() {
                let category = category(visit.zone, self.folded_name(visit.entry));
                child.size += visit.entry.size;
                child.breakdown[category as usize] += visit.entry.size;
            }
        });
        children.sort_unstable_by_key(|child| Reverse(child.size));
        let mut breakdown = [0; 7];
        for child in &children {
            for (total, bytes) in breakdown.iter_mut().zip(child.breakdown) {
                *total += bytes;
            }
        }
        Usage {
            size: breakdown.iter().sum(),
            breakdown,
            children,
        }
    }

    /// The largest files inside a folder, largest first.
    pub fn largest(&self, folder: u32, limit: usize) -> Vec<LargeFile> {
        let mut heap = BinaryHeap::with_capacity(limit + 1);
        self.walk_under(folder, |visit| {
            if !visit.entry.is_dir() {
                heap.push(Reverse((
                    visit.entry.size,
                    visit.index,
                    visit.zone == Zone::System,
                )));
                if heap.len() > limit {
                    heap.pop();
                }
            }
        });
        heap.into_sorted_vec()
            .into_iter()
            .map(|Reverse((size, index, protected))| LargeFile {
                index,
                size,
                protected,
            })
            .collect()
    }

    /// Whether an entry must not be sent to the Recycle Bin.
    pub fn is_protected(&self, index: u32) -> bool {
        let Some(entry) = self.entry(index) else {
            return true;
        };
        if entry.parent == NO_PARENT {
            return true;
        }
        match (self.zone(entry.parent), self.zone(index)) {
            (Some(parent_zone), Some(zone)) => {
                is_protected(entry, zone, parent_zone) || self.starts_zone(entry.parent)
            }
            _ => true,
        }
    }

    /// A folder like Program Files, a profile or `AppData`: what sits directly inside it
    /// (`AppData\Roaming`, `Documents`, an installed program) is protected too.
    fn starts_zone(&self, index: u32) -> bool {
        let Some(entry) = self.entry(index) else {
            return false;
        };
        entry.parent != NO_PARENT && entry.is_dir() && self.zone(index) != self.zone(entry.parent)
    }

    /// Visits every live entry inside `folder` in index order. Parents always come before their
    /// content, so one pass carries each entry's zone and top item down to everything inside it.
    fn walk_under(&self, folder: u32, mut visit: impl FnMut(Visit<'_>)) {
        let Some(folder_zone) = self.zone(folder) else {
            return;
        };
        let folder_is_drive = self
            .entry(folder)
            .is_some_and(|entry| entry.parent == NO_PARENT && is_drive(self.folded_name(entry)));
        let start = folder as usize;
        let entries = self.entries();
        let count = entries.len() - start;
        let mut tops = vec![u32::MAX; count];
        let mut zones = vec![Zone::None; count];
        zones[0] = folder_zone;
        let mut next_top = 0;
        for offset in 1..count {
            let entry = &entries[start + offset];
            if entry.is_removed() || entry.parent == NO_PARENT || (entry.parent as usize) < start {
                continue;
            }
            let parent = entry.parent as usize - start;
            let is_top = parent == 0;
            let top = if is_top {
                next_top += 1;
                next_top - 1
            } else if tops[parent] == u32::MAX {
                continue;
            } else {
                tops[parent]
            };
            let parent_zone = zones[parent];
            let zone = zone_of(
                parent_zone,
                is_top && folder_is_drive,
                entry,
                self.folded_name(entry),
            );
            tops[offset] = top;
            zones[offset] = zone;
            visit(Visit {
                index: u32::try_from(start + offset).unwrap_or(u32::MAX),
                entry,
                zone,
                parent_zone,
                top: top as usize,
                is_top,
            });
        }
    }

    /// The zone of one entry, from its chain of parents.
    fn zone(&self, index: u32) -> Option<Zone> {
        let mut chain = Vec::new();
        let mut current = index;
        loop {
            let entry = self.entry(current)?;
            chain.push(entry);
            if entry.parent == NO_PARENT || chain.len() > 512 {
                break;
            }
            current = entry.parent;
        }
        let mut zone = Zone::None;
        let mut parent_is_drive = false;
        for entry in chain.into_iter().rev() {
            let name = self.folded_name(entry);
            if entry.parent == NO_PARENT {
                parent_is_drive = is_drive(name);
                continue;
            }
            zone = zone_of(zone, parent_is_drive, entry, name);
            parent_is_drive = false;
        }
        Some(zone)
    }
}

fn zone_of(parent: Zone, at_drive_root: bool, entry: &Entry, name: &[u8]) -> Zone {
    if parent == Zone::System {
        return Zone::System;
    }
    if !entry.is_dir() {
        return if at_drive_root && ROOT_SYSTEM_FILES.contains(&name) {
            Zone::System
        } else {
            parent
        };
    }
    if GAMES.contains(&name) {
        return Zone::Games;
    }
    if at_drive_root {
        if ROOT_SYSTEM.contains(&name) {
            return Zone::System;
        }
        if ROOT_APPS.contains(&name) {
            return Zone::Apps;
        }
        if name == b"users" {
            return Zone::Users;
        }
    }
    match parent {
        Zone::Users => Zone::Personal,
        Zone::Personal if name == b"appdata" => Zone::Apps,
        other => other,
    }
}

/// Roots, system items, and folders that start a zone (Windows, Program Files, a profile,
/// `AppData`, a game library).
fn is_protected(entry: &Entry, zone: Zone, parent_zone: Zone) -> bool {
    entry.parent == NO_PARENT || zone == Zone::System || (entry.is_dir() && zone != parent_zone)
}

fn category(zone: Zone, name: &[u8]) -> Category {
    match zone {
        Zone::System => Category::System,
        Zone::Apps => Category::Apps,
        Zone::Games => Category::Games,
        Zone::None | Zone::Users | Zone::Personal => {
            let extension = memchr::memrchr(b'.', name).map_or(&[][..], |dot| &name[dot + 1..]);
            if VIDEOS.contains(&extension) {
                Category::Videos
            } else if IMAGES.contains(&extension) {
                Category::Images
            } else if zone == Zone::Personal {
                Category::Personal
            } else if PROGRAMS.contains(&extension) {
                Category::Apps
            } else {
                Category::Other
            }
        }
    }
}

fn largest_category(breakdown: &Breakdown) -> Category {
    Category::ALL
        .into_iter()
        .zip(breakdown)
        .max_by_key(|(_, bytes)| **bytes)
        .map_or(Category::Other, |(category, _)| category)
}

/// `c:\`, as a drive root is named in the index.
fn is_drive(name: &[u8]) -> bool {
    matches!(name, [_, b':', b'\\'])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::file_index::index::Item;

    fn add(index: &mut Index, name: &str, parent: u32, is_dir: bool, size: u64) -> u32 {
        index.push(&Item {
            name,
            parent,
            is_dir,
            summary: false,
            size,
            modified: 0,
        })
    }

    /// `C:\` with Windows, a profile holding a video and a document, and a game library.
    fn drive() -> (Index, [u32; 6]) {
        let mut index = Index::default();
        let root = add(&mut index, r"C:\", NO_PARENT, true, 0);
        let windows = add(&mut index, "Windows", root, true, 0);
        add(&mut index, "explorer.exe", windows, false, 50);
        let pagefile = add(&mut index, "pagefile.sys", root, false, 400);
        let users = add(&mut index, "Users", root, true, 0);
        let lea = add(&mut index, "Léa", users, true, 0);
        add(&mut index, "trip.mp4", lea, false, 300);
        add(&mut index, "notes.txt", lea, false, 10);
        let steam = add(&mut index, "SteamLibrary", root, true, 0);
        add(&mut index, "game.pak", steam, false, 200);
        (index, [root, windows, pagefile, users, lea, steam])
    }

    #[test]
    fn sums_each_item_and_tells_its_kind() {
        let (index, [root, windows, pagefile, users, lea, steam]) = drive();
        let usage = index.usage(root);
        assert_eq!(usage.size, 960);
        let order: Vec<u32> = usage.children.iter().map(|child| child.index).collect();
        assert_eq!(order, [pagefile, users, steam, windows]);
        let kinds: Vec<Category> = usage.children.iter().map(Child::category).collect();
        assert_eq!(
            kinds,
            [
                Category::System,
                Category::Videos,
                Category::Games,
                Category::System
            ]
        );
        assert_eq!(usage.breakdown[Category::Personal as usize], 10);
        assert!(
            index
                .usage(lea)
                .children
                .iter()
                .all(|child| !child.has_children)
        );
        assert!(usage.children.iter().all(|child| child.protected));
        assert!(index.is_protected(lea));
        assert!(index.is_protected(lea + 1), "directly inside a profile");
        assert!(
            index
                .usage(lea)
                .children
                .iter()
                .all(|child| child.protected)
        );
        assert!(index.is_protected(users) && index.is_protected(steam));
    }

    #[test]
    fn protects_what_holds_programs_and_profiles_but_not_their_content() {
        let (mut index, [.., lea, _]) = drive();
        let downloads = add(&mut index, "Downloads", lea, true, 0);
        let iso = add(&mut index, "setup.iso", downloads, false, 5);
        let old = add(&mut index, "Old", downloads, true, 0);
        assert!(index.is_protected(downloads));
        assert!(!index.is_protected(iso));
        assert!(!index.is_protected(old));
        assert!(
            index
                .usage(downloads)
                .children
                .iter()
                .all(|child| !child.protected)
        );
    }

    #[test]
    fn lists_the_largest_files_and_leaves_out_removed_ones() {
        let (mut index, [root, _, pagefile, _, lea, _]) = drive();
        let largest = index.largest(root, 2);
        assert_eq!(largest[0].index, pagefile);
        assert!(largest[0].protected);
        assert_eq!(largest[1].size, 300);
        index.remove(lea);
        assert_eq!(index.usage(root).size, 650);
        assert_eq!(index.largest(root, 2)[1].size, 200);
    }
}
