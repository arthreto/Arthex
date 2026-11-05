use std::{
    cell::{Cell, RefCell},
    ptr,
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, AtomicIsize, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread,
    time::Duration,
};

use tauri::AppHandle;
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM},
    Graphics::Gdi::{
        AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
        CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC,
        RGBQUAD, ReleaseDC, SelectObject,
    },
    System::{
        Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
        LibraryLoader::GetModuleHandleW,
    },
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        Controls::WM_MOUSELEAVE,
        Input::KeyboardAndMouse::{TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent},
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, EVENT_OBJECT_LOCATIONCHANGE,
            EVENT_SYSTEM_FOREGROUND, FindWindowW, GetMessageW, GetWindowThreadProcessId, IDC_ARROW,
            LoadCursorW, MSG, OBJID_WINDOW, PostMessageW, PostQuitMessage, RegisterClassW, SW_HIDE,
            SW_SHOWNOACTIVATE, ShowWindow, ULW_ALPHA, UpdateLayeredWindow, WINEVENT_OUTOFCONTEXT,
            WINEVENT_SKIPOWNPROCESS, WM_APP, WM_CLOSE, WM_DESTROY, WM_DISPLAYCHANGE, WM_DPICHANGED,
            WM_LBUTTONUP, WM_MOUSEMOVE, WM_SETTINGCHANGE, WNDCLASSW, WS_EX_LAYERED,
            WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
        },
    },
};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

use super::{
    locate::Locator,
    paint::icon,
    place::{Placement, placement},
};
use crate::app::overlay;

/// Sent when the locator has a new place for the button.
const WM_PLACE: u32 = WM_APP + 1;
/// Windows reports a burst of changes for a single move: only the last one is read.
const SETTLE: Duration = Duration::from_millis(250);
/// The button of a new app reaches the taskbar a moment after its window, without any event.
const LATE_LOOK: Duration = Duration::from_secs(1);
/// How often to look for the taskbar after it went away, which only an Explorer restart does.
const TASKBAR_RETRY: Duration = Duration::from_secs(1);
const THEME_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";

type SharedPlacement = Arc<Mutex<Option<Placement>>>;

struct Running {
    stop: Arc<AtomicBool>,
    window: Arc<AtomicIsize>,
}

static RUNNING: Mutex<Option<Running>> = Mutex::new(None);

/// What the window procedure and the event hook need; both run on the thread of the window.
struct Context {
    app: AppHandle,
    taskbar: HWND,
    relocate: Sender<()>,
    shared: SharedPlacement,
    current: Cell<Option<Placement>>,
    hovered: Cell<bool>,
}

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

/// Starts or stops the button. Nothing of it runs while it is off.
pub fn set_enabled(app: &AppHandle, enabled: bool) {
    let mut running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
    if !enabled {
        if let Some(running) = running.take() {
            running.stop.store(true, Ordering::SeqCst);
            post(running.window.load(Ordering::SeqCst), WM_CLOSE);
        }
        return;
    }
    if running.is_some() {
        return;
    }
    let started = Running {
        stop: Arc::default(),
        window: Arc::default(),
    };
    let (app, stop, window) = (
        app.clone(),
        Arc::clone(&started.stop),
        Arc::clone(&started.window),
    );
    let spawned = thread::Builder::new()
        .name("taskbar-search".into())
        .spawn(move || run(app, &stop, &window));
    match spawned {
        Ok(_) => *running = Some(started),
        Err(error) => tracing::warn!(%error, "could not start the taskbar search button"),
    }
}

