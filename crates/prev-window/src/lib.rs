use anyhow::Result;
use hyprland::data::Workspace;
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::prelude::*;

/// Smart workspace switcher with toggle behaviour.
///
/// - If **not** on `target_id` → go there
/// - If **already** on `target_id` → bounce back to previous workspace
///
/// # Arguments
/// * `target_id` - workspace ID to switch to (usually 1–9)
pub fn smart_switch(target_id: i32) -> Result<()> {
    let active = Workspace::get_active()?;

    if active.id == target_id {
        // Already here — bounce back
        Dispatch::call(DispatchType::Workspace(
            WorkspaceIdentifierWithSpecial::Previous,
        ))?;
    } else {
        // Go to target
        Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
            target_id,
        )))?;
    }

    Ok(())
}

/// Unconditionally switches to the previous workspace.
///
/// No toggle logic — always goes back regardless of where you are.
/// Useful as a plain "go back" keybind.
pub fn switch_prev() -> Result<()> {
    Dispatch::call(DispatchType::Workspace(
        WorkspaceIdentifierWithSpecial::Previous,
    ))?;
    Ok(())
}
