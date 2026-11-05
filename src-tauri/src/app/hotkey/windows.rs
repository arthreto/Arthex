//! Low-level keyboard hook. `RegisterHotKey` cannot register modifier-only combinations such as
//! Shift + Win, nor keep the Start menu closed, so Arthex watches key events itself. The hook only
//! feeds the state machine: keys are never logged or stored.

use std::{
    mem, ptr,
    sync::{Mutex, OnceLock, PoisonError},
    thread,
};

use windows_sys::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
            SendInput,
        },
        WindowsAndMessaging::{
            CallNextHookEx, GetMessageW, HC_ACTION, KBDLLHOOKSTRUCT, MSG, SetWindowsHookExW,
            WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
        },
    },
};

use super::{
    keys::{Key, Mods, classify},
    machine::{Machine, Signal},
};

/// Tags the events Arthex injects, so its own hook ignores them.
const OWN_INPUT: usize = 0x4152_5448;
/// An unassigned virtual key: pressing it counts as "using" Win or Alt, so their release opens nothing.
const MASK_KEY: u16 = 0xE8;
/// Left and right Shift, Ctrl, Alt and Win.
const MODIFIER_KEYS: [u16; 8] = [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0x5B, 0x5C];

type Notify = Box<dyn Fn(Signal) + Send + Sync>;

struct Hook {
    machine: Mutex<Machine>,
    notify: Notify,
}

static HOOK: OnceLock<Hook> = OnceLock::new();

pub fn install(machine: Machine, notify: Notify) {
    if HOOK
        .set(Hook {
            machine: Mutex::new(machine),
            notify,
        })
        .is_err()
    {
        return;
    }
    let spawned = thread::Builder::new().name("hotkey".into()).spawn(run);
    if let Err(error) = spawned {
        tracing::error!(%error, "could not start the keyboard hook");
    }
}

pub fn with_machine(update: impl FnOnce(&mut Machine)) {
    if let Some(hook) = HOOK.get() {
        update(&mut hook.machine.lock().unwrap_or_else(PoisonError::into_inner));
    }
}

fn run() {
    // SAFETY: plain Win32 calls on this thread. The hook stays installed for the life of the
    // process, and the message loop is what lets Windows deliver key events to it.
    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_proc),
            GetModuleHandleW(ptr::null()),
            0,
        );
        if hook.is_null() {
            tracing::error!("the keyboard hook was refused");
            return;
        }
        tracing::info!("keyboard hook installed");
        let mut message: MSG = mem::zeroed();
        while GetMessageW(&raw mut message, ptr::null_mut(), 0, 0) > 0 {}
    }
}

/// Must return quickly: Windows silently removes hooks that are too slow.
unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION.cast_signed()
        && let Some(hook) = HOOK.get()
    {
        // SAFETY: for HC_ACTION, `lparam` points to the event's KBDLLHOOKSTRUCT.
        let event = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
        if event.dwExtraInfo != OWN_INPUT {
            let down = matches!(u32::try_from(wparam), Ok(WM_KEYDOWN | WM_SYSKEYDOWN));
            let vk = u16::try_from(event.vkCode).unwrap_or_default();
            let mut machine = hook.machine.lock().unwrap_or_else(PoisonError::into_inner);
            if down && matches!(classify(vk), Key::Modifier(_)) {
                machine.sync(physical_modifiers());
            }
            let reaction = machine.on_key(vk, down);
            drop(machine);
            if reaction.mask {
                press_mask_key();
            }
            if let Some(signal) = reaction.signal {
                (hook.notify)(signal);
            }
            if reaction.swallow {
                return 1;
            }
        }
    }
    // SAFETY: passes the event on unchanged, as every hook must.
    unsafe { CallNextHookEx(ptr::null_mut(), code, wparam, lparam) }
}

/// The modifiers down before the event the hook is handling: Windows updates its key state
/// only once every hook has let the event through.
fn physical_modifiers() -> Mods {
    let mut mods = Mods::default();
    for vk in MODIFIER_KEYS {
        // SAFETY: reads the key state; no pointer involved.
        let state = unsafe { GetAsyncKeyState(i32::from(vk)) };
        if state < 0
            && let Key::Modifier(modifier) = classify(vk)
        {
            mods.insert(modifier);
        }
    }
    mods
}

fn press_mask_key() {
    let input = |flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: MASK_KEY,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: OWN_INPUT,
            },
        },
    };
    let inputs = [input(0), input(KEYEVENTF_KEYUP)];
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let size = mem::size_of::<INPUT>() as i32;
    // SAFETY: `inputs` is a valid array of two keyboard inputs that outlives the call.
    unsafe { SendInput(2, inputs.as_ptr(), size) };
}
