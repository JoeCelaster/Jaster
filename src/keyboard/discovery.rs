use evdev::{Device, KeyCode};
use std::{
    fs::{self, File},
    path::PathBuf,
};

pub struct KeyboardDevice {
    pub device: Device,
    pub path: PathBuf,
}

pub fn find_keyboards() -> Result<Vec<KeyboardDevice>, Box<dyn std::error::Error>> {
    let mut keyboards = Vec::new();

    for entry in fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();

        let Some(name) = path.file_name() else {
            continue;
        };

        if !name.to_string_lossy().starts_with("event") {
            continue;
        }

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => continue,
        };

        let device = match Device::try_from(file) {
            Ok(device) => device,
            Err(_) => continue,
        };

        let Some(keys) = device.supported_keys() else {
            continue;
        };

        let is_keyboard =
            keys.contains(KeyCode::KEY_A)
                && keys.contains(KeyCode::KEY_ENTER)
                && keys.contains(KeyCode::KEY_SPACE);

        if is_keyboard {
            keyboards.push(KeyboardDevice {
                device,
                path,
            });
        }
    }

    Ok(keyboards)
}