use std::path::PathBuf;

/// Where Jaster keeps the pid file, the chosen pack, and the volume.
#[cfg(unix)]
pub fn data_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("jaster")
    })
}

#[cfg(windows)]
pub fn data_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(|profile| PathBuf::from(profile).join("AppData").join("Local"))
        })
        .map(|base| base.join("Jaster"))
}

/// Where the installer puts sound packs.
#[cfg(unix)]
fn installed_sounds() -> PathBuf {
    PathBuf::from("/usr/share/jaster/sounds")
}

#[cfg(windows)]
fn installed_sounds() -> PathBuf {
    data_dir()
        .map(|dir| dir.join("sounds"))
        .unwrap_or_else(|| PathBuf::from("sounds"))
}

/// The directory holding sound packs, first hit wins:
///
/// 1. `$JASTER_SOUNDS` — an explicit override, for packaging and tests
/// 2. `sounds/` beside the executable — the installed layout
/// 3. `assets/sounds` relative to the working directory — a dev checkout
/// 4. the platform's install location
///
/// Rule 3 is what lets `cargo test` and `cargo run` work from the repo root.
pub fn sound_root() -> PathBuf {
    if let Some(override_dir) = std::env::var_os("JASTER_SOUNDS") {
        return PathBuf::from(override_dir);
    }

    let beside_exe = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("sounds")))
        .filter(|dir| dir.is_dir());

    if let Some(dir) = beside_exe {
        return dir;
    }

    let checkout = PathBuf::from("assets/sounds");

    if checkout.is_dir() {
        return checkout;
    }

    installed_sounds()
}
