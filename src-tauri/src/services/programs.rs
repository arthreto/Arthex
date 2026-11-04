//! Installed programs, as listed in the Start menu, for the shortcut picker.

use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    pub name: String,
    /// What gets opened: a `.lnk` from the Start menu or an `.exe`.
    pub path: String,
    /// File name of the program itself, shown next to the name.
    pub file: String,
    pub icon: Option<String>,
}

/// A program the user picked or typed: an existing `.exe` or `.lnk` given by its full path.
pub fn describe(path: &Path) -> Option<Program> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    if !path.is_absolute() || !path.is_file() || !matches!(extension.as_str(), "exe" | "lnk") {
        return None;
    }
    let name = path.file_stem()?.to_string_lossy().into_owned();
    let file = program_file(path)?;
    Some(Program {
        name,
        path: path.to_string_lossy().into_owned(),
        file,
        icon: super::icons::file_icon(path),
    })
}

#[cfg(windows)]
fn program_file(path: &Path) -> Option<String> {
    let target = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
    {
        super::shell::shortcut_target(path)?
    } else {
        path.to_path_buf()
    };
    let is_exe = target
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
    is_exe.then(|| {
        target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    })?
}

#[cfg(not(windows))]
fn program_file(path: &Path) -> Option<String> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

/// Start menu entries of the user and of all users, without uninstallers or documents.
#[cfg(windows)]
pub fn installed() -> Vec<Program> {
    const SKIPPED: [&str; 3] = ["uninstall", "désinstall", "deinstall"];

    let roots = ["APPDATA", "ProgramData"]
        .into_iter()
        .filter_map(std::env::var_os)
        .map(|base| Path::new(&base).join(r"Microsoft\Windows\Start Menu\Programs"));
    let mut links = Vec::new();
    for root in roots {
        collect_links(&root, 4, &mut links);
    }

    let mut programs: Vec<Program> = links
        .iter()
        .filter(|link| {
            let name = link
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            !SKIPPED.iter().any(|word| name.contains(word))
        })
        .filter_map(|link| describe(link))
        .collect();
    programs.sort_by_key(|program| program.name.to_lowercase());
    programs.dedup_by_key(|program| program.name.to_lowercase());
    programs
}

#[cfg(not(windows))]
pub fn installed() -> Vec<Program> {
    Vec::new()
}

#[cfg(windows)]
fn collect_links(dir: &Path, depth: u8, links: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth > 0 {
                collect_links(&path, depth - 1, links);
            }
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
        {
            links.push(path);
        }
    }
}
