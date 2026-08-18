//! One daemon per session, and a way to reach it that does not depend on the
//! pid file.
//!
//! The pid file is the normal way `jaster stop` finds the daemon, but it is a
//! file: it can be deleted, or written for a daemon that failed to start, and
//! then a live daemon is holding the keyboard with nothing pointing at it. On
//! Windows that shows up as every keystroke playing twice — a second daemon
//! started because the first was invisible — and as `jaster stop` reporting
//! success while the sound carries on.
//!
//! So the Windows daemon also takes a named mutex, which no second daemon can
//! take, and waits on a named event that `jaster stop` can set from anywhere.
//! Both are kernel objects: they disappear the instant the daemon does, however
//! it dies, so there is no stale state to clean up.

#[cfg(windows)]
mod imp {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr, thread, time::Duration};

    use windows_sys::Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE},
        System::Threading::{
            CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, INFINITE, OpenEventW, SetEvent,
            WaitForSingleObject,
        },
    };

    /// `Local\` scopes these to the logon session, so two people signed into the
    /// same machine each get their own daemon.
    const RUNNING: &str = "Local\\JasterDaemonRunning";
    const STOP: &str = "Local\\JasterDaemonStop";

    /// How long a starting daemon waits for a predecessor to finish dying.
    /// `jaster start` already waits for the process it killed, so this only
    /// covers a daemon that is on its way out for some other reason.
    const HANDOVER: Duration = Duration::from_secs(2);

    /// How long `jaster stop` waits for a daemon it signalled to actually exit.
    const GOODBYE: Duration = Duration::from_secs(5);

    const RETRY: Duration = Duration::from_millis(100);

    /// Proof that this process holds the daemon slot.
    ///
    /// There is nothing to drop: the mutex and the event are held for as long as
    /// the process lives and the kernel reclaims them when it dies. Releasing
    /// them any earlier would let a second daemon in while this one still has
    /// the keyboard, which is the exact thing they exist to prevent.
    pub struct Claim(());

    fn wide(name: &str) -> Vec<u16> {
        OsStr::new(name).encode_wide().chain(Some(0)).collect()
    }

    pub fn claim() -> Result<Claim, Box<dyn std::error::Error>> {
        let name = wide(RUNNING);

        let mut waited = Duration::ZERO;

        loop {
            let slot = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };

            if slot.is_null() {
                return Err("Could not claim the daemon slot".into());
            }

            // Only the mutex's existence matters, never its ownership, and that
            // ends when the last handle to it closes.
            if unsafe { GetLastError() } != ERROR_ALREADY_EXISTS {
                break;
            }

            unsafe { CloseHandle(slot) };

            if waited >= HANDOVER {
                return Err("Jaster is already running. Run `jaster stop` first.".into());
            }

            thread::sleep(RETRY);
            waited += RETRY;
        }

        // Manual reset: once asked to stop, stay stopped.
        let stop = unsafe { CreateEventW(ptr::null(), 1, 0, wide(STOP).as_ptr()) };

        if stop.is_null() {
            return Err("Could not open the daemon's stop signal".into());
        }

        let stop = stop as usize;

        thread::spawn(move || {
            unsafe { WaitForSingleObject(stop as HANDLE, INFINITE) };

            println!("🛑 Jaster daemon stopped.");

            // The keyboard hook and the audio stream are both parked in blocking
            // calls on other threads and there is nothing to unwind: leave now.
            std::process::exit(0);
        });

        Ok(Claim(()))
    }

    /// Ask a daemon we have no pid for to stop, and wait for it to go. Returns
    /// whether there was one.
    pub fn stop_running() -> bool {
        let name = wide(STOP);

        let stop = unsafe { OpenEventW(EVENT_MODIFY_STATE, 0, name.as_ptr()) };

        if stop.is_null() {
            return false;
        }

        unsafe {
            SetEvent(stop);
            CloseHandle(stop);
        }

        // The event exists only while the daemon holds its handle, so failing to
        // open it is how we know the process is really gone. `jaster start`
        // respawns the moment this returns and must not race a daemon that is
        // still on the keyboard.
        for _ in 0..(GOODBYE.as_millis() / RETRY.as_millis()) {
            thread::sleep(RETRY);

            let lingering = unsafe { OpenEventW(EVENT_MODIFY_STATE, 0, name.as_ptr()) };

            if lingering.is_null() {
                break;
            }

            unsafe { CloseHandle(lingering) };
        }

        true
    }
}

#[cfg(unix)]
mod imp {
    /// Linux finds the daemon through `/proc`, which cannot go stale the way a
    /// pid file can, so there is nothing extra to hold.
    pub struct Claim(());

    pub fn claim() -> Result<Claim, Box<dyn std::error::Error>> {
        Ok(Claim(()))
    }

    pub fn stop_running() -> bool {
        false
    }
}

pub use imp::{Claim, claim, stop_running};
