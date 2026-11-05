//! The history on disk, when the user asks to keep it. A copied text can be anything, so the file is
//! encrypted for the Windows account: another user or another PC cannot read it.

use std::{
    fs::{self, File},
    io::{ErrorKind, Write},
    path::Path,
};

use super::Clip;
use crate::app::Result;

/// `None` without a file, or when it cannot be read any more (another account, a damaged file).
pub fn load(path: &Path) -> Option<Vec<Clip>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            if error.kind() != ErrorKind::NotFound {
                tracing::warn!(%error, "could not read the saved clipboard history");
            }
            return None;
        }
    };
    let clips = crypt::unprotect(&bytes).and_then(|plain| Ok(serde_json::from_slice(&plain)?));
    clips
        .inspect_err(|error| tracing::warn!(%error, "the saved clipboard history is unusable"))
        .ok()
}

/// Replaces the file atomically, so a crash never leaves half of it.
pub fn save(path: &Path, clips: &[Clip]) -> Result<()> {
    let bytes = crypt::protect(&serde_json::to_vec(clips)?)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let temporary = path.with_extension("tmp");
    let mut file = File::create(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, path)?;
    Ok(())
}

pub fn delete(path: &Path) {
    if let Err(error) = fs::remove_file(path)
        && error.kind() != ErrorKind::NotFound
    {
        tracing::warn!(%error, "could not delete the saved clipboard history");
    }
}

#[cfg(windows)]
mod crypt {
    use std::{ptr, slice};

    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
    };

    use crate::app::{AppError, Result};

    fn blob(bytes: &[u8]) -> Result<CRYPT_INTEGER_BLOB> {
        Ok(CRYPT_INTEGER_BLOB {
            cbData: u32::try_from(bytes.len())
                .map_err(|_| AppError::Failed("the clipboard history is too large".into()))?,
            pbData: bytes.as_ptr().cast_mut(),
        })
    }

    /// Copies what Windows allocated for a result, then frees it.
    fn take(output: &CRYPT_INTEGER_BLOB) -> Vec<u8> {
        // SAFETY: on success Windows filled `output` with a buffer of `cbData` bytes, which
        // `LocalFree` releases once and which nothing uses afterwards.
        unsafe {
            let bytes = slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
            LocalFree(output.pbData.cast());
            bytes
        }
    }

    pub fn protect(plain: &[u8]) -> Result<Vec<u8>> {
        let input = blob(plain)?;
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };
        // SAFETY: `input` borrows `plain` for the call; the other pointers are optional and null.
        let done = unsafe {
            CryptProtectData(
                &raw const input,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &raw mut output,
            )
        };
        if done == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(take(&output))
    }

    pub fn unprotect(protected: &[u8]) -> Result<Vec<u8>> {
        let input = blob(protected)?;
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };
        // SAFETY: `input` borrows `protected` for the call; the other pointers are optional and null.
        let done = unsafe {
            CryptUnprotectData(
                &raw const input,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &raw mut output,
            )
        };
        if done == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(take(&output))
    }
}

/// Only Windows has a clipboard history to keep.
#[cfg(not(windows))]
mod crypt {
    use crate::app::{AppError, Result};

    pub fn protect(_: &[u8]) -> Result<Vec<u8>> {
        Err(AppError::Unsupported)
    }

    pub fn unprotect(_: &[u8]) -> Result<Vec<u8>> {
        Err(AppError::Unsupported)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    fn clip(id: u64, text: &str) -> Clip {
        Clip {
            id,
            text: text.into(),
            pinned: id == 1,
            copied_at: id,
        }
    }

    #[test]
    fn gives_back_what_it_saved_without_writing_it_in_clear() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clipboard.bin");
        let clips = [clip(1, "secret text"), clip(2, "été")];
        save(&path, &clips).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(!bytes.windows(6).any(|part| part == b"secret"));
        assert_eq!(load(&path).unwrap(), clips);
    }

    #[test]
    fn gives_nothing_without_a_file_or_with_a_damaged_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clipboard.bin");
        assert_eq!(load(&path), None);
        fs::write(&path, b"damaged").unwrap();
        assert_eq!(load(&path), None);
    }

    #[test]
    fn deleting_a_missing_file_is_fine() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clipboard.bin");
        save(&path, &[clip(1, "a")]).unwrap();
        delete(&path);
        assert!(!path.exists());
        delete(&path);
    }
}
