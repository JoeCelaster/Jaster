use colored::*;

use crate::audio::volume;
use crate::utils::pid;

pub fn run(value: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let current = volume::load();

    let Some(value) = value else {
        println!("🔉 {}", volume::bar(current).cyan());
        println!("  {}", volume::advice().bright_black());

        println!();
        println!(
            "  {}",
            format!(
                "jaster volume 60   jaster volume up   jaster volume mute   (0-{})",
                volume::MAX
            )
            .bright_black()
        );

        return Ok(());
    };

    let percent = volume::parse(&value, current)?;

    volume::save(percent)?;

    println!("🔉 {}", volume::bar(percent).cyan());

    if pid::running().is_none() {
        println!("  {}", "Start Jaster to hear it.".bright_black());
    }

    Ok(())
}
