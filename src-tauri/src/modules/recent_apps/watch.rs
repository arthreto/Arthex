//! Follows the foreground window with `SetWinEventHook`: Windows calls back on each switch, so
//! nothing is polled. Only the program behind the window is read, never its title or content.

use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::Path,
    ptr,
    sync::OnceLock,
    thread,
};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND, LPARAM},
    Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW},
    System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    },
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook},
        WindowsAndMessaging::{
            EVENT_SYSTEM_FOREGROUND, EnumChildWindows, GetClassNameW, GetMessageW,
            GetWindowThreadProcessId, MSG, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    },
};

pub struct Foreground {
    pub path: String,
    pub name: String,
    pub window: isize,
}

type Notify = Box<dyn Fn(Foreground) + Send + Sync>;

static NOTIFY: OnceLock<Notify> = OnceLock::new();

/// Windows of the shell itself: taskbar, desktop, Start, search, task switcher.
const SHELL_CLASSES: [&str; 8] = [
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "Progman",
    "WorkerW",
    "MultitaskingViewFrame",
    "XamlExplorerHostIslandWindow",
    "ForegroundStaging",
    "Windows.UI.Core.CoreWindow",
];
const SHELL_PROGRAMS: [&str; 7] = [
    "StartMenuExperienceHost.exe",
    "SearchHost.exe",
    "SearchApp.exe",
    "ShellExperienceHost.exe",
    "LockApp.exe",
    "TextInputHost.exe",
    "ApplicationFrameHost.exe",
];

pub fn start(notify: impl Fn(Foreground) + Send + Sync + 'static) {
    if NOTIFY.set(Box::new(notify)).is_err() {
        return;
    }
    let spawned = thread::Builder::new().name("recent-apps".into()).spawn(|| {
        // SAFETY: the hook and its message loop live on this thread for the life of the process.
        unsafe {
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                ptr::null_mut(),
                Some(on_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            );
            if hook.is_null() {
                tracing::error!("the foreground hook was refused");
                return;
            }
            let mut message: MSG = std::mem::zeroed();
            while GetMessageW(&raw mut message, ptr::null_mut(), 0, 0) > 0 {}
        }
    });
    if let Err(error) = spawned {
        tracing::error!(%error, "could not start the foreground hook");
    }
}

unsafe extern "system" fn on_event(
    _hook: HWINEVENTHOOK,
    _event: u32,
    window: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    if window.is_null() || SHELL_CLASSES.contains(&class_name(window).as_str()) {
        return;
    }
    let Some(path) = program_of(window) else {
        return;
    };
    let file = Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if SHELL_PROGRAMS
        .iter()
        .any(|shell| shell.eq_ignore_ascii_case(&file))
    {
        return;
    }
    let name = description(&path).unwrap_or_else(|| {
        Path::new(&path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default()
    });
    if let Some(notify) = NOTIFY.get() {
        notify(Foreground {
            path,
            name,
            window: window as isize,
        });
    }
}

fn class_name(window: HWND) -> String {
    let mut buffer = [0u16; 128];
    // SAFETY: the buffer length is passed along.
    let length = unsafe { GetClassNameW(window, buffer.as_mut_ptr(), 128) };
    String::from_utf16_lossy(&buffer[..usize::try_from(length).unwrap_or(0)])
}

/// The program of a window. Store apps run inside `ApplicationFrameHost`, so their real
/// process is the one of a child window.
fn program_of(window: HWND) -> Option<String> {
    let path = process_path(process_id(window))?;
    if !path
        .to_ascii_lowercase()
        .ends_with("applicationframehost.exe")
    {
        return Some(path);
    }
    let mut child_process = 0u32;
    let frame_process = process_id(window);
    let mut context = (frame_process, &raw mut child_process);
    // SAFETY: the callback only runs during this call, while `context` is alive.
    unsafe {
        EnumChildWindows(
            window,
            Some(find_child_process),
            (&raw mut context) as LPARAM,
        )
    };
    (child_process != 0)
        .then(|| process_path(child_process))
        .flatten()
}

unsafe extern "system" fn find_child_process(child: HWND, context: LPARAM) -> i32 {
    // SAFETY: `context` points to the tuple built by `program_of`.
    let (frame_process, found) = unsafe { &mut *(context as *mut (u32, *mut u32)) };
    let process = process_id(child);
    if process != *frame_process {
        // SAFETY: `found` points to a live u32 of `program_of`.
        unsafe { **found = process };
        return 0;
    }
    1
}

fn process_id(window: HWND) -> u32 {
    let mut process = 0;
    // SAFETY: writes the process id into `process`.
    unsafe { GetWindowThreadProcessId(window, &raw mut process) };
    process
}

fn process_path(process: u32) -> Option<String> {
    // SAFETY: the handle is closed before returning; the buffer length is passed along.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process);
        if handle.is_null() {
            return None;
        }
        let mut buffer = [0u16; 1024];
        let mut length = 1024u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            buffer.as_mut_ptr(),
            &raw mut length,
        );
        CloseHandle(handle);
        (ok != 0).then(|| {
            OsString::from_wide(&buffer[..length as usize])
                .to_string_lossy()
                .into_owned()
        })
    }
}

