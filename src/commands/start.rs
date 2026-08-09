use std::fs;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;

    let mut command = Command::new(exe);

    command
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    let child = command.spawn()?;

    let home = std::env::var("HOME")?;
    let dir = format!("{}/.local/share/jaster", home);

    fs::create_dir_all(&dir)?;
    fs::write(format!("{}/jaster.pid", dir), child.id().to_string())?;

    println!("✓ Jaster started.");

    Ok(())
}