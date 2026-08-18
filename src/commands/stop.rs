use crate::utils::{instance, pid};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    if terminate() {
        println!("✓ Jaster stopped.");
    } else {
        println!("Jaster is not running.");
    }

    Ok(())
}

/// Kill the running daemon, if there is one. Returns whether it was running.
pub fn terminate() -> bool {
    let running = pid::running();

    if let Some(pid) = running {
        kill(pid);
    }

    pid::remove();

    // A daemon the pid file no longer points at is the one people actually
    // notice — it keeps playing, `jaster stop` used to have no way to reach it,
    // and the next `jaster start` put a second daemon alongside it. On Windows
    // it can still be asked to leave by name.
    let unlisted = instance::stop_running();

    running.is_some() || unlisted
}

#[cfg(unix)]
fn kill(pid: u32) {
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGTERM);
    }
}

#[cfg(windows)]
fn kill(pid: u32) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE, TerminateProcess, WaitForSingleObject,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, 0, pid);

        if handle.is_null() {
            return;
        }

        TerminateProcess(handle, 0);

        // Termination is a request, not an event: the process is still there
        // for a moment afterwards, still holding the keyboard hook. `jaster
        // start` calls this and then spawns immediately, so returning early is
        // how two daemons ended up briefly sharing a keyboard.
        WaitForSingleObject(handle, 5_000);

        CloseHandle(handle);
    }
}
