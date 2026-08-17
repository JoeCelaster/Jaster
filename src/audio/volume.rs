use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    thread,
    time::Duration,
};

/// 100 is the pack's normalized level — see `TARGET_RMS` in `cache.rs`. Speakers
/// need more than that to feel right, so they set the default.
pub const HEADPHONES: u32 = 60;
pub const SPEAKERS: u32 = 150;
pub const DEFAULT: u32 = SPEAKERS;
pub const MAX: u32 = 200;
pub const STEP: u32 = 10;

/// The one line worth telling people who have not touched their volume yet.
pub fn advice() -> String {
    format!("{HEADPHONES} for headphones · {SPEAKERS} for speakers — jaster volume {HEADPHONES}")
}

/// How quickly a running daemon notices `jaster volume`.
const POLL: Duration = Duration::from_millis(400);

fn file() -> Option<PathBuf> {
    Some(crate::utils::paths::data_dir()?.join("volume"))
}

pub fn load() -> u32 {
    let Some(path) = file() else {
        return DEFAULT;
    };

    fs::read_to_string(path)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .map(|value| value.min(MAX))
        .unwrap_or(DEFAULT)
}

pub fn save(percent: u32) -> Result<(), Box<dyn std::error::Error>> {
    let path = file().ok_or("Could not determine a data directory")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, percent.min(MAX).to_string())?;

    Ok(())
}

/// Turn CLI input — `60`, `60%`, `up`, `down`, `mute`, `max` — into a level.
pub fn parse(input: &str, current: u32) -> Result<u32, String> {
    let input = input.trim().to_lowercase();

    let percent = match input.as_str() {
        "up" | "+" => current.saturating_add(STEP),
        "down" | "-" => current.saturating_sub(STEP),
        "mute" | "off" => 0,
        "max" | "full" => MAX,
        value => value
            .trim_end_matches('%')
            .parse::<u32>()
            .map_err(|_| format!("'{input}' is not a volume. Use 0-{MAX}, up, down, or mute."))?,
    };

    Ok(percent.min(MAX))
}

/// The live volume, shared with the audio threads. The daemon cannot be talked
/// to directly, so it re-reads the saved level in the background and `jaster
/// volume` takes effect without a restart.
#[derive(Clone)]
pub struct Level {
    percent: Arc<AtomicU32>,
}

impl Level {
    pub fn watch() -> Self {
        let percent = Arc::new(AtomicU32::new(load()));

        let watched = Arc::clone(&percent);

        thread::spawn(move || {
            loop {
                thread::sleep(POLL);

                let latest = load();

                if watched.swap(latest, Ordering::Relaxed) != latest {
                    println!("🔉 Volume {latest}%");
                }
            }
        });

        Self { percent }
    }

    pub fn percent(&self) -> u32 {
        self.percent.load(Ordering::Relaxed)
    }

    pub fn gain(&self) -> f32 {
        self.percent() as f32 / 100.0
    }
}

/// `▓▓▓▓▓░░░░░ 100%` — a full bar is [`MAX`], so the default sits short of it.
pub fn bar(percent: u32) -> String {
    let width = 10;
    let filled = (percent.min(MAX) as usize * width).div_ceil(MAX as usize);

    format!(
        "{}{} {}%",
        "▓".repeat(filled),
        "░".repeat(width - filled),
        percent
    )
}
