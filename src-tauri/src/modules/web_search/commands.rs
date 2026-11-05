use tauri::{AppHandle, Url};
use tauri_plugin_opener::OpenerExt;

use crate::app::{AppError, Result, overlay};

/// Only `http` and `https`: the address comes from what the user typed.
fn web_address(url: &str) -> Result<Url> {
    Url::parse(url)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https") && url.host().is_some())
        .ok_or_else(|| AppError::Invalid("not a web address".into()))
}

#[tauri::command]
pub fn web_search_open(app: AppHandle, url: String) -> Result<()> {
    let url = web_address(&url)?;
    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|error| AppError::Failed(error.to_string()))?;
    overlay::hide(&app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_web_addresses_only() {
        assert!(web_address("https://www.google.com/search?q=chat").is_ok());
        assert!(web_address("http://github.com").is_ok());
        for bad in [
            "file:///C:/Windows",
            "javascript:alert(1)",
            "ms-settings:",
            "https://",
            "github.com",
        ] {
            assert!(web_address(bad).is_err(), "{bad}");
        }
    }
}
