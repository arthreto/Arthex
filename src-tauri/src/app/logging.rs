use std::path::Path;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;

/// Daily log files in the app log folder, the last week kept. `ARTHEX_LOG=debug` raises the level.
/// Logs never contain keystrokes, clipboard content or search queries.
pub fn init(dir: &Path) {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("arthex")
        .filename_suffix("log")
        .max_log_files(7)
        .build(dir);
    let appender = match appender {
        Ok(appender) => appender,
        Err(error) => {
            eprintln!("logging disabled: {error}");
            return;
        }
    };

    let filter = EnvFilter::try_from_env("ARTHEX_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(appender)
        .with_ansi(false)
        .try_init();
}
