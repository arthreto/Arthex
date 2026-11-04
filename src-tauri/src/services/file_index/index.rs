//! The file index: every name in two shared buffers (as written, and folded for matching) plus a
//! small fixed record per entry, so a million files fit in a few dozen megabytes.

use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use memchr::memmem;
use rayon::prelude::*;

use crate::services::text::fold;

pub const NO_PARENT: u32 = u32::MAX;
const DIRECTORY: u8 = 1;
/// An excluded folder: kept as one entry holding its total size, without its content.
const SUMMARY: u8 = 2;
/// Sent to the Recycle Bin since the last scan.
const REMOVED: u8 = 4;

#[derive(Debug, Clone, Copy)]
pub struct Entry {
    name_start: u32,
    name_len: u16,
    folded_start: u32,
    folded_len: u16,
    pub parent: u32,
    pub size: u64,
    /// Seconds since the Unix epoch.
    pub modified: u32,
    flags: u8,
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        self.flags & DIRECTORY != 0
    }

    pub fn is_summary(&self) -> bool {
        self.flags & SUMMARY != 0
    }

    pub fn is_removed(&self) -> bool {
        self.flags & REMOVED != 0
    }
}

#[derive(Debug, Default)]
pub struct Index {
    names: String,
    folded: String,
    entries: Vec<Entry>,
}

/// What a scan or a file event knows about one item.
pub struct Item<'a> {
    pub name: &'a str,
    pub parent: u32,
    pub is_dir: bool,
    /// An excluded folder whose `size` is its whole content.
    pub summary: bool,
    pub size: u64,
    pub modified: u32,
}

pub struct Hit {
    pub index: u32,
    pub score: u8,
    pub modified: u32,
}

impl Index {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn push(&mut self, item: &Item<'_>) -> u32 {
        let folded = fold(item.name);
        let entry = Entry {
            name_start: offset(self.names.len()),
            name_len: u16::try_from(item.name.len()).unwrap_or(u16::MAX),
            folded_start: offset(self.folded.len()),
            folded_len: u16::try_from(folded.len()).unwrap_or(u16::MAX),
            parent: item.parent,
            size: item.size,
            modified: item.modified,
            flags: (if item.is_dir { DIRECTORY } else { 0 })
                | (if item.summary { SUMMARY } else { 0 }),
        };
        self.names.push_str(item.name);
        self.folded.push_str(&folded);
        self.entries.push(entry);
        offset(self.entries.len() - 1)
    }

    /// Appends another index whose first entry is a folder of this one, `parent`.
    pub fn append(&mut self, part: Index, parent: u32) {
        let base = offset(self.entries.len());
        let names = offset(self.names.len());
        let folded = offset(self.folded.len());
        self.names.push_str(&part.names);
        self.folded.push_str(&part.folded);
        self.entries.extend(
            part.entries
                .into_iter()
                .enumerate()
                .map(|(position, mut entry)| {
                    entry.name_start += names;
                    entry.folded_start += folded;
                    entry.parent = if position == 0 {
                        parent
                    } else {
                        entry.parent + base
                    };
                    entry
                }),
        );
    }

    /// Gives back the room left over by growing, once the index is complete.
    pub fn shrink_to_fit(&mut self) {
        self.names.shrink_to_fit();
        self.folded.shrink_to_fit();
        self.entries.shrink_to_fit();
    }

