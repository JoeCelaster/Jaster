use crate::keyboard::discovery::find_keyboards;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Scanning input devices...\n");

    let keyboards = find_keyboards()?;

    if keyboards.is_empty() {
        println!("❌ No keyboards found.");
    } else {
        println!("Found {} keyboard(s).", keyboards.len());

        for path in keyboards {
            println!("{}", path.display());
        }
    }

    Ok(())
}