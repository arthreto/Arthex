//! `.arthex` packages. A package is a zip holding a module's files, `files.json` (the SHA-256 of
//! every other file) and `files.json.sig`, the minisign signature of `files.json` made with the
//! update key. Nothing is written to disk before every byte has been checked against it.

use std::{
    collections::BTreeMap,
    fmt::Write as _,
    io::{Cursor, Read},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::app::{AppError, Element, Result, is_module_id};

/// Version of what the app offers modules (`window.__ARTHEX__`). A module built for a newer
/// one is refused; older ones keep working.
pub const HOST_API: u32 = 1;
pub const MAX_PACKAGE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_UNPACKED_BYTES: u64 = 256 * 1024 * 1024;
const MAX_FILES: usize = 2000;
const INDEX: &str = "files.json";
const SIGNATURE: &str = "files.json.sig";
pub const INFO: &str = "module.json";
pub const ENTRY: &str = "index.js";

/// The name and description of a module in one language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Texts {
    pub name: String,
    pub description: String,
}

/// `module.json`, written by `arthex pack`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleInfo {
    pub id: String,
    pub version: String,
    pub api: u32,
    /// `any`, or the system its program was built for (`windows-x86_64`).
    pub platform: String,
    /// Path of its program inside the package.
    #[serde(default)]
    pub native: Option<String>,
    /// The program runs as long as the module is active, not only while it is called.
    #[serde(default)]
    pub background: bool,
    pub elements: Vec<String>,
    /// What the overlay shows before it installs an opened file.
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub tint: String,
    /// By language code.
    #[serde(default)]
    pub texts: BTreeMap<String, Texts>,
}

impl ModuleInfo {
    pub fn elements(&self) -> Vec<Element> {
        self.elements
            .iter()
            .filter_map(|name| Element::parse(name))
            .collect()
    }
}

pub struct Package {
    pub info: ModuleInfo,
    pub files: BTreeMap<String, Vec<u8>>,
}

pub fn current_platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn invalid(reason: impl std::fmt::Display) -> AppError {
    AppError::Invalid(format!("this module package is unusable: {reason}"))
}

/// Opens a package and checks its signature, every file and its `module.json`.
pub fn open(bytes: &[u8], public_key: &str) -> Result<Package> {
    let mut files = unzip(bytes)?;
    let index = files
        .remove(INDEX)
        .ok_or_else(|| invalid("no files.json"))?;
    let signature = files
        .remove(SIGNATURE)
        .ok_or_else(|| invalid("no signature"))?;
    verify_signature(&index, &signature, public_key)?;

    let hashes: BTreeMap<String, String> = serde_json::from_slice(&index).map_err(invalid)?;
    if hashes.keys().ne(files.keys()) {
        return Err(invalid("its files do not match its signed list"));
    }
    for (path, content) in &files {
        if !is_safe_path(path) {
            return Err(invalid(format!("unsafe path {path}")));
        }
        if hashes.get(path) != Some(&sha256(content)) {
            return Err(invalid(format!("{path} was altered")));
        }
    }

    let info: ModuleInfo = files
        .get(INFO)
        .ok_or_else(|| invalid("no module.json"))
        .and_then(|raw| serde_json::from_slice(raw).map_err(invalid))?;
    check_info(&info, &files)?;
    Ok(Package { info, files })
}

fn unzip(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(invalid)?;
    if archive.len() > MAX_FILES {
        return Err(invalid("too many files"));
    }
    let mut files = BTreeMap::new();
    let mut unpacked = 0u64;
    for position in 0..archive.len() {
        let mut entry = archive.by_index(position).map_err(invalid)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_owned();
        let mut content = Vec::new();
        // Reads one byte past the budget, so a lying header cannot hide an oversized file.
        Read::by_ref(&mut entry)
            .take(MAX_UNPACKED_BYTES - unpacked + 1)
            .read_to_end(&mut content)?;
        unpacked += content.len() as u64;
        if unpacked > MAX_UNPACKED_BYTES {
            return Err(invalid("too large once unpacked"));
        }
        if files.insert(name, content).is_some() {
            return Err(invalid("a file appears twice"));
        }
    }
    Ok(files)
}

/// Same format as the updater: base64 of the minisign public key file and of the signature file.
pub fn verify_signature(data: &[u8], signature: &[u8], public_key: &str) -> Result<()> {
    let decode = |text: &[u8]| {
        STANDARD
            .decode(text.trim_ascii())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    };
    let key = decode(public_key.as_bytes())
        .and_then(|text| PublicKey::decode(&text).ok())
        .ok_or_else(|| AppError::Failed("the update key is unusable".into()))?;
    let signature = decode(signature)
        .and_then(|text| Signature::decode(&text).ok())
        .ok_or_else(|| invalid("unreadable signature"))?;
    key.verify(data, &signature, true)
        .map_err(|_| invalid("its signature does not match"))
}

