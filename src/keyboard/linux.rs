use evdev::{Device, EventSummary, KeyCode};
use std::{
    fs::{self, File},
    path::PathBuf,
    sync::Arc,
    thread,
};

use crate::keyboard::Key;

type Error = Box<dyn std::error::Error>;

struct Keyboard {
    device: Device,
    path: PathBuf,
}

/// evdev keycode to set-1 scancode.
///
/// Codes `1..=88` (`KEY_ESC` through `KEY_F12`) are numerically identical to
/// their set-1 scancodes, which is the whole reason this table is short. Only
/// the extended block needs spelling out. Anything unrecognised passes through
/// and lands on the pack's generic sound.
fn from_evdev(code: u16) -> Key {
    Key(match code {
        1..=88 => code,
        96 => 0xE01C,  // keypad enter
        97 => 0xE01D,  // right ctrl
        98 => 0xE035,  // keypad slash
        99 => 0xE037,  // print screen
        100 => 0xE038, // right alt
        102 => 0xE047, // home
        103 => 0xE048, // up
        104 => 0xE049, // page up
        105 => 0xE04B, // left
        106 => 0xE04D, // right
        107 => 0xE04F, // end
        108 => 0xE050, // down
        109 => 0xE051, // page down
        110 => 0xE052, // insert
        111 => 0xE053, // delete
        119 => 0x45,   // pause
        125 => 0xE05B, // left meta
        126 => 0xE05C, // right meta
        127 => 0xE05D, // menu
        other => other,
    })
}

fn find_keyboards() -> Result<Vec<Keyboard>, Error> {
    let mut keyboards = Vec::new();

    for entry in fs::read_dir("/dev/input")? {
        let path = entry?.path();

        let Some(name) = path.file_name() else {
            continue;
        };

        if !name.to_string_lossy().starts_with("event") {
            continue;
        }

        let Ok(file) = File::open(&path) else {
            continue;
        };

        let Ok(device) = Device::try_from(file) else {
            continue;
        };

        let Some(keys) = device.supported_keys() else {
            continue;
        };

        let is_keyboard = keys.contains(KeyCode::KEY_A)
            && keys.contains(KeyCode::KEY_ENTER)
            && keys.contains(KeyCode::KEY_SPACE);

        if is_keyboard {
            keyboards.push(Keyboard { device, path });
        }
    }

    Ok(keyboards)
}

/// One line per keyboard we would listen on, ready to print.
pub fn sources() -> Result<Vec<String>, Error> {
    Ok(find_keyboards()
        .map(|keyboards| {
            keyboards
                .into_iter()
                .map(|keyboard| {
                    format!(
                        "{} ({})",
                        keyboard.device.name().unwrap_or("Unknown Keyboard"),
                        keyboard.path.display()
                    )
                })
                .collect()
        })
        .unwrap_or_default())
}

/// Blocks forever, calling `on_press` once per key-down. evdev reports
/// auto-repeat as value 2, so filtering to 1 gives us one call per press.
pub fn listen<F>(on_press: F) -> Result<(), Error>
where
    F: Fn(Key) + Send + Sync + 'static,
{
    let keyboards = find_keyboards()?;

    if keyboards.is_empty() {
        return Err("No keyboards found. Run `jaster doctor`.".into());
    }

    let on_press = Arc::new(on_press);
    let mut handles = Vec::new();

    for mut keyboard in keyboards {
        let on_press = Arc::clone(&on_press);

        handles.push(thread::spawn(move || {
            loop {
                match keyboard.device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if let EventSummary::Key(_, key, 1) = event.destructure() {
                                on_press(from_evdev(key.0));
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("❌ Error reading {}: {}", keyboard.path.display(), err);
                        break;
                    }
                }
            }
        }));
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::from_evdev;
    use crate::keyboard::Key;

    #[test]
    fn main_block_is_identity() {
        assert_eq!(from_evdev(30), Key::A);
        assert_eq!(from_evdev(57), Key::SPACE);
        assert_eq!(from_evdev(28), Key::ENTER);
    }

    #[test]
    fn arrows_reach_the_pack_encoding() {
        assert_eq!(from_evdev(103), Key::UP);
        assert_eq!(from_evdev(105), Key::LEFT);
        assert_eq!(from_evdev(106), Key::RIGHT);
        assert_eq!(from_evdev(108), Key::DOWN);
    }

    /// Whatever the device reports, it has to land where the pack loader put
    /// its sounds — otherwise a key silently falls back to the generic clip.
    #[test]
    fn evdev_agrees_with_the_pack_parser() {
        for code in [30u16, 57, 28, 14, 96, 97, 103, 105, 111, 125, 127] {
            let key = from_evdev(code);
            assert_eq!(Key::from_pack_code(key.0 as u32), Some(key));
        }
    }
}
