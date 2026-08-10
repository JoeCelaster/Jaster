use crate::keyboard::discovery::find_keyboards;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Scanning input devices...\n");

    let keyboards = find_keyboards()?;

for keyboard in keyboards {
    println!(
        "✓ {} - {}",
        keyboard.path.display(),
        keyboard.device.name().unwrap_or("Unknown Keyboard")
    );
}

    Ok(())
}