fn check_info(info: &ModuleInfo, files: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    if !is_module_id(&info.id) {
        return Err(invalid("bad id"));
    }
    semver::Version::parse(&info.version).map_err(|_| invalid("bad version"))?;
    if info.api == 0 {
        return Err(invalid("bad api version"));
    }
    if info.api > HOST_API {
        return Err(AppError::Outdated);
    }
    if info.platform != "any" && info.platform != current_platform() {
        return Err(AppError::Unsupported);
    }
    if info
        .elements
        .iter()
        .any(|name| Element::parse(name).is_none())
    {
        return Err(invalid("unknown element"));
    }
    if !files.contains_key(ENTRY) {
        return Err(invalid("no index.js"));
    }
    if let Some(native) = &info.native
        && (info.platform == "any" || !files.contains_key(native))
    {
        return Err(invalid("its program is missing or not built for a system"));
    }
    Ok(())
}

/// Relative, `/`-separated, plain names only: nothing can land outside the module's folder.
pub fn is_safe_path(path: &str) -> bool {
    path.len() <= 200
        && path.split('/').all(|segment| {
            !segment.is_empty()
                && !segment.starts_with('.')
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
}

fn sha256(content: &[u8]) -> String {
    Sha256::digest(content)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::*;

    /// Made once with `tauri signer` and a throwaway key, for these tests only.
    const KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDI3MjdERTExNDBBMkQzQjcKUldTMzA2SkFFZDRuSjY0TElmcEkwRGovTGxwNm40cmZxSURPZUhGQzM4M0hPTXB1MmJWQVZEOS8K";
    const INDEX_JSON: &str = r#"{"index.js":"9f085b1079ab38f776bbb3930dfd067a838ca3e0483aff8625f88837e8ed964c","module.json":"3603e627daffc179161ff9e8020fb0c8eef36c63b06844e9252c3f648d4c4e23"}"#;
    const SIGNATURE_TEXT: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVTMzA2SkFFZDRuSjd2SEk3ZG5HczF5U2Rjc3lRaFk5Mm9ZbkZDWEh2eE8yR2pjK2ZKWm1CR3g1bGlGMFJMU1JKWTRhVXNqQnJCbHRpWDdEWUFqSkZPMTRyOStXU0xoMUFjPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkzODM4NzYwCWZpbGU6ZmlsZXMuanNvbgphaUtnRVo4d3lZVU1zSXk0TFZROUpCMlBvREFSMGNydkFoaEtOUVNzKzJ6OXFiQW9iMEpOK0JqUWE2VXRrQUh4eTF4cGNwc0JxY3J3YXhwTmJPbmZEUT09Cg==";
    const MODULE_JSON: &str =
        r#"{"id":"sample","version":"1.0.0","api":1,"platform":"any","elements":["tool"]}"#;
    const INDEX_JS: &str = "export default {};";

    fn zip(files: &[(&str, &str)]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, content) in files {
            writer
                .start_file(*name, SimpleFileOptions::default())
                .unwrap();
            writer.write_all(content.as_bytes()).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn signed(extra: &[(&str, &str)], index_js: &str) -> Vec<u8> {
        let mut files = vec![
            ("files.json", INDEX_JSON),
            ("files.json.sig", SIGNATURE_TEXT),
            ("module.json", MODULE_JSON),
            ("index.js", index_js),
        ];
        files.extend_from_slice(extra);
        zip(&files)
    }

    #[test]
    fn opens_a_signed_package() {
        let package = open(&signed(&[], INDEX_JS), KEY).unwrap();
        assert_eq!(package.info.id, "sample");
        assert_eq!(package.info.elements(), [Element::Tool]);
        assert_eq!(package.files.len(), 2);
    }

    #[test]
    fn refuses_altered_or_extra_files() {
        assert!(open(&signed(&[], "alert(1)"), KEY).is_err());
        assert!(open(&signed(&[("extra.js", "")], INDEX_JS), KEY).is_err());
    }

    #[test]
    fn refuses_another_key_or_no_signature() {
        let other = STANDARD.encode(
            "untrusted comment: minisign public key: 0000000000000000\nRWQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n",
        );
        assert!(open(&signed(&[], INDEX_JS), &other).is_err());
        let unsigned = zip(&[
            ("files.json", INDEX_JSON),
            ("module.json", MODULE_JSON),
            ("index.js", INDEX_JS),
        ]);
        assert!(open(&unsigned, KEY).is_err());
    }

    #[test]
    fn checks_module_info() {
        let mut files = BTreeMap::from([(ENTRY.to_string(), Vec::new())]);
        let info = |api: u32, platform: &str| ModuleInfo {
            id: "sample".into(),
            version: "1.0.0".into(),
            api,
            platform: platform.into(),
            elements: vec!["page".into()],
            ..ModuleInfo::default()
        };
        assert!(check_info(&info(1, "any"), &files).is_ok());
        assert!(matches!(
            check_info(&info(HOST_API + 1, "any"), &files),
            Err(AppError::Outdated)
        ));
        assert!(matches!(
            check_info(&info(1, "plan9-mips"), &files),
            Err(AppError::Unsupported)
        ));
        let mut native = info(1, &current_platform());
        native.native = Some("native/sample".into());
        assert!(check_info(&native, &files).is_err());
        files.insert("native/sample".into(), Vec::new());
        assert!(check_info(&native, &files).is_ok());
    }

    #[test]
    fn accepts_plain_relative_paths_only() {
        assert!(is_safe_path("index.js"));
        assert!(is_safe_path("native/color-picker.exe"));
        for bad in [
            "", "../x", "a//b", "/abs", "a\\b", ".hidden", "a/./b", "C:x",
        ] {
            assert!(!is_safe_path(bad), "{bad}");
        }
    }
}
