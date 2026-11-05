//! The logo of Arthex on the Windows taskbar, which opens the overlay.
//!
//! Windows has no API to add anything to its taskbar or to its search box, so this is a window of
//! our own laid over the taskbar. The taskbar owns it: it stays above the taskbar and goes away
//! with it. Its place comes from UI Automation and is read again only when Windows reports a
//! change.

mod locate;
mod paint;
mod place;
mod window;

pub use window::set_enabled;
