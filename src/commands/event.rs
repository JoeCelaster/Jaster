use crate::keyboard;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Scanning input devices...\n");

    let sources = keyboard::sources()?;

    if sources.is_empty() {
        println!("✗ No keyboards detected.");
        return Ok(());
    }

    for source in sources {
        println!("✓ {source}");
    }

    Ok(())
}
