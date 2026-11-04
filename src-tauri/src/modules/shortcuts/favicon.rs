//! Fetches a site's icon once, when its shortcut is added.

use std::time::Duration;

use tauri::Url;

use crate::services::icons;

const CANDIDATES: [&str; 2] = ["/apple-touch-icon.png", "/favicon.ico"];
const MAX_BYTES: u64 = 1024 * 1024;

pub fn fetch(site: &Url) -> Option<String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .max_redirects(3)
        .user_agent(concat!("Arthex/", env!("CARGO_PKG_VERSION")))
        .build()
        .into();
    CANDIDATES.iter().find_map(|path| {
        let url = site.join(path).ok()?;
        let mut response = agent.get(url.as_str()).call().ok()?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit(MAX_BYTES)
            .read_to_vec()
            .ok()?;
        icons::png_data_url(&bytes)
    })
}
