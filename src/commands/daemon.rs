use crate::{
    audio::{cache::SoundCache, engine::AudioEngine, theme, volume::Level},
    keyboard,
    utils::instance,
};

use std::sync::Arc;

pub fn run(sound: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎹 Jaster daemon started.");

    // First thing, before the keyboard hook exists: two daemons on one keyboard
    // play every keystroke twice, at twice the volume, and only one of them is
    // the one `jaster stop` knows about.
    let _claim = instance::claim()?;

    // The stream itself lives on the engine's own thread — on Windows the
    // WASAPI stream is not `Send` — and follows the default output device, so
    // plugging in headphones moves the sound with it.
    let (engine, device) = AudioEngine::start()?;
    println!("✓ Audio engine initialized");
    println!("🔊 Output: {device}");

    let pack = theme::resolve(sound.as_deref())?;

    let cache = Arc::new(SoundCache::load(&pack)?);
    println!(
        "✓ Sound pack loaded: {} ({} keys)",
        pack.name,
        cache.sounds.len()
    );

    let level = Level::watch();
    println!("✓ Volume {}%", level.percent());

    println!("🔍 Discovering keyboards...");
    let sources = keyboard::sources()?;

    for source in &sources {
        println!("• {source}");
    }

    println!("🚀 Jaster is running.");

    keyboard::listen(move |key| {
        let gain = level.gain();

        if gain > 0.0 {
            let sound = cache.sounds.get(&key).unwrap_or(&cache.generic);

            engine.play(sound.clone(), gain);
        }
    })
}
