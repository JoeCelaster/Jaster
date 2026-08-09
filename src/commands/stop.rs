use std::{fs, process::Command};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let pid_file = format!("{}/.local/share/jaster/jaster.pid", home);

    let pid = match fs::read_to_string(&pid_file) {
        Ok(pid) => pid,
        Err(_) => {
            println!("Jaster is not running.");
            return Ok(());
        }
    };

    Command::new("kill")
        .arg(pid.trim())
        .status()?;

    let _ = fs::remove_file(pid_file);

    println!("✓ Jaster stopped.");

    Ok(())
}