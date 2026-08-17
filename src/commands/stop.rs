use std::process::{Command, Stdio};

use crate::utils::pid;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    if terminate() {
        println!("✓ Jaster stopped.");
    } else {
        println!("Jaster is not running.");
    }

    Ok(())
}

/// Kill the running daemon, if there is one. Returns whether it was running.
pub fn terminate() -> bool {
    let running = pid::running();

    if let Some(pid) = running {
        let _ = Command::new("kill")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    pid::remove();

    running.is_some()
}
