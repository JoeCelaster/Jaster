use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jaster")]
#[command(version = "0.1.0")]
#[command(about = "Your keyboard. Your sound.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Start,
    Stop,
    Doctor,

    #[command(hide = true)]
    Daemon,

    Event,
    Update
}