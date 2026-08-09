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