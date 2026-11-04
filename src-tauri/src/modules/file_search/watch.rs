//! Follows changes through the system's change notifications (no polling) to add new files and
//! folders as they appear. Removed ones drop out of results by themselves, and the periodic scan
//! tidies up.

use std::path::PathBuf;

use notify::{
    EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{ModifyKind, RenameMode},
};

pub fn start(
    roots: &[PathBuf],
    on_added: impl Fn(Vec<PathBuf>) + Send + 'static,
) -> Option<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        let Ok(event) = result else { return };
        let added = matches!(
            event.kind,
            EventKind::Create(_)
                | EventKind::Modify(ModifyKind::Name(
                    RenameMode::To | RenameMode::Both | RenameMode::Any
                ))
        );
        if added {
            on_added(event.paths);
        }
    })
    .inspect_err(|error| tracing::warn!(%error, "could not watch file changes"))
    .ok()?;
    for root in roots {
        if let Err(error) = watcher.watch(root, RecursiveMode::Recursive) {
            tracing::warn!(%error, root = %root.display(), "could not watch a folder");
        }
    }
    Some(watcher)
}
