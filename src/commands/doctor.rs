use colored::*;

use crate::audio::theme;
use crate::keyboard;
use crate::utils::pid;

const OS: &str = if cfg!(windows) {
    "Windows"
} else if cfg!(target_os = "macos") {
    "macOS"
} else {
    "Linux"
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🩺 Jaster Doctor\n");

    println!("Operating System");
    println!("  ✓ {OS}");
    println!();

    let audio = check_audio();
    println!();

    let (keyboards, advice) = check_keyboard();
    println!();

    let packs = check_packs();
    println!();

    if let Some(pid) = pid::running() {
        println!("Daemon");
        println!("  ✓ Running (pid {pid})");
        println!();
    }

    for line in &advice {
        println!("{line}");
    }

    if !advice.is_empty() {
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if audio && keyboards && packs {
        ready();
    } else {
        println!();
        println!("⚠ Jaster is not ready.");
        println!();
        println!("Fix the issues above, then run:");
        println!();
        println!("    jaster doctor");
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// Ask the audio stack for a real output device rather than looking for
/// command line tools, which is both portable and a truer test.
fn check_audio() -> bool {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};

    println!("Audio");

    match rodio::cpal::default_host().default_output_device() {
        Some(device) => {
            let name = device.name().unwrap_or_else(|_| "unnamed device".into());
            println!("  ✓ Output: {name}");
            true
        }
        None => {
            println!("  ✗ No audio output device");
            false
        }
    }
}

fn check_packs() -> bool {
    println!("Sound packs");

    let packs = theme::available();

    if packs.is_empty() {
        println!("  ✗ None found in {}", theme::sound_root().display());
        return false;
    }

    println!(
        "  ✓ {} installed in {}",
        packs.len(),
        theme::sound_root().display()
    );

    true
}

/// Returns whether keys can be captured, plus any remediation to print.
#[cfg(target_os = "linux")]
fn check_keyboard() -> (bool, Vec<String>) {
    println!("Keyboard");

    let sources = keyboard::sources().unwrap_or_default();

    for source in &sources {
        println!("  ✓ {source}");
    }

    if !sources.is_empty() {
        return (true, Vec::new());
    }

    // An empty list on Linux is almost always the `input` group, since
    // /dev/input/event* is unreadable rather than absent.
    println!("  ✗ No keyboards detected.");

    (
        false,
        vec![
            "❌ Jaster cannot read your keyboard.".to_string(),
            String::new(),
            "Run:".to_string(),
            "    sudo usermod -aG input $USER".to_string(),
            "    exec su - \"$USER\"".to_string(),
        ],
    )
}

#[cfg(windows)]
fn check_keyboard() -> (bool, Vec<String>) {
    println!("Keyboard");

    let installable = keyboard::hook_is_available();

    if installable {
        println!("  ✓ System-wide keyboard hook available");
    } else {
        println!("  ✗ Could not install the keyboard hook");
    }

    let mut advice = vec![
        "ℹ Windows needs no permission grant for key listening, but anti-cheat".to_string(),
        "  (Vanguard, EasyAntiCheat, BattlEye) and endpoint security software".to_string(),
        "  can block low-level keyboard hooks.".to_string(),
        String::new(),
        "ℹ Keys typed into windows running as administrator are silent unless".to_string(),
        "  Jaster runs elevated too. That is a Windows security boundary.".to_string(),
    ];

    if !installable {
        advice.insert(0, "❌ Jaster cannot capture keys on this machine.".to_string());
        advice.insert(1, String::new());
    }

    (installable, advice)
}

/// macOS answers this with a permission rather than a device list: the tap is
/// always there, it just receives nothing until Input Monitoring is granted.
#[cfg(target_os = "macos")]
fn check_keyboard() -> (bool, Vec<String>) {
    use crate::keyboard::Access;

    println!("Keyboard");

    let mut access = keyboard::access();

    // Nothing has asked yet, so there is no switch in System Settings to point
    // anyone at. Asking is what creates it — and it may put the prompt on
    // screen, which is the shortest path to a working install.
    if access == Access::Unknown {
        keyboard::request();
        access = keyboard::access();
    }

    match access {
        Access::Granted => println!("  ✓ Input Monitoring granted"),
        Access::Denied => println!("  ✗ Input Monitoring denied"),
        Access::Unknown => println!("  ? Input Monitoring not decided yet"),
    }

    let tap = keyboard::tap_can_be_created();

    if tap {
        println!("  ✓ System-wide keyboard event tap available");
    } else {
        println!("  ✗ Could not create the keyboard event tap");
    }

    let ready = access == Access::Granted && tap;

    let mut advice = vec![
        "ℹ macOS gives Input Monitoring to whatever *launched* Jaster, not to".to_string(),
        "  Jaster. Started from a terminal, the switch to turn on carries your".to_string(),
        "  terminal's name — Terminal, iTerm2, Ghostty, VS Code — and there may".to_string(),
        "  be no \"jaster\" entry in the list at all.".to_string(),
        String::new(),
        "ℹ Typing is silent inside password fields and in any app using Secure".to_string(),
        "  Keyboard Entry (Terminal has it in its own menu). macOS shuts every".to_string(),
        "  event tap out of those, by design.".to_string(),
        String::new(),
        "ℹ The grant is tied to the exact binary, so expect to allow it once".to_string(),
        "  more after `jaster update` replaces it.".to_string(),
    ];

    if !ready {
        advice.splice(
            0..0,
            [
                "❌ Jaster cannot capture keys yet.".to_string(),
                String::new(),
                "Open System Settings → Privacy & Security → Input Monitoring and".to_string(),
                "turn on the entry for your terminal. Then quit the terminal".to_string(),
                "completely — ⌘Q, a new window is not enough, since the grant".to_string(),
                "only reaches processes started after it — reopen it and run:".to_string(),
                String::new(),
                "    jaster doctor".to_string(),
                String::new(),
            ],
        );
    }

    (ready, advice)
}

fn ready() {
    println!("{}", "                      Jaster is Ready!".bold().cyan());
    println!();

    println!("{}", "Get Started".yellow());
    println!();
    println!(
        "    {}   {}",
        "jaster start".green(),
        "// Start Jaster".bright_black()
    );
    println!(
        "    {}    {}",
        "jaster oreo".green(),
        "// Switch sound instantly (see jaster sounds)".bright_black()
    );
    println!();

    println!("{}", "Available Commands".yellow());
    println!();

    let commands = [
        ("jaster doctor", "// Diagnose Jaster Health"),
        ("jaster sounds", "// List installed sound packs"),
        (
            "jaster volume",
            "// Show or set the volume  | headphones - 60 | speakers - 150",
        ),
        ("jaster event", "// List detected keyboards"),
        ("jaster stop", "// Stop the Jaster daemon"),
        ("jaster update", "// Update to the latest version"),
        ("jaster version", "// Jaster's latest version"),
    ];

    let width = commands
        .iter()
        .map(|(command, _)| command.len())
        .max()
        .unwrap_or(0);

    for (command, description) in commands {
        let padding = " ".repeat(width - command.len());
        println!(
            "    {}{}    {}",
            command.green(),
            padding,
            description.bright_black()
        );
    }

    println!();
    println!("{}", "GitHub".yellow());
    println!();
    println!(
        "    {}",
        "https://github.com/JoeCelaster/Jaster".bright_black()
    );
    println!();
    println!(
        "{}",
        "                 Enjoy the typing experience!".bold().cyan()
    );
}