fn post(window: isize, message: u32) {
    if window != 0 {
        // SAFETY: posting to a stale handle only fails.
        unsafe { PostMessageW(window as HWND, message, 0, 0) };
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

fn run(app: AppHandle, stop: &AtomicBool, slot: &Arc<AtomicIsize>) {
    let class = wide("ArthexTaskbarSearch");
    let taskbar_class = wide("Shell_TrayWnd");
    let shared = SharedPlacement::default();
    let (relocate, requests) = mpsc::channel();
    let spawned = {
        let (slot, shared) = (Arc::clone(slot), Arc::clone(&shared));
        thread::Builder::new()
            .name("taskbar-search-locator".into())
            .spawn(move || locate_on_request(&requests, &slot, &shared))
    };
    if let Err(error) = spawned {
        tracing::warn!(%error, "could not start the taskbar search locator");
        return;
    }
    // SAFETY: the class name outlives the call; registering it twice only fails.
    unsafe {
        let description = WNDCLASSW {
            lpfnWndProc: Some(window_procedure),
            hInstance: GetModuleHandleW(ptr::null()),
            hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
            lpszClassName: class.as_ptr(),
            // SAFETY: every other field of this C struct is valid when zero.
            ..std::mem::zeroed()
        };
        RegisterClassW(&raw const description);
    }
    while !stop.load(Ordering::SeqCst) {
        // SAFETY: the class name is a NUL-terminated string that outlives the call.
        let taskbar = unsafe { FindWindowW(taskbar_class.as_ptr(), ptr::null()) };
        if taskbar.is_null() {
            thread::sleep(TASKBAR_RETRY);
            continue;
        }
        CONTEXT.set(Some(Context {
            app: app.clone(),
            taskbar,
            relocate: relocate.clone(),
            shared: Arc::clone(&shared),
            current: Cell::new(None),
            hovered: Cell::new(false),
        }));
        show_until_gone(taskbar, class.as_ptr(), slot, &relocate);
        slot.store(0, Ordering::SeqCst);
        CONTEXT.set(None);
    }
}

/// Lives as long as its window, which goes when the taskbar goes (Explorer restarting).
fn show_until_gone(taskbar: HWND, class: *const u16, slot: &AtomicIsize, relocate: &Sender<()>) {
    // SAFETY: the window, its hooks and its message loop all live on this thread, and the hooks
    // are removed before it returns. The taskbar handle is only an owner: a stale one only fails.
    unsafe {
        let window = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class,
            ptr::null(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            taskbar,
            ptr::null_mut(),
            GetModuleHandleW(ptr::null()),
            ptr::null(),
        );
        if window.is_null() {
            tracing::warn!("could not create the taskbar search button");
            thread::sleep(TASKBAR_RETRY);
            return;
        }
        slot.store(window as isize, Ordering::SeqCst);

        let mut explorer = 0;
        GetWindowThreadProcessId(taskbar, &raw mut explorer);
        let hooks = [
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                ptr::null_mut(),
                Some(on_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            ),
            SetWinEventHook(
                EVENT_OBJECT_LOCATIONCHANGE,
                EVENT_OBJECT_LOCATIONCHANGE,
                ptr::null_mut(),
                Some(on_event),
                explorer,
                0,
                WINEVENT_OUTOFCONTEXT,
            ),
        ];
        let _ = relocate.send(());

        let mut message: MSG = std::mem::zeroed();
        while GetMessageW(&raw mut message, ptr::null_mut(), 0, 0) > 0 {
            DispatchMessageW(&raw const message);
        }
        for hook in hooks {
            UnhookWinEvent(hook);
        }
    }
}

/// UI Automation asks Explorer, which may wait on the windows of the caller: it runs here, away
/// from the thread of our window.
fn locate_on_request(requests: &Receiver<()>, window: &AtomicIsize, shared: &SharedPlacement) {
    // SAFETY: paired with `CoUninitialize` below, once the COM object is dropped.
    let initialised =
        unsafe { CoInitializeEx(ptr::null(), COINIT_MULTITHREADED.cast_unsigned()) } >= 0;
    if let Some(locator) = Locator::new() {
        let publish = || {
            let taskbar_class = wide("Shell_TrayWnd");
            // SAFETY: the class name is a NUL-terminated string that outlives the call.
            let taskbar = unsafe { FindWindowW(taskbar_class.as_ptr(), ptr::null()) };
            let found = locator
                .read(taskbar as isize)
                .and_then(|(bounds, items)| placement(bounds, &items));
            tracing::debug!(?found, "taskbar icon placement");
            *shared.lock().unwrap_or_else(PoisonError::into_inner) = found;
            post(window.load(Ordering::SeqCst), WM_PLACE);
        };
        while requests.recv().is_ok() {
            loop {
                while requests.recv_timeout(SETTLE).is_ok() {}
                publish();
                match requests.recv_timeout(LATE_LOOK) {
                    Ok(()) => {}
                    Err(RecvTimeoutError::Timeout) => {
                        publish();
                        break;
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        }
    }
    if initialised {
        // SAFETY: balances the successful initialisation above, on the same thread.
        unsafe { CoUninitialize() };
    }
}

unsafe extern "system" fn on_event(
    _hook: HWINEVENTHOOK,
    event: u32,
    window: HWND,
    object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    CONTEXT.with_borrow(|context| {
        if let Some(context) = context
            && (event == EVENT_SYSTEM_FOREGROUND
                || (window == context.taskbar && object == OBJID_WINDOW))
        {
            let _ = context.relocate.send(());
        }
    });
}

unsafe extern "system" fn window_procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    CONTEXT.with_borrow(|context| {
        if let Some(context) = context {
            handle(context, window, message);
        }
    });
    if message == WM_DESTROY {
        // SAFETY: ends the message loop of this thread.
        unsafe { PostQuitMessage(0) };
    }
    // SAFETY: forwards the arguments Windows gave us.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

fn handle(context: &Context, window: HWND, message: u32) {
    match message {
        WM_PLACE => {
            let found = *context
                .shared
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if found != context.current.get() {
                context.current.set(found);
                repaint(context, window);
            }
            // SAFETY: our own window, shown without taking the focus.
            unsafe {
                ShowWindow(
                    window,
                    if found.is_some() {
                        SW_SHOWNOACTIVATE
                    } else {
                        SW_HIDE
                    },
                )
            };
        }
        WM_SETTINGCHANGE | WM_DISPLAYCHANGE | WM_DPICHANGED => {
            // Forgets the place so the next one repaints, for a new theme or scale.
            context.current.set(None);
            let _ = context.relocate.send(());
        }
        WM_MOUSEMOVE if !context.hovered.get() => {
            context.hovered.set(true);
            let mut leave = TRACKMOUSEEVENT {
                cbSize: u32::try_from(size_of::<TRACKMOUSEEVENT>()).unwrap_or_default(),
                dwFlags: TME_LEAVE,
                hwndTrack: window,
                dwHoverTime: 0,
            };
            // SAFETY: the struct outlives the call and names our own window.
            unsafe { TrackMouseEvent(&raw mut leave) };
            repaint(context, window);
        }
        WM_MOUSELEAVE => {
            context.hovered.set(false);
            repaint(context, window);
        }
        WM_LBUTTONUP => {
            let handle = context.app.clone();
            let _ = context
                .app
                .run_on_main_thread(move || overlay::toggle(&handle));
        }
        _ => {}
    }
}

fn repaint(context: &Context, window: HWND) {
    if let Some(place) = context.current.get() {
        let pixels = icon(place.size, taskbar_is_dark(), context.hovered.get());
        show_pixels(window, place, &pixels);
    }
}

/// The taskbar follows the theme of Windows, not the one chosen in Arthex.
fn taskbar_is_dark() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(THEME_KEY)
        .and_then(|key| key.get_value::<u32, _>("SystemUsesLightTheme"))
        .map_or(true, |light| light == 0)
}

fn show_pixels(window: HWND, place: Placement, pixels: &[u8]) {
    let side = i32::from(place.size);
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).unwrap_or_default(),
            biWidth: side,
            // Negative: the rows go from the top down, as `icon` writes them.
            biHeight: -side,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD::default()],
    };
    let position = POINT {
        x: place.x,
        y: place.y,
    };
    let extent = SIZE { cx: side, cy: side };
    let source = POINT { x: 0, y: 0 };
    let blend = BLENDFUNCTION {
        BlendOp: u8::try_from(AC_SRC_OVER).unwrap_or_default(),
        BlendFlags: 0,
        SourceConstantAlpha: u8::MAX,
        AlphaFormat: u8::try_from(AC_SRC_ALPHA).unwrap_or_default(),
    };
    // SAFETY: GDI objects created and released here; the bitmap holds `side * side` pixels of 4
    // bytes, which is what `icon` returns, and every pointer outlives its call.
    unsafe {
        let screen = GetDC(ptr::null_mut());
        let memory = CreateCompatibleDC(screen);
        let mut bits = ptr::null_mut();
        let bitmap = CreateDIBSection(
            memory,
            &raw const info,
            DIB_RGB_COLORS,
            &raw mut bits,
            ptr::null_mut(),
            0,
        );
        if !bitmap.is_null() && !bits.is_null() {
            ptr::copy_nonoverlapping(pixels.as_ptr(), bits.cast::<u8>(), pixels.len());
            let previous = SelectObject(memory, bitmap);
            UpdateLayeredWindow(
                window,
                screen,
                &raw const position,
                &raw const extent,
                memory,
                &raw const source,
                0,
                &raw const blend,
                ULW_ALPHA,
            );
            SelectObject(memory, previous);
            DeleteObject(bitmap);
        }
        DeleteDC(memory);
        ReleaseDC(ptr::null_mut(), screen);
    }
}
