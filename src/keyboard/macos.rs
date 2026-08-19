use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    ptr,
    sync::mpsc::{SyncSender, sync_channel},
    thread,
};

use crate::keyboard::{Key, macos_keys};

type Error = Box<dyn std::error::Error>;

// The tap surface is a dozen calls across three frameworks, declared here
// rather than taken from the `core-graphics` crate. That crate is a safe
// wrapper whose event-tap API boxes a Rust closure, and it does not expose the
// raw port we need to re-arm a tap the system switches off; the permission
// calls below have no crate binding at all. `windows.rs` makes the same choice
// by using windows-sys, the raw sibling of `windows`.

type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFRunLoopRef = *mut c_void;
type CFRunLoopMode = *const c_void;
type CFAllocatorRef = *const c_void;
type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;

/// `Boolean` is a byte, not C's `int`.
type Boolean = u8;

type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: Boolean);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> u64;

    /// macOS 10.15+. Whether this process may listen to keyboard events —
    /// the same permission the tap needs, as a plain yes or no.
    fn CGPreflightListenEventAccess() -> Boolean;
    fn CGRequestListenEventAccess() -> Boolean;
}

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    /// macOS 10.15+. Tri-state, which is the only way to tell "denied" from
    /// "never asked" — the difference between telling someone to flip a switch
    /// and telling them one is about to appear.
    fn IOHIDCheckAccess(request_type: u32) -> u32;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;

    fn CFMachPortInvalidate(port: CFMachPortRef);
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(run_loop: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    fn CFRunLoopRun();
    fn CFRelease(cf: *const c_void);

    #[link_name = "kCFRunLoopCommonModes"]
    static COMMON_MODES: CFRunLoopMode;
}

/// Session level rather than `kCGHIDEventTap`: it is the least privileged
/// location that still sees every key in the login session, and the HID one
/// wants root.
const SESSION_EVENT_TAP: u32 = 1;
const HEAD_INSERT_EVENT_TAP: u32 = 0;

/// Listen only. A tap that can rewrite events is a tap that can swallow them,
/// and Jaster has no business touching a keystroke on its way to an app. This
/// is the same invariant `windows.rs` upholds by always calling
/// `CallNextHookEx`, except here the OS enforces it for us.
const TAP_OPTION_LISTEN_ONLY: u32 = 1;

const EVENT_KEY_DOWN: u32 = 10;
const EVENT_FLAGS_CHANGED: u32 = 12;
const EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;
const EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFF_FFFF;

const FIELD_AUTOREPEAT: u32 = 8;
const FIELD_KEYCODE: u32 = 9;

const REQUEST_TYPE_LISTEN_EVENT: u32 = 1;
const ACCESS_TYPE_GRANTED: u32 = 0;
const ACCESS_TYPE_DENIED: u32 = 1;

const EVENT_MASK: u64 = (1 << EVENT_KEY_DOWN) | (1 << EVENT_FLAGS_CHANGED);

/// What macOS thinks about our Input Monitoring request.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Access {
    Granted,
    Denied,
    /// Nothing has asked yet, so there is no entry in System Settings to turn
    /// on — which makes "go and switch it on" advice about a switch that does
    /// not exist. `request()` creates it.
    Unknown,
}

/// Whether this process may listen to keys.
pub fn access() -> Access {
    // The tap is what actually has to work, and this is the reading that
    // matches it, so a yes here settles the question by itself.
    if unsafe { CGPreflightListenEventAccess() } != 0 {
        return Access::Granted;
    }

    // It said no. IOKit's tri-state is only consulted to separate a refusal
    // from a permission nobody has asked for yet — the symptom is identical
    // and the advice is not. A stale "granted" from IOKit still counts as
    // denied, because the call above is the one the tap will agree with.
    match unsafe { IOHIDCheckAccess(REQUEST_TYPE_LISTEN_EVENT) } {
        ACCESS_TYPE_DENIED | ACCESS_TYPE_GRANTED => Access::Denied,
        _ => Access::Unknown,
    }
}

/// Ask macOS for the permission, which is what puts the entry in System
/// Settings in the first place. Returns whether it was granted.
pub fn request() -> bool {
    unsafe { CGRequestListenEventAccess() != 0 }
}

/// What the tap callback needs, reached through `CGEventTapCreate`'s refcon.
///
/// `windows.rs` uses a thread-local because `SetWindowsHookExW` offers the hook
/// proc no user pointer. `CGEventTapCreate` does, so this takes it — which also
/// drops the "must be the installing thread" invariant that goes with it.
struct Tap {
    /// Set once `CGEventTapCreate` has returned and before the run loop starts,
    /// so a callback can re-arm the very tap it is running inside. A `Cell` and
    /// not a lock: it is touched only from the run-loop thread, and taking a
    /// lock in here is one of the few ways to miss the system's deadline.
    port: Cell<CFMachPortRef>,
    sink: SyncSender<Key>,
    /// Which modifiers we believe are physically down. See `macos_keys`.
    modifiers: RefCell<[bool; 128]>,
}

