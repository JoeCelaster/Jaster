use colored::*;
use std::process::{Command, Stdio};

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

    command.arg("daemon").arg("--sound").arg(&pack.id);

    detach(&mut command);

    let child = command.spawn()?;

    pid::save(child.id())?;

    Ok(replaced)
}

/// Put the daemon in its own session so it outlives this terminal.
#[cfg(unix)]
fn detach(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
}

/// A detached process has no console, so its inherited handles would be
/// invalid and the daemon's first `println!` would panic. Send its output to a
/// log file instead — which is also the only way to see why it failed.
#[cfg(windows)]
fn detach(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

    let log = crate::utils::paths::data_dir().and_then(|dir| {
        std::fs::create_dir_all(&dir).ok()?;
        let file = std::fs::File::create(dir.join("daemon.log")).ok()?;
        let clone = file.try_clone().ok()?;
        Some((file, clone))
    });

    let (out, err) = match log {
        Some((file, clone)) => (Stdio::from(file), Stdio::from(clone)),
        None => (Stdio::null(), Stdio::null()),
    };

    command
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
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
