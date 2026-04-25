use anyhow::{Context, Result};
use hyprland::data::Clients;
use hyprland::shared::HyprData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::process::Command;

// ── Types ────────────────────────────────────────────────────────────────────

/// A saved layout — map of workspace id → list of app classes
#[derive(Debug, Serialize, Deserialize)]
pub struct Layout {
    /// The unique name of the layout.
    pub name: String,
    /// A map where the key is the workspace ID and the value is a list of application classes.
    pub workspaces: BTreeMap<i32, Vec<String>>,
}

// ── Paths ────────────────────────────────────────────────────────────────────

/// Returns the directory where layout files are stored.
///
/// If `XDG_DATA_HOME` is set, it uses `$XDG_DATA_HOME/hyprutil/layouts`.
/// Otherwise, it defaults to `$HOME/.local/share/hyprutil/layouts`.
fn layouts_dir() -> Result<PathBuf> {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME not set");
            PathBuf::from(home).join(".local/share")
        });
    let dir = base.join("hyprutil/layouts");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the full path to a layout file given its name.
///
/// # Arguments
///
/// * `name` - The name of the layout (without extension).
fn layout_path(name: &str) -> Result<PathBuf> {
    Ok(layouts_dir()?.join(format!("{name}.toml")))
}

// ── Notifications ─────────────────────────────────────────────────────────────

/// Sends a desktop notification using `notify-send`.
///
/// # Arguments
///
/// * `title` - The title of the notification.
/// * `body` - The body message of the notification.
fn notify(title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args([
            "--app-name=HyprUtility",
            "--icon=preferences-desktop",
            "--urgency=normal",
            title,
            body,
        ])
        .spawn();
}

// ── Save ──────────────────────────────────────────────────────────────────────

/// Snapshot all open windows grouped by workspace, write to a TOML layout file.
///
/// It queries Hyprland for all active clients, filters those without a class,
/// groups them by workspace ID, and saves the resulting layout to disk.
///
/// # Arguments
///
/// * `name` - The name to save the layout as.
pub fn save(name: &str) -> Result<()> {
    let clients = Clients::get()?;

    let mut workspaces: BTreeMap<i32, Vec<String>> = BTreeMap::new();

    for client in clients.iter() {
        // skip clients with no class (bars, overlays, etc.)
        if client.class.is_empty() {
            continue;
        }
        workspaces
            .entry(client.workspace.id)
            .or_default()
            .push(client.class.clone());
    }

    // deduplicate apps per workspace — same app can have multiple windows
    for apps in workspaces.values_mut() {
        apps.dedup();
    }

    let workspace_count = workspaces.len();
    let app_count: usize = workspaces.values().map(|v| v.len()).sum();

    let layout = Layout {
        name: name.to_string(),
        workspaces,
    };

    let toml = toml::to_string_pretty(&layout)?;
    std::fs::write(layout_path(name)?, toml)?;

    notify(
        &format!("Layout '{name}' saved"),
        &format!("{workspace_count} workspaces · {app_count} apps"),
    );

    Ok(())
}

// ── Load ──────────────────────────────────────────────────────────────────────

/// Launch all apps from a saved layout, each on their saved workspace.
///
/// Uses `[workspace N silent]` dispatch rule so apps open on the right workspace
/// without switching focus. Launches all workspaces concurrently.
///
/// # Arguments
///
/// * `name` - The name of the layout to load.
/// * `delete_after` - If true, the layout file will be deleted after loading.
///
/// # Note
///
/// Browsers (Zen, Firefox, Chromium) may ignore the workspace rule if
/// already running — a known Hyprland limitation.
pub async fn load(name: &str, delete_after: bool) -> Result<()> {
    let path = layout_path(name)?;
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("Layout '{name}' not found"))?;
    let layout: Layout = toml::from_str(&content)?;

    let workspace_count = layout.workspaces.len();
    let app_count: usize = layout.workspaces.values().map(|v| v.len()).sum();

    notify(
        &format!("Loading layout '{name}'"),
        &format!("{workspace_count} workspaces · {app_count} apps"),
    );

    // launch all workspaces concurrently
    let mut handles = vec![];

    for (ws_id, apps) in layout.workspaces {
        let handle = tokio::spawn(async move {
            for app in apps {
                // [workspace N silent] opens the app on workspace N without switching
                let rule = format!("[workspace {ws_id} silent]");
                let _ = Command::new("hyprctl")
                    .args(["dispatch", "exec", &format!("{rule} {app}")])
                    .output()
                    .await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    if delete_after {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

// ── List ──────────────────────────────────────────────────────────────────────

/// Print all saved layouts with their workspace/app counts to stdout.
pub fn list() -> Result<()> {
    let dir = layouts_dir()?;
    let mut found = false;

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }

        let content = std::fs::read_to_string(&path)?;
        if let Ok(layout) = toml::from_str::<Layout>(&content) {
            let app_count: usize = layout.workspaces.values().map(|v| v.len()).sum();
            println!(
                "{} — {} workspaces · {} apps",
                layout.name,
                layout.workspaces.len(),
                app_count
            );
        }
        found = true;
    }

    if !found {
        println!("No layouts saved.");
    }

    Ok(())
}

// ── Delete ────────────────────────────────────────────────────────────────────

/// Delete a saved layout by name.
///
/// # Arguments
///
/// * `name` - The name of the layout to delete.
pub fn delete(name: &str) -> Result<()> {
    let path = layout_path(name)?;
    std::fs::remove_file(&path).with_context(|| format!("Layout '{name}' not found"))?;

    notify(&format!("Layout '{name}' deleted"), "");

    Ok(())
}
