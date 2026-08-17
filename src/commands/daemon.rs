use crate::{
    audio::{cache::SoundCache, engine::AudioEngine, player::AudioPlayer, theme, volume::Level},
    keyboard,
};

use std::sync::Arc;

pub fn run(sound: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎹 Jaster daemon started.");

    // The stream stays on this thread and only its mixer is shared: on Windows
    // the underlying WASAPI stream is not `Send`, so the engine itself cannot
    // cross into the listener.
    let engine = AudioEngine::new()?;
    let mixer = engine.mixer().clone();
    println!("✓ Audio engine initialized");

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

            AudioPlayer::play(&mixer, sound.clone(), gain);
        }
    })
}
