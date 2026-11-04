//! Follows changes through the system's change notifications (no polling). Changes are gathered
//! for a moment, then each changed path is checked on disk once.

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// A batch goes out after this much quiet, or after `LONGEST` during a burst of changes.
const QUIET: Duration = Duration::from_secs(1);
const LONGEST: Duration = Duration::from_secs(5);

pub fn start(
    roots: &[PathBuf],
    on_changes: impl Fn(Vec<PathBuf>) + Send + 'static,
) -> Option<RecommendedWatcher> {
    let (sender, receiver) = mpsc::channel::<Vec<PathBuf>>();
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result
            && !matches!(event.kind, EventKind::Access(_) | EventKind::Other)
        {
            sender.send(event.paths).ok();
        }
    })
    .inspect_err(|error| tracing::warn!(%error, "could not watch file changes"))
    .ok()?;
    thread::Builder::new()
        .name("file-index-changes".into())
        .spawn(move || gather(&receiver, &on_changes))
        .inspect_err(|error| tracing::warn!(%error, "could not follow file changes"))
        .ok()?;
    for root in roots {
        if let Err(error) = watcher.watch(root, RecursiveMode::Recursive) {
            tracing::warn!(%error, root = %root.display(), "could not watch a folder");
        }
    }
    Some(watcher)
}

/// Ends when the watcher is dropped, which closes the channel.
fn gather(receiver: &Receiver<Vec<PathBuf>>, on_changes: &impl Fn(Vec<PathBuf>)) {
    while let Ok(first) = receiver.recv() {
        let started = Instant::now();
        let mut paths: HashSet<PathBuf> = first.into_iter().collect();
        let open = loop {
            match receiver.recv_timeout(QUIET.min(LONGEST.saturating_sub(started.elapsed()))) {
                Ok(more) => paths.extend(more),
                Err(RecvTimeoutError::Timeout) => break true,
                Err(RecvTimeoutError::Disconnected) => break false,
            }
        };
        on_changes(paths.into_iter().collect());
        if !open {
            return;
        }
    }
}
