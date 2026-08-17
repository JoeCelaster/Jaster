use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jaster")]
#[command(version)]
#[command(about = "Your keyboard. Your sound.")]
#[command(after_help = "Shortcuts:\n  \
    jaster <sound>    Switch sound instantly, e.g. `jaster oreo`, `jaster blue`\n  \
    jaster sounds     Every pack and its shortcut")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Start {
        /// Sound pack to use (skips the picker), e.g. cherrymx or nk-cream
        #[arg(short, long, value_name = "PACK")]
        sound: Option<String>,
    },
    Stop,
    Doctor,

    /// List the installed sound packs
    Sounds,

    /// Show or set the typing volume: a percentage, up, down, or mute
    #[command(alias = "vol")]
    Volume {
        #[arg(value_name = "LEVEL")]
        value: Option<String>,
    },

    #[command(hide = true)]
    Daemon {
        #[arg(short, long, value_name = "PACK")]
        sound: Option<String>,
    },

    Event,
    Update,
    Version,

    /// Switch to a sound pack by shortcut, e.g. `jaster oreo`
    #[command(external_subcommand)]
    Switch(Vec<String>),
}