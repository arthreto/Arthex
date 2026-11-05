// Hides the console window of release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebView2 reads this when it creates the webview: a transparent background from the start.
    // SAFETY: no other thread exists yet.
    #[cfg(windows)]
    unsafe {
        std::env::set_var("WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "0");
    }
    arthex_lib::run();
}
