use evdev::{Device, KeyCode};
use std::{
    fs::{self, File},
    process::Command,
};
use colored::*;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🩺 Jaster Doctor\n");

    // ---------------------------
    // Operating System
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
        println!();
        println!("Run:");
        println!("    sudo usermod -aG input $USER");
        println!("    exec su - \"$USER\"");
    }

    if !found && !permission_error {
        println!("  ✗ No keyboards detected.");
    }

    println!();

    // ---------------------------
    // Summary
    // ---------------------------
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if found && !permission_error && audio.success() {
        println!("{}", "                      Jaster is Ready!".bold().cyan());
        println!();

        println!("{}", "Get Started".yellow());
        println!();


        println!("    {}   {}", "jaster start".green(),
            "// Start Jaster".bright_black());

        println!(
            "    {}    {}",
            "jaster oreo".green(),
            "// Switch sound instantly (see jaster sounds)".bright_black()
        );

        println!();


        println!("{}", "Available Commands".yellow());
        println!();


        println!(
            "    {}    {}",
            "jaster doctor".green(),
            "// Diagnose Jaster Health".bright_black()
        );

        println!(
            "    {}    {}",
            "jaster sounds".green(),
            "// List installed sound packs".bright_black()
        );

        println!(
            "    {}    {}",
            "jaster volume".green(),
            "// Show or set the volume  | headphones - 60 | speakers - 150".bright_black()
        );

        println!(
            "    {}     {}",
            "jaster event".green(),
            "// List detected keyboards".bright_black()
        );

        println!(
            "    {}      {}",
            "jaster stop".green(),
            "// Stop the Jaster daemon".bright_black()
        );

        println!(
            "    {}    {}",
            "jaster update".green(),
            "// Update to the latest version".bright_black()
        );

        println!(
            "    {}   {}",
            "jaster version".green(),
            "// Jaster's latest version".bright_black()
        );

        println!();

        println!("{}", "GitHub".yellow());
        println!();

        println!(
            "    {}",
            "https://github.com/JoeCelaster/Jaster".bright_black()
        );

        println!();

        println!(
            "{}",
            "                 Enjoy the typing experience!"
                .bold()
                .cyan()
        );
    } else {
        println!();
        println!("⚠ Jaster is not ready.");
        println!();
        println!("Fix the issues above, then run:");
        println!();
        println!("    jaster doctor");
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}