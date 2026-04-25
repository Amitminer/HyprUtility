use anyhow::Result;

/// Executes the workspace switch logic.
///
/// If `id` is provided, it performs a "smart switch" (toggle back if already on that workspace).
/// If `id` is `None`, it unconditionally switches to the previous workspace.
///
/// # Arguments
///
/// * `id` - Optional target workspace ID.
pub fn run(id: Option<i32>) -> Result<()> {
    match id {
        Some(id) => prev_window::smart_switch(id),
        None => prev_window::switch_prev(),
    }
}
