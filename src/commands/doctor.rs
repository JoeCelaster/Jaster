use evdev::{Device, KeyCode};
use std::{
    fs::{self, File},
    process::Command,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🩺 Jaster Doctor\n");

    // ---------------------------
    // OS
    // ---------------------------
    println!("Operating System");

    if cfg!(target_os = "linux") {
        println!("  ✓ Linux");
    } else {
        println!("  ✗ Unsupported OS");
        return Ok(());
    }

    println!();

    // ---------------------------
    // Audio
    // ---------------------------
    println!("Audio");

    let audio = Command::new("sh")
        .arg("-c")
        .arg("command -v pactl >/dev/null || command -v pw-cli >/dev/null || command -v aplay >/dev/null")
        .status()?;

    if audio.success() {
        println!("  ✓ Audio backend detected");
    } else {
        println!("  ✗ No audio backend found");
    }

    println!();

    // ---------------------------
    // Keyboard
    // ---------------------------
    println!("Keyboard");

    let mut found = false;
    let mut permission_error = false;

    for entry in fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();

        let Some(name) = path.file_name() else {
            continue;
        };

        if !name.to_string_lossy().starts_with("event") {
            continue;
        }

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::PermissionDenied {
                    permission_error = true;
                }
                continue;
            }
        };

        let device = match Device::try_from(file) {
            Ok(device) => device,
            Err(_) => continue,
        };

        if let Some(keys) = device.supported_keys() {
            if keys.contains(KeyCode::KEY_A)
                && keys.contains(KeyCode::KEY_SPACE)
                && keys.contains(KeyCode::KEY_ENTER)
            {
                println!(
                    "  ✓ {} ({})",
                    device.name().unwrap_or("Unknown Keyboard"),
                    path.display()
                );

                found = true;
            }
        }
    }

    if permission_error {
        println!();
        println!("❌ Permission denied.");
        println!("Run:");
        println!("    sudo usermod -aG input $USER");
        println!("Then log out and log back in.");
    }

    if !found && !permission_error {
        println!("  ✗ No keyboards detected.");
    }

    println!();

    // ---------------------------
    // Result
    // ---------------------------
    if found && !permission_error && audio.success() {
        println!("🎉 Everything looks good!");
        println!("Run:");
        println!("    jaster start");
    } else {
        println!("⚠️ Fix the issues above before starting Jaster.");
    }

    Ok(())
}