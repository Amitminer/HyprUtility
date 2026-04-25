use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::layout::LayoutAction;

mod commands;

/// Command line utility for Hyprland automation.
#[derive(Parser)]
#[command(name = "hyprcli", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for hyprcli.
#[derive(Subcommand)]
enum Commands {
    /// Smart workspace switcher with toggle behaviour.
    PrevWindow {
        /// Target workspace ID (1–9). Omit for plain previous.
        #[arg(short, long)]
        id: Option<i32>,
    },
    /// Manage workspace layouts (save, load, list, delete).
    Layout {
        #[command(subcommand)]
        action: LayoutAction,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::PrevWindow { id } => commands::prev_window::run(id),
        Commands::Layout { action } => commands::layout::run(action).await,
    }
}
