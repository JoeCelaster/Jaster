use serde::Deserialize;
use std::process::Command;

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
        return Ok(());
    }

    println!("📦 New version available: v{}", latest);
    println!("⬇️  Updating Jaster...\n");

    let status = Command::new("bash")
        .arg("-c")
        .arg("curl -fsSL https://raw.githubusercontent.com/JoeCelaster/Jaster/main/install.sh | bash")
        .status()?;

    if !status.success() {
        return Err("Update failed.".into());
    }

    println!("\n✅ Successfully updated to v{}!", latest);

    Ok(())
}