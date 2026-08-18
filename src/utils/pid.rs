use std::{fs, path::PathBuf};

use crate::utils::paths;

pub fn pid_file() -> Option<PathBuf> {
    Some(paths::data_dir()?.join("jaster.pid"))
}

pub fn save(pid: u32) -> std::io::Result<()> {
    let path = pid_file().ok_or_else(|| {
        std::io::Error::other("Could not determine a data directory")
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, pid.to_string())
}

/// The daemon's PID, but only if that process is still alive and still Jaster —
/// a stale PID file can otherwise point at a recycled, unrelated process.
pub fn running() -> Option<u32> {
    let pid = load()?;

    is_jaster(pid).then_some(pid)
}

#[cfg(unix)]
fn is_jaster(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/comm"))
        .is_ok_and(|command| command.trim().contains("jaster"))
}

/// Windows has no `/proc`, so ask the kernel directly: the process must still
/// be running *and* its image must be Jaster, which is the same recycled-PID
/// guard the Linux path gets from `comm`.
#[cfg(windows)]
fn is_jaster(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, WAIT_TIMEOUT};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
        QueryFullProcessImageNameW, WaitForSingleObject,
    };

    unsafe {
        // PROCESS_SYNCHRONIZE is what makes the handle waitable. Without it
        // WaitForSingleObject does not time out, it fails — so every daemon
        // read as already dead, `jaster stop` had nothing to kill, and `jaster
        // start` stacked a second daemon on top of the one still running.
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid,
        );

        if handle.is_null() {
            return false;
        }

        // A process can have exited while a handle keeps its PID reserved.
        // Signalled means exited, so only a timeout proves it is still alive.
        let alive = WaitForSingleObject(handle, 0) == WAIT_TIMEOUT;

        let mut buffer = [0u16; 260];
        let mut length = buffer.len() as u32;

        let named = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            buffer.as_mut_ptr(),
            &mut length,
        ) != 0;

        CloseHandle(handle);

        alive
            && named
            && String::from_utf16_lossy(&buffer[..length as usize])
                .to_ascii_lowercase()
                .contains("jaster")
    }
}

pub fn load() -> Option<u32> {
    fs::read_to_string(pid_file()?).ok()?.trim().parse().ok()
}

pub fn remove() {
    if let Some(path) = pid_file() {
        let _ = fs::remove_file(path);
    }
}
