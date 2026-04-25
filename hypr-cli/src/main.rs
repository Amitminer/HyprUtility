use anyhow::Result;
use clap::{Parser, Subcommand};

/// HyprUtility CLI — a collection of Hyprland utilities.
///
/// Each subcommand maps to a utility crate. No logic lives here —
/// this binary just parses args and delegates.
#[derive(Parser)]
#[command(name = "hyprcli", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Smart workspace switcher with toggle behaviour.
    ///
    /// Without --id: always go to previous workspace.
    /// With --id: go to that workspace, or bounce back if already there.
    PrevWindow {
        /// Target workspace ID (1–9). Omit for plain previous.
        #[arg(short, long)]
        id: Option<i32>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::PrevWindow { id } => match id {
            Some(id) => prev_window::smart_switch(id),
            None => prev_window::switch_prev(),
        },
    }
}
