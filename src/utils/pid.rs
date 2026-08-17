use std::{
    fs,
    path::PathBuf,
};

pub fn pid_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("jaster")
        .join("jaster.pid")
}

pub fn save(pid: u32) -> std::io::Result<()> {
    let path = pid_file();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, pid.to_string())
}

/// The daemon's PID, but only if that process is still alive and still Jaster —
/// a stale PID file can otherwise point at a recycled, unrelated process.
pub fn running() -> Option<u32> {
    let pid = load()?;
    let command = fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;

    command.trim().contains("jaster").then_some(pid)
}

pub fn load() -> Option<u32> {
    let path = pid_file();

    fs::read_to_string(path)
        .ok()?
        .trim()
        .parse()
        .ok()
}

pub fn remove() {
    let _ = fs::remove_file(pid_file());
}