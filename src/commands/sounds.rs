use colored::*;

use crate::audio::theme;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let packs = theme::available();

    if packs.is_empty() {
        println!("No sound packs found in {}", theme::sound_root().display());
        return Ok(());
    }

    let current = theme::resolve(None).ok().map(|pack| pack.id);

    println!("🔊 Sound packs\n");

    let width = packs
        .iter()
        .map(|pack| pack.name.chars().count())
        .max()
        .unwrap_or(0);

    for pack in &packs {
        let marker = if Some(&pack.id) == current.as_ref() {
            "●".cyan()
        } else {
            " ".normal()
        };

        let padding = " ".repeat(width - pack.name.chars().count());
        let shortcut = theme::shortcut(&pack.id).unwrap_or(&pack.id);

        println!(
            "  {} {}{}  {}",
            marker,
            pack.name,
            padding,
            format!("jaster {shortcut}").bright_black()
        );
    }

    println!();
    println!(
        "  {}",
        "Type any of the above to switch instantly.".bright_black()
    );

    Ok(())
}
