pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Jaster v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}