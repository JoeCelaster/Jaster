use colored::*;
use serde::Deserialize;

use crate::audio::theme;
use crate::audio::volume;
use crate::commands::stop;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking for updates...");

    let client = reqwest::blocking::Client::new();

    let release: Release = client
        .get("https://api.github.com/repos/JoeCelaster/Jaster/releases/latest")
        .header("User-Agent", "jaster")
        .send()?
        .error_for_status()?
        .json()?;

    let latest = release.tag_name.trim_start_matches('v');

    if latest == VERSION {
        println!("✅ Jaster is already up to date (v{}).", VERSION);
        println!("🔉 {}", volume::advice().bright_black());
        return Ok(());
    }

    println!("📦 New version available: v{}", latest);

    // Only stop a daemon that is actually running — there is nothing to stop on
    // a fresh install, and shelling out to `jaster stop` just to be told so cost
    // us a process that had to be on PATH.
    let was_running = stop::terminate();

    if was_running {
        println!("🛑 Stopped the Jaster daemon.");
    }

    println!("⬇️  Updating Jaster...");

    install()?;

    println!("✅ Jaster updated to v{}.", latest);

    // We took the daemon down to update it, so we owe them it back. Anything
    // less makes `jaster update` a command that silently kills your key sounds.
    if was_running && restart() {
        return Ok(());
    }

    println!("🔉 {}", volume::advice().bright_black());

    if was_running {
        println!(
            "▶️  Run {} to pick the new build back up.",
            "jaster start".green()
        );
    }

    Ok(())
}

#[cfg(unix)]
const INSTALLER: &str =
    "curl -fsSL https://raw.githubusercontent.com/JoeCelaster/Jaster/main/install.sh | bash";

/// Where the installer puts the binary. We restart through this path rather
/// than `current_exe()` because by then `current_exe()` is the *replaced*
/// inode — the old build we just updated away from.
#[cfg(unix)]
const INSTALLED: &str = "/usr/local/bin/jaster";

#[cfg(unix)]
fn install() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    // Run the installer in the foreground so it keeps this terminal: its `sudo`
    // needs a tty to prompt on, and it no longer has one the moment we exit.
    // JASTER_UPDATE tells it to skip the first-run welcome banner — the person
    // running `jaster update` has already been welcomed.
    let status = Command::new("bash")
        .arg("-c")
        .arg(INSTALLER)
        .env("JASTER_UPDATE", "1")
        .status()?;

    if !status.success() {
        return Err(format!(
            "The installer failed (exit {}). Run it yourself to see why:\n    {}",
            status.code().unwrap_or(-1),
            INSTALLER
        )
        .into());
    }

    Ok(())
}

#[cfg(unix)]
fn installed_path() -> std::path::PathBuf {
    std::path::PathBuf::from(INSTALLED)
}

#[cfg(windows)]
const INSTALLER: &str =
    "irm https://raw.githubusercontent.com/JoeCelaster/Jaster/main/install.ps1 | iex";

/// Windows will not let anyone overwrite a running `.exe`, but it will happily
/// *rename* one — the process keeps executing from the file it already mapped.
/// So we step out of the way, let the installer drop a fresh binary into the
/// name we vacated, and put ourselves back if it fails.
#[cfg(windows)]
fn install() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let exe = std::env::current_exe()?;
    let retired = exe.with_extension("old.exe");

    // Last update's copy: it was still mapped then, so this is the first
    // moment we can get rid of it.
    let _ = std::fs::remove_file(&retired);

    std::fs::rename(&exe, &retired)?;

    let status = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", INSTALLER])
        .env("JASTER_UPDATE", "1")
        // The installer clears any jaster still running, and from where it
        // stands we are one of them. Without this it would stop the update
        // halfway through, leaving the rename above and nothing to undo it.
        .env("JASTER_UPDATE_PID", std::process::id().to_string())
        .status();

    let failed = match status {
        Ok(status) if status.success() => None,
        Ok(status) => Some(format!("exit {}", status.code().unwrap_or(-1))),
        Err(err) => Some(err.to_string()),
    };

    if let Some(reason) = failed {
        // Nothing replaced us, so move back and leave the install as it was.
        let _ = std::fs::rename(&retired, &exe);

        return Err(format!(
            "The installer failed ({reason}). Run it yourself to see why:\n    {INSTALLER}"
        )
        .into());
    }

    Ok(())
}

#[cfg(windows)]
fn installed_path() -> std::path::PathBuf {
    std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("jaster.exe"))
}

/// Bring the daemon back on the pack it was already using, via the new binary.
/// We pass the pack explicitly so `start` never stops to ask — an update is not
/// the moment to hand someone a menu. Returns whether it came back up.
fn restart() -> bool {
    let pack = theme::load_selection().unwrap_or_else(|| theme::DEFAULT_PACK.to_string());

    std::process::Command::new(installed_path())
        .arg("start")
        .arg("--sound")
        .arg(&pack)
        .status()
        .is_ok_and(|status| status.success())
}
