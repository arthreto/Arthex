//! Which folders are indexed and which are left out, saved in `file-search.json`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::services::text::fold;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    /// Folders added by the user, on top of the fixed drives.
    pub folders: Vec<String>,
    /// Folder names (`node_modules`), path ends (`AppData\Local\Temp`) or full paths.
    pub exclusions: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            exclusions: [
                "$Recycle.Bin",
                r"C:\Windows\WinSxS",
                "node_modules",
                r"AppData\Local\Temp",
            ]
            .map(String::from)
            .to_vec(),
        }
    }
}

/// Exclusions prepared for fast matching against folder paths.
pub struct Exclusions(Vec<String>);

impl Exclusions {
    pub fn new(list: &[String]) -> Self {
        Self(
            list.iter()
                .map(|item| normalized(item))
                .filter(|item| !item.is_empty())
                .collect(),
        )
    }

    pub fn excludes(&self, folder: &Path) -> bool {
        let path = normalized(&folder.to_string_lossy());
        self.0.iter().any(|exclusion| {
            if is_absolute(exclusion) {
                path == *exclusion || path.starts_with(&format!("{exclusion}\\"))
            } else {
                path.ends_with(&format!("\\{exclusion}"))
                    || path.contains(&format!("\\{exclusion}\\"))
            }
        })
    }
}

/// Folded, with `\` separators and no trailing one, so Windows and other paths compare alike.
fn normalized(path: &str) -> String {
    fold(path)
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_owned()
}

fn is_absolute(path: &str) -> bool {
    path.starts_with('\\') || path.as_bytes().get(1) == Some(&b':')
}

/// The fixed drives plus the user's folders, without folders already inside another root.
pub fn roots(config: &Config) -> Vec<PathBuf> {
    let mut roots = drives();
    for folder in &config.folders {
        let folder = PathBuf::from(folder);
        if folder.is_dir() && !roots.iter().any(|root| folder.starts_with(root)) {
            roots.retain(|root| !root.starts_with(&folder));
            roots.push(folder);
        }
    }
    roots
}

/// The fixed drives (the home folder on other systems).
#[cfg(windows)]
pub fn drives() -> Vec<PathBuf> {
    use windows_sys::Win32::{
        Storage::FileSystem::GetDriveTypeW, System::WindowsProgramming::DRIVE_FIXED,
    };

    // SAFETY: plain query of the drive bit mask.
    let mask = unsafe { windows_sys::Win32::Storage::FileSystem::GetLogicalDrives() };
    (0u8..26)
        .filter(|bit| mask & (1 << bit) != 0)
        .map(|bit| format!("{}:\\", char::from(b'A' + bit)))
        .filter(|root| {
            let wide: Vec<u16> = root.encode_utf16().chain(Some(0)).collect();
            // SAFETY: `wide` is a terminated UTF-16 string.
            unsafe { GetDriveTypeW(wide.as_ptr()) == DRIVE_FIXED }
        })
        .map(PathBuf::from)
        .collect()
}

#[cfg(not(windows))]
pub fn drives() -> Vec<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .into_iter()
        .collect()
}

/// Total and free bytes of the drive holding `path`.
#[cfg(windows)]
pub fn space(path: &Path) -> Option<(u64, u64)> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let (mut available, mut total, mut free) = (0u64, 0u64, 0u64);
    // SAFETY: `wide` is a terminated UTF-16 path and the out pointers live for the call.
    let done = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &raw mut available,
            &raw mut total,
            &raw mut free,
        )
    };
    (done != 0).then_some((total, free))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_names_path_ends_and_full_paths() {
        let exclusions = Exclusions::new(&Config::default().exclusions);
        assert!(exclusions.excludes(Path::new(r"C:\$Recycle.Bin")));
        assert!(exclusions.excludes(Path::new(r"C:\Windows\WinSxS")));
        assert!(exclusions.excludes(Path::new(r"c:\windows\winsxs\amd64_x")));
        assert!(exclusions.excludes(Path::new(r"D:\code\app\node_modules")));
        assert!(exclusions.excludes(Path::new(r"D:\code\app\node_modules\left-pad")));
        assert!(exclusions.excludes(Path::new(r"C:\Users\Léa\AppData\Local\Temp")));
        assert!(!exclusions.excludes(Path::new(r"C:\Windows")));
        assert!(!exclusions.excludes(Path::new(r"D:\code\node_modules_backup")));
        assert!(!exclusions.excludes(Path::new(r"C:\Users\Léa\AppData\Local")));
    }
}
