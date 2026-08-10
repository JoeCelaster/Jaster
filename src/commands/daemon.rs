use crate::{
    audio::{
        cache::SoundCache,
        engine::AudioEngine,
        player::AudioPlayer,
    },
    keyboard::discovery::find_keyboards,
};

use evdev::{Device, EventSummary};
use std::fs::File;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎹 Jaster daemon started.");

    let engine = AudioEngine::new()?;
    let cache = SoundCache::new()?;

    let keyboards = find_keyboards()?;

    if keyboards.is_empty() {
        println!("❌ No keyboards detected.");
        return Ok(());
    }

    println!("Found {} keyboard(s).", keyboards.len());

    let path = &keyboards[0];

    println!("Using {}", path.display());

    let file = File::open(path)?;
    let mut device = Device::try_from(file)?;

    println!("Listening for keyboard events...");

    loop {
        for event in device.fetch_events()? {
            if let EventSummary::Key(_, key, value) = event.destructure() {
                if value == 1 {
                    if let Some(sound) = cache.sounds.get(&key) {
                        AudioPlayer::play(engine.mixer(), sound.clone());
                    } else {
                        AudioPlayer::play(engine.mixer(), cache.generic.clone());
                    }
                }
            }
        }
    }
}