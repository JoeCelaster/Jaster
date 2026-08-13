use std::process::Command;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Updating Jaster...");

    let status = Command::new("bash")
        .arg("-c")
        .arg(
            "curl -fsSL https://raw.githubusercontent.com/JoeCelaster/Jaster/main/install.sh | bash",
        )
        .status()?;

    if status.success() {
        println!("✅ Jaster updated!");
    } else {
        eprintln!("❌ Update failed.");
    }

    Ok(())
}