    pub fn entry(&self, index: u32) -> Option<&Entry> {
        self.entries.get(index as usize)
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Hides an entry, and so everything inside it, until the next scan.
    pub fn remove(&mut self, index: u32) {
        if let Some(entry) = self.entries.get_mut(index as usize) {
            entry.flags |= REMOVED;
        }
    }

    pub fn name(&self, index: u32) -> &str {
        self.entry(index)
            .and_then(|entry| self.names.get(span(entry.name_start, entry.name_len)))
            .unwrap_or_default()
    }

    pub fn folded_name(&self, entry: &Entry) -> &[u8] {
        self.folded
            .as_bytes()
            .get(span(entry.folded_start, entry.folded_len))
            .unwrap_or_default()
    }

    /// The full path, rebuilt from the parents. Roots store their whole path as their name.
    pub fn path(&self, index: u32) -> PathBuf {
        let mut parts = Vec::new();
        let mut current = index;
        while let Some(entry) = self.entry(current) {
            parts.push(self.name(current));
            if entry.parent == NO_PARENT || parts.len() > 512 {
                break;
            }
            current = entry.parent;
        }
        let mut path = PathBuf::new();
        for part in parts.into_iter().rev() {
            path.push(part);
        }
        path
    }

    /// Entries whose name holds every word, best first: exact name, then name start, then
    /// anywhere; most recently modified first among equals.
    pub fn search(&self, words: &[String], limit: usize) -> Vec<Hit> {
        if words.is_empty() {
            return Vec::new();
        }
        let finders: Vec<_> = words
            .iter()
            .map(|word| memmem::Finder::new(word.as_bytes()))
            .collect();
        let whole = words.join(" ");
        let first = words[0].as_bytes();
        let mut hits: Vec<Hit> = self
            .entries
            .par_iter()
            .enumerate()
            .filter_map(|(position, entry)| {
                if entry.flags & (SUMMARY | REMOVED) != 0 {
                    return None;
                }
                let name = self.folded_name(entry);
                if !finders.iter().all(|finder| finder.find(name).is_some()) {
                    return None;
                }
                let score = if name == whole.as_bytes() || stem(name) == whole.as_bytes() {
                    3
                } else if name.starts_with(first) {
                    2
                } else {
                    1
                };
                Some(Hit {
                    index: offset(position),
                    score,
                    modified: entry.modified,
                })
            })
            .collect();
        hits.par_sort_unstable_by(|a, b| b.score.cmp(&a.score).then(b.modified.cmp(&a.modified)));
        hits.truncate(limit);
        hits
    }

    pub fn parts(&self) -> (&str, &str, &[Entry]) {
        (&self.names, &self.folded, &self.entries)
    }

    pub fn from_parts(names: String, folded: String, entries: Vec<Entry>) -> Option<Self> {
        let count = entries.len();
        let valid = entries.iter().all(|entry| {
            names.get(span(entry.name_start, entry.name_len)).is_some()
                && folded
                    .get(span(entry.folded_start, entry.folded_len))
                    .is_some()
                && (entry.parent == NO_PARENT || (entry.parent as usize) < count)
        });
        valid.then_some(Self {
            names,
            folded,
            entries,
        })
    }
}

impl Entry {
    pub fn raw(&self) -> (u32, u16, u32, u16, u32, u64, u32, u8) {
        (
            self.name_start,
            self.name_len,
            self.folded_start,
            self.folded_len,
            self.parent,
            self.size,
            self.modified,
            self.flags,
        )
    }

    pub fn from_raw(raw: (u32, u16, u32, u16, u32, u64, u32, u8)) -> Self {
        let (name_start, name_len, folded_start, folded_len, parent, size, modified, flags) = raw;
        Self {
            name_start,
            name_len,
            folded_start,
            folded_len,
            parent,
            size,
            modified,
            flags,
        }
    }
}

/// The name without its extension, so "rapport" is an exact match for "rapport.pdf".
fn stem(name: &[u8]) -> &[u8] {
    match memchr::memrchr(b'.', name) {
        Some(dot) if dot > 0 => &name[..dot],
        _ => name,
    }
}

fn span(start: u32, len: u16) -> std::ops::Range<usize> {
    start as usize..start as usize + len as usize
}

fn offset(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// The folder part of a path, with a trailing separator, as the interface shows it.
pub fn location(path: &Path) -> String {
    path.parent()
        .map(|parent| {
            let mut text = parent.to_string_lossy().into_owned();
            if !text.ends_with(MAIN_SEPARATOR) {
                text.push(MAIN_SEPARATOR);
            }
            text
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, parent: u32, is_dir: bool, modified: u32) -> Item<'_> {
        Item {
            name,
            parent,
            is_dir,
            summary: false,
            size: 0,
            modified,
        }
    }

    fn sample() -> Index {
        let mut index = Index::default();
        let root = index.push(&item("/data", NO_PARENT, true, 0));
        let docs = index.push(&item("Documents", root, true, 0));
        index.push(&item("Rapport été.pdf", docs, false, 10));
        index.push(&item("ancien rapport.docx", docs, false, 30));
        index.push(&item("rapport", root, true, 5));
        index
    }

    fn names(index: &Index, hits: &[Hit]) -> Vec<String> {
        hits.iter()
            .map(|hit| index.name(hit.index).to_owned())
            .collect()
    }

    #[test]
    fn finds_every_word_ignoring_case_and_accents() {
        let index = sample();
        let hits = index.search(&["rapport".into(), "ete".into()], 10);
        assert_eq!(names(&index, &hits), ["Rapport été.pdf"]);
        assert_eq!(
            index.path(hits[0].index),
            Path::new("/data/Documents/Rapport été.pdf")
        );
    }

    #[test]
    fn ranks_exact_then_prefix_then_recent() {
        let index = sample();
        let hits = index.search(&["rapport".into()], 10);
        assert_eq!(
            names(&index, &hits),
            ["rapport", "Rapport été.pdf", "ancien rapport.docx"]
        );
    }

    #[test]
    fn appends_a_part_under_a_folder() {
        let mut index = sample();
        let mut part = Index::default();
        let music = part.push(&item("Music", NO_PARENT, true, 0));
        part.push(&item("song.mp3", music, false, 0));
        index.append(part, 0);
        let hits = index.search(&["song".into()], 10);
        assert_eq!(index.path(hits[0].index), Path::new("/data/Music/song.mp3"));
    }

    #[test]
    fn rejects_broken_parts() {
        let index = sample();
        let (names, folded, entries) = index.parts();
        assert!(Index::from_parts(names.into(), folded.into(), entries.to_vec()).is_some());
        assert!(Index::from_parts(String::new(), folded.into(), entries.to_vec()).is_none());
    }
}
