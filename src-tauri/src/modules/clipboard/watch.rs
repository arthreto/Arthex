//! Follows the clipboard with `AddClipboardFormatListener`: Windows sends a message on each
//! change, so nothing is polled. Content that password managers mark as private is skipped.

use std::{iter, ptr, sync::OnceLock, thread, time::Duration};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::{
        DataExchange::{
            AddClipboardFormatListener, CloseClipboard, GetClipboardData,
            IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW,
        },
        LibraryLoader::GetModuleHandleW,
        Memory::{GlobalLock, GlobalSize, GlobalUnlock},
        Ole::CF_UNICODETEXT,
    },
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, GetMessageW, HWND_MESSAGE, MSG, RegisterClassW,
        WM_CLIPBOARDUPDATE, WNDCLASSW,
    },
};

struct Handlers {
    listening: Box<dyn Fn() -> bool + Send + Sync>,
    on_text: Box<dyn Fn(String) + Send + Sync>,
}

static HANDLERS: OnceLock<Handlers> = OnceLock::new();

/// Formats that clipboard tools agree to honour: set by password managers and by apps that
/// want a copy kept out of histories.
const PRIVATE_FORMATS: [&str; 2] = [
    "ExcludeClipboardContentFromMonitorProcessing",
    "Clipboard Viewer Ignore",
];

pub fn start(
    listening: impl Fn() -> bool + Send + Sync + 'static,
    on_text: impl Fn(String) + Send + Sync + 'static,
) {
    let handlers = Handlers {
        listening: Box::new(listening),
        on_text: Box::new(on_text),
    };
    if HANDLERS.set(handlers).is_err() {
        return;
    }
    let spawned = thread::Builder::new().name("clipboard".into()).spawn(|| {
        let class = wide("ArthexClipboard");
        // SAFETY: a message-only window and its loop, living on this thread for the life of
        // the process; `class` outlives the calls that read it.
        unsafe {
            let instance = GetModuleHandleW(ptr::null());
            let mut window_class: WNDCLASSW = std::mem::zeroed();
            window_class.lpfnWndProc = Some(on_message);
            window_class.hInstance = instance;
            window_class.lpszClassName = class.as_ptr();
            RegisterClassW(&raw const window_class);
            let window = CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                ptr::null_mut(),
                instance,
                ptr::null(),
            );
            if window.is_null() || AddClipboardFormatListener(window) == 0 {
                tracing::error!("could not listen to the clipboard");
                return;
            }
            let mut message: MSG = std::mem::zeroed();
            while GetMessageW(&raw mut message, ptr::null_mut(), 0, 0) > 0 {
                windows_sys::Win32::UI::WindowsAndMessaging::DispatchMessageW(&raw const message);
            }
        }
    });
    if let Err(error) = spawned {
        tracing::error!(%error, "could not start the clipboard listener");
    }
}

unsafe extern "system" fn on_message(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_CLIPBOARDUPDATE {
        if let Some(handlers) = HANDLERS.get()
            && (handlers.listening)()
            && let Some(text) = read_text(window)
        {
            (handlers.on_text)(text);
        }
        return 0;
    }
    // SAFETY: every other message goes to the default handling.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

/// The text on the clipboard, unless it is marked private. Another app may hold the clipboard
/// for a moment, so opening it is retried briefly.
fn read_text(window: HWND) -> Option<String> {
    // SAFETY: the clipboard is opened, read and closed on this thread; the global memory is
    // only read between `GlobalLock` and `GlobalUnlock`, within the size Windows reports.
    unsafe {
        let mut opened = false;
        for _ in 0..5 {
            if OpenClipboard(window) != 0 {
                opened = true;
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        if !opened {
            return None;
        }
        let text = if is_private() { None } else { unicode_text() };
        CloseClipboard();
        text
    }
}

/// SAFETY: the clipboard must be open.
unsafe fn is_private() -> bool {
    let marked = PRIVATE_FORMATS.iter().any(|name| {
        let name = wide(name);
        // SAFETY: `name` is a terminated UTF-16 string.
        let format = unsafe { RegisterClipboardFormatW(name.as_ptr()) };
        format != 0 && unsafe { IsClipboardFormatAvailable(format) } != 0
    });
    marked || unsafe { history_refused() }
}

/// `CanIncludeInClipboardHistory` set to 0 asks clipboard histories not to keep the content.
///
/// SAFETY: the clipboard must be open.
unsafe fn history_refused() -> bool {
    let name = wide("CanIncludeInClipboardHistory");
    // SAFETY: `name` is terminated; the data is read within its reported size.
    unsafe {
        let format = RegisterClipboardFormatW(name.as_ptr());
        if format == 0 || IsClipboardFormatAvailable(format) == 0 {
            return false;
        }
        let handle = GetClipboardData(format);
        if handle.is_null() || GlobalSize(handle) < 4 {
            return true;
        }
        let data = GlobalLock(handle).cast::<u32>();
        if data.is_null() {
            return true;
        }
        let allowed = data.read_unaligned();
        GlobalUnlock(handle);
        allowed == 0
    }
}

/// SAFETY: the clipboard must be open.
unsafe fn unicode_text() -> Option<String> {
    let format = u32::from(CF_UNICODETEXT);
    // SAFETY: the text is read within the size of its global memory block.
    unsafe {
        if IsClipboardFormatAvailable(format) == 0 {
            return None;
        }
        let handle = GetClipboardData(format);
        if handle.is_null() {
            return None;
        }
        let units = GlobalSize(handle) / 2;
        // UTF-8 never takes fewer bytes than UTF-16 units, so this many units is too large.
        if units > super::MAX_BYTES + 1 {
            return None;
        }
        let data = GlobalLock(handle).cast::<u16>();
        if data.is_null() {
            return None;
        }
        let all = std::slice::from_raw_parts(data, units);
        let length = all.iter().position(|&unit| unit == 0).unwrap_or(units);
        let text = String::from_utf16_lossy(&all[..length]);
        GlobalUnlock(handle);
        Some(text)
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(iter::once(0)).collect()
}
