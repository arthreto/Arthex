use std::{env, fmt::Write as _, fs, path::PathBuf};

/// Commands callable from the interface; each one gets an `allow-<name>` permission.
const COMMANDS: &[&str] = &[
    "settings_get",
    "settings_update",
    "overlay_hide",
    "app_info",
    "app_open_data_dir",
    "app_uninstall",
];

fn main() {
    embed_locales();
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("tauri-build failed");
}

/// Embeds every `src/core/locales/*.json`, so the tray menu speaks the same languages as the UI
/// and a new language file is picked up without touching Rust code.
fn embed_locales() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let dir = manifest_dir.join("../src/core/locales");
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("src/core/locales must exist")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();

    let mut code = String::from("pub const LOCALES: &[(&str, &str)] = &[\n");
    for file in files {
        let code_name = file
            .file_stem()
            .expect("file name")
            .to_string_lossy()
            .into_owned();
        let path = file.display().to_string();
        writeln!(code, "    ({code_name:?}, include_str!({path:?})),").expect("write to String");
    }
    code.push_str("];\n");

    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("locales.rs");
    fs::write(out, code).expect("write locales.rs");
}
