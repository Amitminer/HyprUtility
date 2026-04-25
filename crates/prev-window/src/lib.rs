use anyhow::Result;
use hyprland::data::Workspace;
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::shared::HyprDataActive;

/// Smart workspace switcher with toggle behaviour.
///
/// - If current workspace is **not** `target_id` → switches to `target_id`.
/// - If current workspace is **already** `target_id` → switches back to the previous workspace.
///
/// # Arguments
///
/// * `target_id` - The workspace ID to switch to (e.g., 0-9).
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
/// This does not include any toggle logic; it always invokes the `workspace previous` dispatch.
pub fn switch_prev() -> Result<()> {
    Dispatch::call(DispatchType::Workspace(
        WorkspaceIdentifierWithSpecial::Previous,
    ))?;
    Ok(())
}