/// Runs for every keystroke in the session. Like the Windows hook proc it does
/// nothing but a table lookup and a non-blocking send — no audio, no
/// allocation, no locks the audio side might hold.
unsafe extern "C" fn handle(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    // `tap_can_be_created` passes a null refcon, but it never adds its port to
    // a run loop, so this is only ever reached with a real one.
    if user_info.is_null() {
        return event;
    }

    let tap = unsafe { &*(user_info as *const Tap) };

    match event_type {
        // The system switches a tap off when a callback is slow, and again
        // when the user's own input outruns it. Nothing else reports it: the
        // daemon just goes deaf while still looking healthy, so re-arming here
        // is not optional. This is macOS's version of the Windows 300 ms
        // `LowLevelHooksTimeout` hazard.
        EVENT_TAP_DISABLED_BY_TIMEOUT | EVENT_TAP_DISABLED_BY_USER_INPUT => {
            let port = tap.port.get();

            if !port.is_null() {
                unsafe { CGEventTapEnable(port, 1) };
            }

            return event;
        }

        EVENT_KEY_DOWN => {
            // Unlike the Windows hook, macOS marks auto-repeat for us — the
            // same favour evdev does with value 2 — so there is no held-key
            // table to maintain and nothing to leak when a key goes up while
            // we are not looking.
            let repeat = unsafe { CGEventGetIntegerValueField(event, FIELD_AUTOREPEAT) };

            if repeat == 0 {
                let code = unsafe { CGEventGetIntegerValueField(event, FIELD_KEYCODE) };

                send(tap, code as u16);
            }
        }

        EVENT_FLAGS_CHANGED => {
            let code = unsafe { CGEventGetIntegerValueField(event, FIELD_KEYCODE) } as u16;
            let flags = unsafe { CGEventGetFlags(event) };

            let press = tap
                .modifiers
                .try_borrow_mut()
                .map(|mut down| macos_keys::transition(code, flags, &mut down))
                .unwrap_or(false);

            if press {
                send(tap, code);
            }
        }

        _ => {}
    }

    // A listen-only tap has its return value ignored, but handing the event
    // straight back is the contract and costs nothing.
    event
}

fn send(tap: &Tap, virtual_key: u16) {
    // Never block the callback. A full queue means we are hundreds of keys
    // behind and the audio is lost anyway.
    let _ = tap.sink.try_send(macos_keys::from_virtual(virtual_key));
}

/// The tap is session-wide and deliberately device-agnostic, so there is
/// exactly one source to report.
pub fn sources() -> Result<Vec<String>, Error> {
    Ok(vec!["System-wide keyboard event tap".to_string()])
}

/// Blocks forever, calling `on_press` once per key-down.
pub fn listen<F>(on_press: F) -> Result<(), Error>
where
    F: Fn(Key) + Send + Sync + 'static,
{
    let (sender, receiver) = sync_channel::<Key>(256);

    let state = Box::into_raw(Box::new(Tap {
        port: Cell::new(ptr::null_mut()),
        sink: sender,
        modifiers: RefCell::new([false; 128]),
    }));

    // Audio happens here, off the run loop and clear of its deadline.
    thread::spawn(move || {
        for key in receiver {
            on_press(key);
        }
    });

    let port = unsafe {
        CGEventTapCreate(
            SESSION_EVENT_TAP,
            HEAD_INSERT_EVENT_TAP,
            TAP_OPTION_LISTEN_ONLY,
            EVENT_MASK,
            handle,
            state.cast(),
        )
    };

    // Since 10.15 this is what a missing Input Monitoring grant looks like:
    // not an error code, not an empty stream — a null port.
    if port.is_null() {
        unsafe { drop(Box::from_raw(state)) };

        return Err("Could not create the keyboard event tap. Run `jaster doctor`.".into());
    }

    // Safe to write: nothing can call back until the source is on a running
    // run loop, which is two statements away.
    unsafe { (*state).port.set(port) };

    unsafe {
        let source = CFMachPortCreateRunLoopSource(ptr::null(), port, 0);

        if source.is_null() {
            CFMachPortInvalidate(port);
            CFRelease(port.cast());
            drop(Box::from_raw(state));

            return Err("Could not attach the keyboard tap to a run loop.".into());
        }

        CFRunLoopAddSource(CFRunLoopGetCurrent(), source, COMMON_MODES);
        CGEventTapEnable(port, 1);

        // A tap is delivered through the run loop it was added to, so the
        // callback is never called without one running. We expect no other
        // sources — this parks the thread somewhere the system can reach it,
        // exactly as the Windows message pump does.
        CFRunLoopRun();

        CGEventTapEnable(port, 0);
        CFMachPortInvalidate(port);
        CFRelease(source.cast());
        CFRelease(port.cast());
        drop(Box::from_raw(state));
    }

    Ok(())
}

/// Whether an event tap can actually be created, which is the same question
/// `hook_is_available` answers on Windows. On macOS a `false` here is almost
/// always the Input Monitoring grant.
pub fn tap_can_be_created() -> bool {
    let port = unsafe {
        CGEventTapCreate(
            SESSION_EVENT_TAP,
            HEAD_INSERT_EVENT_TAP,
            TAP_OPTION_LISTEN_ONLY,
            EVENT_MASK,
            handle,
            // No state to pass: this port is never added to a run loop, so
            // `handle` cannot run. It checks for null anyway.
            ptr::null_mut(),
        )
    };

    if port.is_null() {
        return false;
    }

    unsafe {
        CFMachPortInvalidate(port);
        CFRelease(port.cast());
    }

    true
}
