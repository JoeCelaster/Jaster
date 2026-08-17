use clap::Parser;

use jaster::cli::args::{Cli, Commands};
use jaster::commands;

fn main() {
    if let Err(err) = run() {
        eprintln!("✗ {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { sound } => commands::start::run(sound)?,
        Commands::Daemon { sound } => commands::daemon::run(sound)?,
        Commands::Doctor => commands::doctor::run()?,
        Commands::Sounds => commands::sounds::run()?,
        Commands::Volume { value } => commands::volume::run(value)?,
        Commands::Stop => commands::stop::run()?,
        Commands::Event => commands::event::run()?,
        Commands::Update => commands::update::run()?,
        Commands::Version => commands::version::run()?,
        Commands::Switch(args) => commands::switch::run(args)?,
    }

    Ok(())
}