/// The "File description" of a program, such as "Visual Studio Code" for `Code.exe`.
fn description(path: &str) -> Option<String> {
    let wide: Vec<u16> = Path::new(path)
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    // SAFETY: the version block outlives every pointer read from it; lengths come from Windows.
    unsafe {
        let size = GetFileVersionInfoSizeW(wide.as_ptr(), ptr::null_mut());
        if size == 0 {
            return None;
        }
        let mut block = vec![0u8; size as usize];
        if GetFileVersionInfoW(wide.as_ptr(), 0, size, block.as_mut_ptr().cast()) == 0 {
            return None;
        }
        let translation = query(&block, r"\VarFileInfo\Translation")?;
        if translation.len() < 4 {
            return None;
        }
        let language = u16::from_le_bytes([translation[0], translation[1]]);
        let code_page = u16::from_le_bytes([translation[2], translation[3]]);
        let text = query(
            &block,
            &format!(r"\StringFileInfo\{language:04x}{code_page:04x}\FileDescription"),
        )?;
        let units: Vec<u16> = text
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes(*pair))
            .collect();
        let description = String::from_utf16_lossy(&units)
            .trim_end_matches('\0')
            .trim()
            .to_owned();
        (!description.is_empty()).then_some(description)
    }
}

/// Reads a value of a version block as bytes.
unsafe fn query<'a>(block: &'a [u8], name: &str) -> Option<&'a [u8]> {
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mut value = ptr::null_mut();
    let mut length = 0u32;
    // SAFETY: Windows points `value` inside `block`; for strings `length` counts UTF-16 units.
    unsafe {
        if VerQueryValueW(
            block.as_ptr().cast(),
            wide.as_ptr(),
            &raw mut value,
            &raw mut length,
        ) == 0
            || value.is_null()
        {
            return None;
        }
        let bytes = if name.starts_with(r"\StringFileInfo") {
            length as usize * 2
        } else {
            length as usize
        };
        Some(std::slice::from_raw_parts(value.cast::<u8>(), bytes))
    }
}

pub fn is_open(window: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindow, IsWindowVisible};
    let window = window as HWND;
    // SAFETY: both calls accept any handle value, including stale ones.
    unsafe { IsWindow(window) != 0 && IsWindowVisible(window) != 0 }
}

/// Brings an app's window back, restoring it if minimised. Must run while Arthex has the focus,
/// since Windows only lets the foreground app hand it over.
pub fn bring_to_front(window: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        IsIconic, SW_RESTORE, SetForegroundWindow, ShowWindow,
    };
    if !is_open(window) {
        return false;
    }
    let window = window as HWND;
    // SAFETY: the handle was checked just above; a stale handle only makes the calls fail.
    unsafe {
        if IsIconic(window) != 0 {
            ShowWindow(window, SW_RESTORE);
        }
        SetForegroundWindow(window) != 0
    }
}
