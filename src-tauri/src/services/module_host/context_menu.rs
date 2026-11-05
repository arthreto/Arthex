//! Actions of modules in the right-click menu of Explorer. Each is a verb of the current user's
//! registry, so no admin rights are needed. Windows 11 shows such verbs under "Show more options":
//! its first menu takes only commands of a signed, packaged app.
#![cfg_attr(not(windows), allow(dead_code))]

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    thread,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

use super::{ModuleHost, package::is_safe_path, program};
use crate::app::{self, Element, Settings, is_module_id, overlay};

const VERB_PREFIX: &str = "Arthex.";
const ARGUMENT: &str = "--module-action";
const METHOD: &str = "contextMenu";
/// Sent when an action asks for its module's page; the interface then asks `modules_pending_page`.
pub const OPEN_PAGE: &str = "modules://open-page";
pub const MAX_ACTIONS: usize = 16;
const MAX_SUBMENU: usize = 32;
const MAX_LABEL_CHARS: usize = 64;
const MAX_EXTENSION_CHARS: usize = 16;
const FALLBACK_LANGUAGE: &str = "en";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    File,
    Folder,
    /// The empty space of an open folder.
    FolderBackground,
    Drive,
}

/// An entry of a submenu. Its label is a name such as a format, the same in every language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubAction {
    pub id: String,
    pub label: String,
}

/// An entry of `contextMenu` in `module.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    pub targets: Vec<Target>,
    /// For files: only these, as `.png`. All of them when empty.
    #[serde(default)]
    pub extensions: Vec<String>,
    /// A `.ico` inside the module. The icon of Arthex when absent.
    #[serde(default)]
    pub icon: Option<String>,
    /// Choices shown under this entry, which then runs nothing by itself.
    #[serde(default)]
    pub submenu: Vec<SubAction>,
    /// Opens the overlay on the module's page when the action runs.
    #[serde(default)]
    pub open_page: bool,
    /// By language code.
    pub labels: BTreeMap<String, String>,
}

impl Action {
    /// Everything here ends up in the registry and in a command line, so nothing is taken on trust.
    pub fn is_valid(&self) -> bool {
        is_module_id(&self.id)
            && !self.targets.is_empty()
            && self
                .extensions
                .iter()
                .all(|extension| is_extension(extension))
            && self.icon.as_deref().is_none_or(is_icon_path)
            && !self.labels.is_empty()
            && self.labels.values().all(|label| is_label(label))
            && self.submenu.len() <= MAX_SUBMENU
            && self
                .submenu
                .iter()
                .all(|entry| is_module_id(&entry.id) && is_label(&entry.label))
    }

    /// `choice` is the entry of the submenu clicked, when the action has one.
    fn runs(&self, choice: Option<&str>) -> bool {
        match choice {
            Some(choice) => self.submenu.iter().any(|entry| entry.id == choice),
            None => self.submenu.is_empty(),
        }
    }

    fn label(&self, language: &str) -> &str {
        let base = language.split(['-', '_']).next().unwrap_or(language);
        [language, base, FALLBACK_LANGUAGE]
            .iter()
            .find_map(|code| self.labels.get(*code))
            .or_else(|| self.labels.values().next())
            .map_or("", String::as_str)
    }
}

