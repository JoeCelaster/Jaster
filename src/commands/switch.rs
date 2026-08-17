use colored::*;

use crate::audio::theme;
use crate::commands::start;

/// `jaster oreo` and friends: swap the sound the daemon is playing.
pub fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args.into_iter();

    let Some(query) = args.next() else {
        return Err("Name a sound pack, e.g. `jaster oreo`.".into());
    };

    if let Some(extra) = args.next() {
        return Err(format!("Unexpected argument '{extra}'. Try `jaster {query}`.").into());
    }

    let pack = theme::find(&query).map_err(|err| {
        format!("{err}\n  Run `jaster sounds` to list packs, or `jaster --help` for commands.")
    })?;

    let was_running = start::spawn(&pack)?;

    if was_running {
        println!("✓ Switched to {}.", pack.name.cyan());
    } else {
        println!("✓ Jaster started with {}.", pack.name.cyan());
    }

    Ok(())
}
