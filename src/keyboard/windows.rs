use std::{
    cell::RefCell,
    sync::mpsc::{SyncSender, sync_channel},
    thread,
};

use windows_sys::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, HC_ACTION, KBDLLHOOKSTRUCT,
        LLKHF_EXTENDED, MSG, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    },
};

use crate::keyboard::Key;

type Error = Box<dyn std::error::Error>;

thread_local! {
    /// Where the hook proc drops keys. A low-level hook always runs on the
    /// thread that installed it, so thread-local is both correct and cheaper
    /// than a global.
    static SINK: RefCell<Option<SyncSender<Key>>> = const { RefCell::new(None) };

    /// Which keys are physically held. Windows re-sends WM_KEYDOWN for
    /// auto-repeat with nothing to distinguish it from a fresh press, so
    /// without this a held key machine-guns the sound. 512 slots covers
    /// 0x00..=0xFF twice, plain and extended.
    static HELD: RefCell<[bool; 512]> = const { RefCell::new([false; 512]) };
}

/// Runs for every keystroke on the desktop, inside a 300 ms budget (see
/// `LowLevelHooksTimeout`): exceed it and Windows quietly stops calling us.
/// So this does nothing but bookkeeping and a non-blocking send — no audio, no
/// allocation, no locks the audio side might hold.
unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let info = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };

        let scancode = info.scanCode as u8;
        let extended = info.flags & LLKHF_EXTENDED != 0;
        let slot = scancode as usize + if extended { 256 } else { 0 };

        // WM_SYSKEYDOWN is how Alt combinations and F10 arrive; dropping it
        // would make those keys silent.
        let pressed = wparam as u32 == WM_KEYDOWN || wparam as u32 == WM_SYSKEYDOWN;

        HELD.with(|held| {
            let mut held = held.borrow_mut();

            if !pressed {
                held[slot] = false;
                return;
            }

            if held[slot] {
                return;
            }

            held[slot] = true;

            SINK.with(|sink| {
                if let Some(sender) = sink.borrow().as_ref() {
                    // Never block the hook. A full queue means we are hundreds
                    // of keys behind and the audio is lost anyway.
                    let _ = sender.try_send(Key::new(scancode, extended));
                }
            });
        });
    }

    // Always pass the event on. Returning anything else swallows the keystroke
    // for every application on the desktop.
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

/// The hook is system-wide and deliberately device-agnostic, so there is
/// exactly one source to report.
pub fn sources() -> Result<Vec<String>, Error> {
    Ok(vec!["System-wide keyboard hook".to_string()])
}

/// Blocks forever, calling `on_press` once per key-down.
pub fn listen<F>(on_press: F) -> Result<(), Error>
where
    F: Fn(Key) + Send + Sync + 'static,
{
    let (sender, receiver) = sync_channel::<Key>(256);

    SINK.with(|sink| *sink.borrow_mut() = Some(sender));

    // Audio happens here, off the hook thread and clear of its deadline.
    thread::spawn(move || {
        for key in receiver {
            on_press(key);
        }
    });

    let hook = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(hook_proc),
            GetModuleHandleW(std::ptr::null()),
            0,
        )
    };

    if hook.is_null() {
        return Err("Could not install the keyboard hook. Run `jaster doctor`.".into());
    }

    // A low-level hook is dispatched through the installing thread's message
    // queue, so the proc is never called without a pump. We expect no actual
    // messages — this loop just parks the thread in a place Windows can reach.
    let mut message: MSG = unsafe { std::mem::zeroed() };

    unsafe {
        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        UnhookWindowsHookEx(hook);
    }

    SINK.with(|sink| *sink.borrow_mut() = None);

    Ok(())
}

/// Whether a low-level keyboard hook can actually be installed. Anti-cheat and
/// endpoint-security software is the usual reason this fails.
pub fn hook_is_available() -> bool {
    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(hook_proc),
            GetModuleHandleW(std::ptr::null()),
            0,
        );

        if hook.is_null() {
            return false;
        }

        UnhookWindowsHookEx(hook);
        true
    }
}
