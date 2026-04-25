use anyhow::Result;
use clap::Subcommand;

/// Supported actions for the layout command.
#[derive(Subcommand)]
pub enum LayoutAction {
    /// Snapshot current workspaces and save as a named layout.
    Save { 
        /// The name to save the layout as.
        name: String 
    },
    /// Launch all apps from a saved layout on their saved workspaces.
    Load {
        /// The name of the layout to load.
        name: String,
        /// Delete the layout after loading (useful for session restore).
        #[arg(short, long)]
        delete: bool,
    },
    /// List all saved layouts.
    List,
    /// Delete a saved layout.
    Delete { 
        /// The name of the layout to delete.
        name: String 
    },
}

/// Executes the requested layout action.
///
/// # Arguments
///
/// * `action` - The specific layout action to perform (save, load, list, or delete).
pub async fn run(action: LayoutAction) -> Result<()> {
    match action {
        LayoutAction::Save { name } => layout::save(&name),
        LayoutAction::Load { name, delete } => layout::load(&name, delete).await,
        LayoutAction::List => layout::list(),
        LayoutAction::Delete { name } => layout::delete(&name),
    }
}
