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

    // Stop the daemon before updating
    println!("🛑 Stopping Jaster daemon...");

    let stop_status = Command::new("jaster")
        .arg("stop")
        .status()?;

    if !stop_status.success() {
        eprintln!("⚠️ Could not stop the daemon. Continuing with update...");
    }

    println!("⬇️  Updating Jaster...");
    println!("🚀 Launching installer...");

    Command::new("bash")
        .arg("-c")
        .arg(
            "curl -fsSL https://raw.githubusercontent.com/JoeCelaster/Jaster/main/install.sh | bash",
        )
        .spawn()?;

    println!("👋 Jaster will now exit so the update can finish.");

    std::process::exit(0);
}