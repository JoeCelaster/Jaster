use rodio::{
    source::Buffered,
    Decoder,
    Source,
};
use std::{
    fs::File,
    io::BufReader,
};

pub struct SoundCache {
    pub sounds: std::collections::HashMap<evdev::KeyCode, Buffered<Decoder<BufReader<File>>>>,
    pub generic: Buffered<Decoder<BufReader<File>>>,
}

impl SoundCache {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut sounds = std::collections::HashMap::new();
        
        let mut load_sound = |key: evdev::KeyCode, path: &str| -> Result<(), Box<dyn std::error::Error>> {
            if let Ok(file) = File::open(path) {
                if let Ok(decoder) = Decoder::try_from(BufReader::new(file)) {
                    sounds.insert(key, decoder.buffered());
                }
            }
            Ok(())
        };

        load_sound(evdev::KeyCode::KEY_A, "assets/sounds/nk-cream/a.wav")?;
        load_sound(evdev::KeyCode::KEY_B, "assets/sounds/nk-cream/b.wav")?;
        load_sound(evdev::KeyCode::KEY_C, "assets/sounds/nk-cream/c.wav")?;
        load_sound(evdev::KeyCode::KEY_D, "assets/sounds/nk-cream/d.wav")?;
        load_sound(evdev::KeyCode::KEY_E, "assets/sounds/nk-cream/e.wav")?;
        load_sound(evdev::KeyCode::KEY_F, "assets/sounds/nk-cream/f.wav")?;
        load_sound(evdev::KeyCode::KEY_G, "assets/sounds/nk-cream/g.wav")?;
        load_sound(evdev::KeyCode::KEY_H, "assets/sounds/nk-cream/h.wav")?;
        load_sound(evdev::KeyCode::KEY_I, "assets/sounds/nk-cream/i.wav")?;
        load_sound(evdev::KeyCode::KEY_J, "assets/sounds/nk-cream/j.wav")?;
        load_sound(evdev::KeyCode::KEY_K, "assets/sounds/nk-cream/k.wav")?;
        load_sound(evdev::KeyCode::KEY_L, "assets/sounds/nk-cream/l.wav")?;
        load_sound(evdev::KeyCode::KEY_M, "assets/sounds/nk-cream/m.wav")?;
        load_sound(evdev::KeyCode::KEY_N, "assets/sounds/nk-cream/n.wav")?;
        load_sound(evdev::KeyCode::KEY_O, "assets/sounds/nk-cream/o.wav")?;
        load_sound(evdev::KeyCode::KEY_P, "assets/sounds/nk-cream/p.wav")?;
        load_sound(evdev::KeyCode::KEY_Q, "assets/sounds/nk-cream/q.wav")?;
        load_sound(evdev::KeyCode::KEY_R, "assets/sounds/nk-cream/r.wav")?;
        load_sound(evdev::KeyCode::KEY_S, "assets/sounds/nk-cream/s.wav")?;
        load_sound(evdev::KeyCode::KEY_T, "assets/sounds/nk-cream/t.wav")?;
        load_sound(evdev::KeyCode::KEY_U, "assets/sounds/nk-cream/u.wav")?;
        load_sound(evdev::KeyCode::KEY_V, "assets/sounds/nk-cream/v.wav")?;
        load_sound(evdev::KeyCode::KEY_W, "assets/sounds/nk-cream/w.wav")?;
        load_sound(evdev::KeyCode::KEY_X, "assets/sounds/nk-cream/x.wav")?;
        load_sound(evdev::KeyCode::KEY_Y, "assets/sounds/nk-cream/y.wav")?;
        load_sound(evdev::KeyCode::KEY_Z, "assets/sounds/nk-cream/z.wav")?;

        load_sound(evdev::KeyCode::KEY_SPACE, "assets/sounds/nk-cream/space.wav")?;
        load_sound(evdev::KeyCode::KEY_ENTER, "assets/sounds/nk-cream/enter.wav")?;
        load_sound(evdev::KeyCode::KEY_BACKSPACE, "assets/sounds/nk-cream/backspace.wav")?;
        load_sound(evdev::KeyCode::KEY_LEFTSHIFT, "assets/sounds/nk-cream/shift.wav")?;
        load_sound(evdev::KeyCode::KEY_RIGHTSHIFT, "assets/sounds/nk-cream/shift.wav")?;
        load_sound(evdev::KeyCode::KEY_CAPSLOCK, "assets/sounds/nk-cream/caps lock.wav")?;
        load_sound(evdev::KeyCode::KEY_TAB, "assets/sounds/nk-cream/tab.wav")?;
        load_sound(evdev::KeyCode::KEY_LEFTBRACE, "assets/sounds/nk-cream/[.wav")?;
        load_sound(evdev::KeyCode::KEY_RIGHTBRACE, "assets/sounds/nk-cream/].wav")?;

        let generic = Decoder::try_from(BufReader::new(
            File::open("assets/sounds/nk-cream/a.wav")?,
        ))?
        .buffered();

        Ok(Self { sounds, generic })
    }
}