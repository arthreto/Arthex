use std::{env, fmt::Write as _, fs, path::PathBuf};

fn main() {
    embed_locales();
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(granted_commands())),
    )
    .expect("tauri-build failed");
}

/// The capability files are the single list of commands: every `allow-<name>` permission they
/// grant declares the command `<name>`, so adding a command never means editing this file.
fn granted_commands() -> &'static [&'static str] {
    let dir = manifest_dir().join("capabilities");
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut commands = Vec::new();
    for entry in fs::read_dir(&dir).expect("capabilities/ must exist") {
        let path = entry.expect("readable capabilities/").path();
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("readable capability"))
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let permissions = json["permissions"].as_array().into_iter().flatten();
        for permission in permissions.filter_map(serde_json::Value::as_str) {
            if let Some(name) = permission.strip_prefix("allow-") {
                let name = name.replace('-', "_");
                if !commands.contains(&name) {
                    commands.push(name);
                }
            }
        }
    }
    // tauri-build wants 'static strings; leaking is harmless in a build script.
    Box::leak(
        commands
            .into_iter()
            .map(|name| &*Box::leak(name.into_boxed_str()))
            .collect(),
    )
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
}

/// Embeds every `src/core/locales/*.json`, so the tray menu speaks the same languages as the UI
/// and a new language file is picked up without touching Rust code.
fn embed_locales() {
    let dir = manifest_dir().join("../src/core/locales");
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
