use evdev::{Device, EventSummary};
use crate::audio::{
    cache::SoundCache,
    engine::AudioEngine,
    player::AudioPlayer,
};
use std::fs::File;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎹 Jaster daemon started.");

    let engine = AudioEngine::new()?;
    let cache = SoundCache::new()?;

    let file = File::open("/dev/input/event3")?;
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