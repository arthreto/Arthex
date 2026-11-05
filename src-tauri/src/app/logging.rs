use std::{path::Path, sync::OnceLock};

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, Registry, layer::SubscriberExt, reload, util::SubscriberInitExt,
};

/// Changes the level while the app runs, for the "Debug log" setting.
static LEVEL: OnceLock<reload::Handle<EnvFilter, Registry>> = OnceLock::new();

/// Daily log files in the app log folder, the last week kept. The "Debug log" setting adds the
/// detail of what Arthex does (when the overlay opens and closes, and why); `ARTHEX_LOG` overrides
/// both. Logs never contain keystrokes, clipboard content or search queries.
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

    // `ARTHEX_LOG` fixes the level: the setting then leaves it alone.
    let fixed = EnvFilter::try_from_env("ARTHEX_LOG").ok();
    let reloadable = fixed.is_none();
    let (filter, handle) = reload::Layer::new(fixed.unwrap_or_else(|| filter_for(false)));
    let initialized = tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(appender)
                .with_ansi(false),
        )
        .try_init();
    if initialized.is_ok() && reloadable {
        let _ = LEVEL.set(handle);
    }
}

pub fn set_debug(debug: bool) {
    if let Some(handle) = LEVEL.get()
        && let Err(error) = handle.reload(filter_for(debug))
    {
        tracing::warn!(%error, "could not change the log level");
    }
}

fn filter_for(debug: bool) -> EnvFilter {
    EnvFilter::new(if debug {
        "info,arthex_lib=debug"
    } else {
        "info"
    })
}
