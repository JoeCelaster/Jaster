use crate::utils::pid;

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

    running.is_some()
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
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);

        if handle.is_null() {
            return;
        }

        TerminateProcess(handle, 0);
        CloseHandle(handle);
    }
}
