use colored::*;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::audio::theme::{self, SoundPack};
use crate::audio::volume;
use crate::commands::stop;
use crate::utils::pid;
use crate::utils::select::{self, Item, Selection};

pub fn run(sound: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let Some(pack) = choose(sound)? else {
        println!("Cancelled.");
        return Ok(());
    };

    let replaced = spawn(&pack)?;

    if replaced {
        println!("✓ Jaster restarted with {}.", pack.name.cyan());
    } else {
        println!("✓ Jaster started with {}.", pack.name.cyan());
    }
    println!("🔉 {}", volume::advice().bright_black());

    Ok(())
}

/// Detach the daemon on a pack, remember the choice, and record the PID. Any
/// daemon already running is stopped first — a second one would play every key
/// twice, at twice the volume. Returns whether it replaced a running daemon.
pub fn spawn(pack: &SoundPack) -> Result<bool, Box<dyn std::error::Error>> {
    let replaced = stop::terminate();

    theme::save_selection(&pack.id);

    let exe = std::env::current_exe()?;

    let mut command = Command::new(exe);

    command
        .arg("daemon")
        .arg("--sound")
        .arg(&pack.id)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    let child = command.spawn()?;

    pid::save(child.id())?;

    Ok(replaced)
}

/// An explicit `--sound` wins; otherwise ask, falling back to the last used
/// pack when there is no terminal to ask on.
fn choose(sound: Option<String>) -> Result<Option<SoundPack>, Box<dyn std::error::Error>> {
    if let Some(requested) = sound {
        return theme::find(&requested).map(Some).map_err(Into::into);
    }

    let packs = theme::available();

    if packs.len() < 2 {
        return theme::resolve(None).map(Some);
    }

    let items: Vec<Item> = packs
        .iter()
        .map(|pack| Item {
            label: pack.name.clone(),
            hint: format!("jaster {}", theme::shortcut(&pack.id).unwrap_or(&pack.id)),
        })
        .collect();

    let previous = theme::load_selection().unwrap_or_else(|| theme::DEFAULT_PACK.to_string());

    let default = packs
        .iter()
        .position(|pack| pack.id == previous)
        .unwrap_or(0);

    match select::select("Choose your keyboard sound", &items, default)? {
        Selection::Chosen(index) => Ok(packs.into_iter().nth(index)),
        Selection::Cancelled => Ok(None),
        Selection::NotInteractive => theme::resolve(None).map(Some),
    }
}
