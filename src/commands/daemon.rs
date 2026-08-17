use crate::{
    audio::{
        cache::SoundCache,
        engine::AudioEngine,
        player::AudioPlayer,
        theme,
        volume::Level,
    },
    keyboard::discovery::find_keyboards,
};

use evdev::EventSummary;
use std::{
    sync::Arc,
    thread,
};

pub fn run(sound: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎹 Jaster daemon started.");

    let engine = Arc::new(AudioEngine::new()?);
    println!("✓ Audio engine initialized");

    let pack = theme::resolve(sound.as_deref())?;

    let cache = Arc::new(SoundCache::load(&pack)?);
    println!("✓ Sound pack loaded: {} ({} keys)", pack.name, cache.sounds.len());

    let level = Level::watch();
    println!("✓ Volume {}%", level.percent());

    println!("🔍 Discovering keyboards...");
    let keyboards = find_keyboards()?;

    if keyboards.is_empty() {
        println!("❌ No keyboards found.");
        return Ok(());
    }

    println!("✓ Found {} keyboard(s)", keyboards.len());

    let mut handles = Vec::new();

    for mut keyboard in keyboards {
        let device_name = keyboard
            .device
            .name()
            .unwrap_or("Unknown Keyboard")
            .to_string();

        println!("• {} ({})", device_name, keyboard.path.display());

        let engine = Arc::clone(&engine);
        let cache = Arc::clone(&cache);
        let level = level.clone();

        let handle = thread::spawn(move || {
            println!("🎧 Listening on {}", device_name);

            loop {
                match keyboard.device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if let EventSummary::Key(_, key, value) = event.destructure() {
                                #[cfg(debug_assertions)]
                                if value == 1 {
                                    println!("⌨️  {}: {:?}", device_name, key);
                                }
                                let gain = level.gain();

                                if value == 1 && gain > 0.0 {
                                    let sound = cache.sounds.get(&key).unwrap_or(&cache.generic);

                                    AudioPlayer::play(engine.mixer(), sound.clone(), gain);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "❌ Error reading {}: {}",
                            keyboard.path.display(),
                            err
                        );
                        break;
                    }
                }
            }
        });

        handles.push(handle);
    }

    println!("🚀 Jaster is running.");

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}