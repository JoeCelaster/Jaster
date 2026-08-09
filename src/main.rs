use clap::Parser;

use jaster::cli::args::{Cli, Commands};
use jaster::commands;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => commands::start::run()?,
        Commands::Daemon => commands::daemon::run()?,
        Commands::Doctor => commands::doctor::run()?,
        Commands::Stop => commands::stop::run()?,
    }

    Ok(())
}