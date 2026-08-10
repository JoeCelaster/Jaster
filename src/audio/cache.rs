use evdev::KeyCode;
use rodio::{source::Buffered, Decoder, Source};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

pub struct SoundCache {
    pub sounds: HashMap<KeyCode, Buffered<Decoder<BufReader<File>>>>,
    pub generic: Buffered<Decoder<BufReader<File>>>,
}

fn sound_root() -> PathBuf {
    if Path::new("assets/sounds").exists() {
        PathBuf::from("assets/sounds")
    } else {
        PathBuf::from("/usr/share/jaster/sounds")
    }
}

impl SoundCache {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut sounds = HashMap::new();

        let theme = sound_root().join("nk-cream");

        let mut load_sound =
            |key: KeyCode, file: &str| -> Result<(), Box<dyn std::error::Error>> {
                let path = theme.join(file);

                if let Ok(file) = File::open(&path) {
                    let decoder = Decoder::try_from(BufReader::new(file))?;
                    sounds.insert(key, decoder.buffered());
                }

                Ok(())
            };

        // Letters
        load_sound(KeyCode::KEY_A, "a.wav")?;
        load_sound(KeyCode::KEY_B, "b.wav")?;
        load_sound(KeyCode::KEY_C, "c.wav")?;
        load_sound(KeyCode::KEY_D, "d.wav")?;
        load_sound(KeyCode::KEY_E, "e.wav")?;
        load_sound(KeyCode::KEY_F, "f.wav")?;
        load_sound(KeyCode::KEY_G, "g.wav")?;
        load_sound(KeyCode::KEY_H, "h.wav")?;
        load_sound(KeyCode::KEY_I, "i.wav")?;
        load_sound(KeyCode::KEY_J, "j.wav")?;
        load_sound(KeyCode::KEY_K, "k.wav")?;
        load_sound(KeyCode::KEY_L, "l.wav")?;
        load_sound(KeyCode::KEY_M, "m.wav")?;
        load_sound(KeyCode::KEY_N, "n.wav")?;
        load_sound(KeyCode::KEY_O, "o.wav")?;
        load_sound(KeyCode::KEY_P, "p.wav")?;
        load_sound(KeyCode::KEY_Q, "q.wav")?;
        load_sound(KeyCode::KEY_R, "r.wav")?;
        load_sound(KeyCode::KEY_S, "s.wav")?;
        load_sound(KeyCode::KEY_T, "t.wav")?;
        load_sound(KeyCode::KEY_U, "u.wav")?;
        load_sound(KeyCode::KEY_V, "v.wav")?;
        load_sound(KeyCode::KEY_W, "w.wav")?;
        load_sound(KeyCode::KEY_X, "x.wav")?;
        load_sound(KeyCode::KEY_Y, "y.wav")?;
        load_sound(KeyCode::KEY_Z, "z.wav")?;

        // Special keys
        load_sound(KeyCode::KEY_SPACE, "space.wav")?;
        load_sound(KeyCode::KEY_ENTER, "enter.wav")?;
        load_sound(KeyCode::KEY_BACKSPACE, "backspace.wav")?;
        load_sound(KeyCode::KEY_LEFTSHIFT, "shift.wav")?;
        load_sound(KeyCode::KEY_RIGHTSHIFT, "shift.wav")?;
        load_sound(KeyCode::KEY_CAPSLOCK, "caps lock.wav")?;
        load_sound(KeyCode::KEY_TAB, "tab.wav")?;
        load_sound(KeyCode::KEY_LEFTBRACE, "[.wav")?;
        load_sound(KeyCode::KEY_RIGHTBRACE, "].wav")?;

        let generic = Decoder::try_from(BufReader::new(
            File::open(theme.join("a.wav"))?,
        ))?
        .buffered();

        Ok(Self { sounds, generic })
    }
}