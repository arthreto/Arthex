//! Saves the index to `index.bin`, so it is ready at launch while a fresh scan runs later.

use std::{
    fs::{self, File},
    io::{BufWriter, Read, Write},
    path::Path,
};

use super::index::{Entry, Index};
use crate::app::Result;

const MAGIC: &[u8; 8] = b"ARTHEXI1";
/// Refuses absurd files rather than allocating for them.
const MAX_ENTRIES: usize = 50_000_000;
const ENTRY_BYTES: usize = 4 + 2 + 4 + 2 + 4 + 8 + 8 + 1;

pub fn save(path: &Path, index: &Index) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("bin.tmp");
    let mut out = BufWriter::new(File::create(&tmp)?);
    let (names, folded, entries) = index.parts();
    out.write_all(MAGIC)?;
    for length in [entries.len(), names.len(), folded.len()] {
        out.write_all(&u64::try_from(length).unwrap_or(u64::MAX).to_le_bytes())?;
    }
    out.write_all(names.as_bytes())?;
    out.write_all(folded.as_bytes())?;
    for entry in entries {
        let (name_start, name_len, folded_start, folded_len, parent, size, modified, flags) =
            entry.raw();
        out.write_all(&name_start.to_le_bytes())?;
        out.write_all(&name_len.to_le_bytes())?;
        out.write_all(&folded_start.to_le_bytes())?;
        out.write_all(&folded_len.to_le_bytes())?;
        out.write_all(&parent.to_le_bytes())?;
        out.write_all(&size.to_le_bytes())?;
        out.write_all(&modified.to_le_bytes())?;
        out.write_all(&[flags])?;
    }
    out.into_inner()
        .map_err(std::io::IntoInnerError::into_error)?
        .sync_all()?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// `None` for a missing, old or damaged file: the next scan rebuilds it.
pub fn load(path: &Path) -> Option<Index> {
    let mut bytes = Vec::new();
    File::open(path).ok()?.read_to_end(&mut bytes).ok()?;
    let mut reader = Reader {
        bytes: &bytes,
        at: 0,
    };
    if reader.take(MAGIC.len())? != MAGIC {
        return None;
    }
    let count = usize::try_from(reader.u64()?).ok()?;
    let names_len = usize::try_from(reader.u64()?).ok()?;
    let folded_len = usize::try_from(reader.u64()?).ok()?;
    if count > MAX_ENTRIES
        || bytes.len() != MAGIC.len() + 24 + names_len + folded_len + count * ENTRY_BYTES
    {
        return None;
    }
    let names = String::from_utf8(reader.take(names_len)?.to_vec()).ok()?;
    let folded = String::from_utf8(reader.take(folded_len)?.to_vec()).ok()?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(Entry::from_raw((
            reader.u32()?,
            reader.u16()?,
            reader.u32()?,
            reader.u16()?,
            reader.u32()?,
            reader.u64()?,
            i64::from_le_bytes(reader.array()?),
            reader.take(1)?[0],
        )));
    }
    Index::from_parts(names, folded, entries)
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        let slice = self.bytes.get(self.at..self.at.checked_add(length)?)?;
        self.at += length;
        Some(slice)
    }

    fn array<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N)?.try_into().ok()
    }

    fn u16(&mut self) -> Option<u16> {
        self.array().map(u16::from_le_bytes)
    }

    fn u32(&mut self) -> Option<u32> {
        self.array().map(u32::from_le_bytes)
    }

    fn u64(&mut self) -> Option<u64> {
        self.array().map(u64::from_le_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::file_index::index::{Item, NO_PARENT};

    #[test]
    fn round_trips_and_refuses_damage() {
        let mut index = Index::default();
        let root = index.push(&Item {
            name: "root",
            parent: NO_PARENT,
            is_dir: true,
            summary: false,
            size: 0,
            modified: 1,
        });
        index.push(&Item {
            name: "Été.txt",
            parent: root,
            is_dir: false,
            summary: false,
            size: 42,
            modified: 7,
        });
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.bin");
        save(&path, &index).unwrap();

        let loaded = load(&path).unwrap();
        let hits = loaded.search(&["ete".into()], 5);
        assert_eq!(loaded.name(hits[0].index), "Été.txt");
        assert_eq!(loaded.entry(hits[0].index).unwrap().size, 42);

        let mut bytes = fs::read(&path).unwrap();
        bytes.truncate(bytes.len() - 3);
        fs::write(&path, bytes).unwrap();
        assert!(load(&path).is_none());
    }
}