fn is_extension(value: &str) -> bool {
    value.len() <= MAX_EXTENSION_CHARS
        && value.strip_prefix('.').is_some_and(|name| {
            !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
}

fn is_icon_path(value: &str) -> bool {
    is_safe_path(value) && value.to_ascii_lowercase().ends_with(".ico")
}

fn is_label(value: &str) -> bool {
    let length = value.chars().count();
    (1..=MAX_LABEL_CHARS).contains(&length) && !value.chars().any(char::is_control)
}

#[derive(Debug, PartialEq, Eq)]
enum Runs {
    Command(String),
    /// `(key name, label, command)`. Explorer sorts them by key name, so it starts with their rank.
    Submenu(Vec<(String, String, String)>),
}

/// A verb to write, by its key under `Software\Classes`.
#[derive(Debug, PartialEq, Eq)]
struct Verb {
    key: String,
    label: String,
    icon: String,
    runs: Runs,
}

fn classes_of(action: &Action) -> Vec<(String, &'static str)> {
    let mut classes = Vec::new();
    for target in &action.targets {
        match target {
            Target::File if action.extensions.is_empty() => classes.push(("*".to_string(), "%1")),
            Target::File => classes.extend(action.extensions.iter().map(|extension| {
                (
                    format!(r"SystemFileAssociations\{}", extension.to_ascii_lowercase()),
                    "%1",
                )
            })),
            Target::Folder => classes.push(("Directory".to_string(), "%1")),
            Target::FolderBackground => classes.push((r"Directory\Background".to_string(), "%V")),
            Target::Drive => classes.push(("Drive".to_string(), "%1")),
        }
    }
    classes
}

fn command(program: &str, module: &str, action: &str, argument: &str) -> String {
    format!("\"{program}\" {ARGUMENT} {module} {action} \"{argument}\"")
}

fn verbs_of(
    module: &str,
    dir: &Path,
    actions: &[Action],
    program: &Path,
    language: &str,
) -> Vec<Verb> {
    let program = program.display().to_string();
    let program_text = program.as_str();
    actions
        .iter()
        .filter(|action| action.is_valid())
        .flat_map(|action| {
            let icon = action.icon.as_ref().map_or_else(
                || format!("\"{program_text}\",0"),
                |icon| dir.join(icon).display().to_string(),
            );
            classes_of(action)
                .into_iter()
                .map(move |(class, argument)| Verb {
                    key: format!(r"{class}\shell\{VERB_PREFIX}{module}.{}", action.id),
                    label: action.label(language).to_string(),
                    icon: icon.clone(),
                    runs: if action.submenu.is_empty() {
                        Runs::Command(command(program_text, module, &action.id, argument))
                    } else {
                        Runs::Submenu(
                            action
                                .submenu
                                .iter()
                                .enumerate()
                                .map(|(rank, entry)| {
                                    let token = format!("{}:{}", action.id, entry.id);
                                    (
                                        format!("{rank:02}-{}", entry.id),
                                        entry.label.clone(),
                                        command(program_text, module, &token, argument),
                                    )
                                })
                                .collect(),
                        )
                    },
                })
        })
        .collect()
}

#[cfg(windows)]
mod registry {
    use std::io;

    use winreg::{RegKey, enums::HKEY_CURRENT_USER};

    use super::{Runs, VERB_PREFIX, Verb};

    const CLASSES: &str = r"Software\Classes";
    const ROOTS: [&str; 4] = ["*", "Directory", r"Directory\Background", "Drive"];
    const FILE_TYPES: &str = "SystemFileAssociations";
    /// Keeps the entry when many files are selected, where Explorer otherwise hides it.
    const MULTI_SELECT: (&str, &str) = ("MultiSelectModel", "Player");

    pub fn classes() -> io::Result<RegKey> {
        Ok(RegKey::predef(HKEY_CURRENT_USER).create_subkey(CLASSES)?.0)
    }

    fn written(classes: &RegKey) -> Vec<String> {
        let file_types = classes.open_subkey(FILE_TYPES).map_or_else(
            |_| Vec::new(),
            |key| {
                key.enum_keys()
                    .flatten()
                    .map(|extension| format!(r"{FILE_TYPES}\{extension}"))
                    .collect()
            },
        );
        ROOTS
            .iter()
            .map(ToString::to_string)
            .chain(file_types)
            .flat_map(|root| {
                let shell = format!(r"{root}\shell");
                let names: Vec<String> = classes
                    .open_subkey(&shell)
                    .map_or_else(|_| Vec::new(), |key| key.enum_keys().flatten().collect());
                names
                    .into_iter()
                    .filter(|name| name.starts_with(VERB_PREFIX))
                    .map(move |name| format!(r"{shell}\{name}"))
            })
            .collect()
    }

    /// Leaves exactly `verbs`: what a module no longer offers disappears with what it offers now.
    pub fn apply(classes: &RegKey, verbs: &[Verb]) -> io::Result<()> {
        for key in written(classes) {
            if !verbs.iter().any(|verb| verb.key == key) {
                classes.delete_subkey_all(&key)?;
            }
        }
        for verb in verbs {
            // From scratch: a verb that had a submenu must not keep it.
            if classes.open_subkey(&verb.key).is_ok() {
                classes.delete_subkey_all(&verb.key)?;
            }
            let (key, _) = classes.create_subkey(&verb.key)?;
            key.set_value("MUIVerb", &verb.label)?;
            key.set_value("Icon", &verb.icon)?;
            key.set_value(MULTI_SELECT.0, &MULTI_SELECT.1)?;
            match &verb.runs {
                Runs::Command(command) => {
                    key.create_subkey("command")?.0.set_value("", command)?;
                }
                Runs::Submenu(entries) => {
                    key.set_value("SubCommands", &"")?;
                    for (name, label, command) in entries {
                        let (entry, _) = key.create_subkey(format!(r"shell\{name}"))?;
                        entry.set_value("MUIVerb", label)?;
                        entry.set_value(MULTI_SELECT.0, &MULTI_SELECT.1)?;
                        entry.create_subkey("command")?.0.set_value("", command)?;
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn verb(key: &str) -> Verb {
            Verb {
                key: key.to_string(),
                label: "Label".to_string(),
                icon: "icon.ico".to_string(),
                runs: Runs::Command("\"a.exe\" \"%1\"".to_string()),
            }
        }

        #[test]
        fn leaves_exactly_what_is_wanted() {
            let name = format!(r"Software\ArthexTest-{}", std::process::id());
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let (root, _) = hkcu.create_subkey(&name).unwrap();
            let keep = r"*\shell\Arthex.a.keep";
            let gone = r"SystemFileAssociations\.png\shell\Arthex.a.gone";
            root.create_subkey(r"*\shell\SomeoneElse").unwrap();

            apply(&root, &[verb(keep), verb(gone)]).unwrap();
            assert_eq!(written(&root).len(), 2);
            let command: String = root
                .open_subkey(format!(r"{keep}\command"))
                .unwrap()
                .get_value("")
                .unwrap();
            assert_eq!(command, "\"a.exe\" \"%1\"");

            let menu = Verb {
                runs: Runs::Submenu(vec![(
                    "00-png".to_string(),
                    "PNG".to_string(),
                    "\"a.exe\" png".to_string(),
                )]),
                ..verb(keep)
            };
            apply(&root, &[menu]).unwrap();
            assert!(root.open_subkey(format!(r"{keep}\command")).is_err());
            let entry: String = root
                .open_subkey(format!(r"{keep}\shell\00-png\command"))
                .unwrap()
                .get_value("")
                .unwrap();
            assert_eq!(entry, "\"a.exe\" png");
            apply(&root, &[verb(keep), verb(gone)]).unwrap();

            apply(&root, &[verb(keep)]).unwrap();
            assert_eq!(written(&root), [keep]);
            assert!(root.open_subkey(r"*\shell\SomeoneElse").is_ok());

            apply(&root, &[]).unwrap();
            assert_eq!(written(&root), Vec::<String>::new());
            hkcu.delete_subkey_all(&name).unwrap();
        }
    }
}

/// Writes the actions of the active modules in the language of the interface.
#[cfg(windows)]
pub fn sync(app: &AppHandle, settings: &Settings) {
    let host = app.state::<ModuleHost>();
    let language = settings
        .language
        .clone()
        .or_else(app::i18n::system_locale)
        .unwrap_or_else(|| FALLBACK_LANGUAGE.to_string());
    let applied = std::env::current_exe().and_then(|program| {
        let verbs: Vec<Verb> = host
            .list()
            .iter()
            .filter(|module| settings.is_element_enabled(&module.id, Element::ContextMenu))
            .flat_map(|module| {
                verbs_of(
                    &module.id,
                    &module.dir,
                    &module.info.context_menu,
                    &program,
                    &language,
                )
            })
            .collect();
        registry::apply(&registry::classes()?, &verbs)
    });
    if let Err(error) = applied {
        tracing::warn!(%error, "could not update the right-click menu");
    }
}

#[cfg(not(windows))]
pub fn sync(_: &AppHandle, _: &Settings) {}

/// For the uninstaller: a verb left behind would point to a program that no longer exists.
#[cfg(windows)]
pub fn clear() {
    if let Err(error) = registry::classes().and_then(|classes| registry::apply(&classes, &[])) {
        eprintln!("could not clear the right-click menu: {error}");
    }
}

fn parse(args: &[String]) -> Option<(&str, &str, &str)> {
    let at = args.iter().position(|arg| arg == ARGUMENT)?;
    match (args.get(at + 1), args.get(at + 2), args.get(at + 3)) {
        (Some(module), Some(action), Some(path)) => Some((module, action, path)),
        _ => None,
    }
}

/// `canonicalize` gives `\\?\C:\…`, which many programs do not read.
fn plain_path(path: &Path) -> PathBuf {
    let text = path.display().to_string();
    if let Some(share) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{share}"));
    }
    text.strip_prefix(r"\\?\")
        .map_or_else(|| path.to_path_buf(), PathBuf::from)
}

fn run(app: &AppHandle, module: &str, action: &str, path: &str) -> app::Result<()> {
    let host = app
        .try_state::<ModuleHost>()
        .ok_or_else(|| app::AppError::Failed("modules are not ready".into()))?;
    let (id, choice) = action
        .split_once(':')
        .map_or((action, None), |(id, choice)| (id, Some(choice)));
    let entry = host.get(module).and_then(|installed| {
        installed
            .info
            .context_menu
            .into_iter()
            .find(|entry| entry.id == id && entry.is_valid() && entry.runs(choice))
    });
    let enabled = app::settings(app).is_element_enabled(module, Element::ContextMenu);
    let (Some(entry), true) = (entry, enabled) else {
        return Err(app::AppError::Invalid(format!(
            "{module} has no action {action}"
        )));
    };
    let path = plain_path(&Path::new(path).canonicalize()?);
    program::call(
        app,
        module,
        METHOD,
        json!({ "action": action, "path": path.display().to_string() }),
    )?;
    if entry.open_page {
        host.set_pending_page(module);
        overlay::show(app);
        app.emit(OPEN_PAGE, ())?;
    }
    Ok(())
}

/// Runs the action a click in Explorer asked for. `true` when these arguments came from one, in
/// which case the overlay stays closed: the module's program decides what to show.
pub fn run_from_args(app: &AppHandle, args: &[String]) -> bool {
    if !args.iter().any(|arg| arg == ARGUMENT) {
        return false;
    }
    let Some((module, action, path)) = parse(args) else {
        return true;
    };
    let (app, module, action, path) = (
        app.clone(),
        module.to_string(),
        action.to_string(),
        path.to_string(),
    );
    thread::spawn(move || {
        if let Err(error) = run(&app, &module, &action, &path) {
            tracing::warn!(%module, %action, %error, "a right-click action failed");
        }
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action() -> Action {
        Action {
            id: "compress".into(),
            targets: vec![Target::File],
            extensions: Vec::new(),
            icon: None,
            submenu: Vec::new(),
            open_page: false,
            labels: BTreeMap::from([
                ("en".to_string(), "Compress".to_string()),
                ("fr".to_string(), "Compresser".to_string()),
            ]),
        }
    }

    #[test]
    fn refuses_what_could_escape_a_registry_key_or_a_command_line() {
        assert!(action().is_valid());
        let with = |change: fn(&mut Action)| {
            let mut changed = action();
            change(&mut changed);
            changed.is_valid()
        };
        assert!(!with(|a| a.id = "a\" --evil \"".into()));
        assert!(!with(|a| a.id = r"a\b".into()));
        assert!(!with(|a| a.targets.clear()));
        assert!(!with(|a| a.extensions = vec![r".png\shell".into()]));
        assert!(!with(|a| a.extensions = vec!["png".into()]));
        assert!(!with(|a| a.icon = Some("../icon.ico".into())));
        assert!(!with(|a| a.icon = Some("icon.png".into())));
        assert!(!with(|a| a.labels.clear()));
        assert!(!with(|a| {
            a.labels.insert("en".into(), "a\nb".into());
        }));
        assert!(with(|a| a.icon = Some("assets/icon.ico".into())));
        assert!(with(|a| a.extensions = vec![".PNG".into()]));
    }

    #[test]
    fn picks_the_label_of_the_language_then_english() {
        let action = action();
        assert_eq!(action.label("fr"), "Compresser");
        assert_eq!(action.label("fr-CA"), "Compresser");
        assert_eq!(action.label("de"), "Compress");
    }

    #[test]
    fn writes_a_verb_per_target_with_the_icon_of_arthex_by_default() {
        let mut action = action();
        action.targets = vec![
            Target::File,
            Target::Folder,
            Target::FolderBackground,
            Target::Drive,
        ];
        let verbs = verbs_of(
            "tools",
            Path::new("modules/tools"),
            &[action],
            Path::new("arthex.exe"),
            "fr",
        );
        let keys: Vec<&str> = verbs.iter().map(|verb| verb.key.as_str()).collect();
        assert_eq!(
            keys,
            [
                r"*\shell\Arthex.tools.compress",
                r"Directory\shell\Arthex.tools.compress",
                r"Directory\Background\shell\Arthex.tools.compress",
                r"Drive\shell\Arthex.tools.compress",
            ]
        );
        assert_eq!(verbs[0].label, "Compresser");
        assert_eq!(verbs[0].icon, "\"arthex.exe\",0");
        assert_eq!(
            verbs[0].runs,
            Runs::Command("\"arthex.exe\" --module-action tools compress \"%1\"".to_string())
        );
        assert_eq!(
            verbs[2].runs,
            Runs::Command("\"arthex.exe\" --module-action tools compress \"%V\"".to_string())
        );
    }

    #[test]
    fn narrows_files_by_extension_and_uses_the_icon_of_the_module() {
        let mut action = action();
        action.extensions = vec![".PNG".into(), ".jpg".into()];
        action.icon = Some("assets/compress.ico".into());
        let verbs = verbs_of(
            "tools",
            Path::new("dir"),
            &[action],
            Path::new("a.exe"),
            "en",
        );
        assert_eq!(
            verbs[0].key,
            r"SystemFileAssociations\.png\shell\Arthex.tools.compress"
        );
        assert_eq!(
            verbs[1].key,
            r"SystemFileAssociations\.jpg\shell\Arthex.tools.compress"
        );
        assert_eq!(
            verbs[0].icon,
            Path::new("dir")
                .join("assets/compress.ico")
                .display()
                .to_string()
        );
    }

    #[test]
    fn a_submenu_runs_its_entries_in_their_order() {
        let mut action = action();
        action.submenu = vec![
            SubAction {
                id: "png".into(),
                label: "PNG".into(),
            },
            SubAction {
                id: "jpeg".into(),
                label: "JPEG".into(),
            },
        ];
        assert!(action.is_valid());
        assert!(action.runs(Some("jpeg")));
        assert!(!action.runs(Some("gif")));
        assert!(!action.runs(None));
        let verbs = verbs_of(
            "tools",
            Path::new("d"),
            &[action.clone()],
            Path::new("a.exe"),
            "en",
        );
        assert_eq!(
            verbs[0].runs,
            Runs::Submenu(vec![
                (
                    "00-png".to_string(),
                    "PNG".to_string(),
                    "\"a.exe\" --module-action tools compress:png \"%1\"".to_string(),
                ),
                (
                    "01-jpeg".to_string(),
                    "JPEG".to_string(),
                    "\"a.exe\" --module-action tools compress:jpeg \"%1\"".to_string(),
                ),
            ])
        );
        action.submenu[0].id = "a b".into();
        assert!(!action.is_valid());
    }

    #[test]
    fn leaves_out_an_invalid_action() {
        let mut bad = action();
        bad.id = "not valid".into();
        assert_eq!(
            verbs_of("tools", Path::new("d"), &[bad], Path::new("a.exe"), "en"),
            Vec::new()
        );
    }

    #[test]
    fn reads_an_action_from_the_arguments() {
        let args = |list: &[&str]| list.iter().map(ToString::to_string).collect::<Vec<_>>();
        let full = args(&["arthex.exe", ARGUMENT, "tools", "compress", r"C:\a b.png"]);
        assert_eq!(parse(&full), Some(("tools", "compress", r"C:\a b.png")));
        assert_eq!(parse(&args(&["arthex.exe", ARGUMENT, "tools"])), None);
        assert_eq!(parse(&args(&["arthex.exe", "--autostart"])), None);
    }

    #[test]
    fn gives_paths_other_programs_can_read() {
        assert_eq!(plain_path(Path::new(r"\\?\C:\a")), PathBuf::from(r"C:\a"));
        assert_eq!(
            plain_path(Path::new(r"\\?\UNC\server\share")),
            PathBuf::from(r"\\server\share")
        );
        assert_eq!(plain_path(Path::new("/tmp/a")), PathBuf::from("/tmp/a"));
    }
}
