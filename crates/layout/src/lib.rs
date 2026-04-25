use anyhow::{Context, Result};
use hyprland::data::Clients;
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::shared::HyprData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

// ── Types ─────────────────────────────────────────────────────────────────────

/// A saved layout — map of workspace id → list of app classes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Layout {
    pub name: String,
    pub workspaces: BTreeMap<i32, Vec<String>>,
}

/// User-defined class → launch command overrides.
///
/// Only needed when the window class doesn't match the launch command.
/// Example: `"com.mitchellh.ghostty" = "ghostty"`
///
/// Falls back to the raw class name if no mapping found.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ClassMap {
    #[serde(default)]
    pub map: BTreeMap<String, String>,
}

// ── Paths ─────────────────────────────────────────────────────────────────────

fn config_dir() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").expect("HOME not set")).join(".config")
        });
    base.join("hyprutil")
}

fn data_dir() -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").expect("HOME not set")).join(".local/share")
        });
    base.join("hyprutil/layouts")
}

fn layout_path(name: &str) -> Result<PathBuf> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join(format!("{name}.toml")))
}

fn class_map_path() -> PathBuf {
    config_dir().join("class_map.toml")
}

// ── Class map ─────────────────────────────────────────────────────────────────

/// Load the class → command map from `~/.config/hyprutil/class_map.toml`.
/// Returns an empty map silently if the file doesn't exist.
fn load_class_map() -> ClassMap {
    let path = class_map_path();

    // DEBUG
    println!("[DEBUG] loading class_map from: {}", path.display());

    let Ok(content) = std::fs::read_to_string(&path) else {
        // DEBUG
        println!("[DEBUG] class_map not found — falling back to raw class names");
        return ClassMap::default();
    };

    // DEBUG
    println!("[DEBUG] class_map: {content}");

    toml::from_str(&content).unwrap_or_default()
}

/// Resolve a Hyprland window class to its launch command.
///
/// Resolution order:
/// 1. Check `class_map.toml` for an explicit override
/// 2. Fall back to the raw class name as-is
///
/// Apps where class == command (e.g. `discord`, `spotify`) work with zero
/// config. Only add entries to `class_map.toml` for mismatches like
/// `"com.mitchellh.ghostty" = "ghostty"`.
fn resolve_class<'a>(class: &'a str, map: &'a ClassMap) -> &'a str {
    map.map.get(class).map(|s| s.as_str()).unwrap_or(class)
}

// ── Notifications ─────────────────────────────────────────────────────────────

/// Send a desktop notification via `notify-send`.
fn notify(title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args([
            "--app-name=HyprUtility",
            "--icon=preferences-desktop",
            "--urgency=normal",
            "--expire-time=3000",
            title,
            body,
        ])
        .spawn();
}

// ── Save ──────────────────────────────────────────────────────────────────────

/// Snapshot all open windows grouped by workspace, write to a TOML layout file.
///
/// - Skips clients with no class (bars, overlays, etc.)
/// - Skips special workspaces (negative IDs)
/// - Deduplicates apps per workspace
///
/// # Arguments
/// * `name` - The name to save the layout as.
pub fn save(name: &str) -> Result<()> {
    let clients = Clients::get()?;
    let mut workspaces: BTreeMap<i32, Vec<String>> = BTreeMap::new();

    for client in clients.iter() {
        // skip bars, overlays, and special workspaces (negative IDs)
        if client.class.is_empty() || client.workspace.id < 0 {
            continue;
        }
        workspaces
            .entry(client.workspace.id)
            .or_default()
            .push(client.class.clone());
    }

    // deduplicate — same app can have multiple windows on one workspace
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
    println!("saved '{name}' — {workspace_count} workspaces, {app_count} apps");

    Ok(())
}

// ── Load ──────────────────────────────────────────────────────────────────────

/// Launch all apps from a saved layout, each on their saved workspace.
///
/// - Skips apps already open on their saved workspace
/// - Resolves class names via `class_map.toml`, falls back to raw class
/// - Uses `[workspace N silent]` dispatch rule via hyprland-rs IPC
/// - Launches apps **sequentially** with a delay between each — concurrent
///   launching causes workspace rule races, especially for daemon-based apps
///   like ghostty where the first launch starts the daemon and subsequent
///   ones connect to it, breaking Hyprland's PID-based workspace tracking
/// - Optionally deletes the layout file after loading (for session restore)
///
/// # Arguments
/// * `name` - The name of the layout to load.
/// * `delete_after` - If true, deletes the layout file after loading.
///
/// # Note
/// Chromium-based browsers may still open on the wrong workspace — they clear
/// their process environment, making PID tracking impossible for Hyprland.
pub async fn load(name: &str, delete_after: bool) -> Result<()> {
    let path = layout_path(name)?;
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("Layout '{name}' not found"))?;
    let layout: Layout =
        toml::from_str(&content).with_context(|| format!("Failed to parse layout '{name}'"))?;

    let class_map = load_class_map();
    let workspace_count = layout.workspaces.len();
    let app_count: usize = layout.workspaces.values().map(|v| v.len()).sum();

    // DEBUG
    println!("[DEBUG] layout: {name}, {workspace_count} workspaces, {app_count} apps");
    println!("[DEBUG] class_map entries: {:?}", class_map.map);

    notify(
        &format!("Loading layout '{name}'"),
        &format!("{workspace_count} workspaces · {app_count} apps"),
    );

    // fetch currently open windows to skip already-running apps
    // DEBUG
    println!("[DEBUG] fetching active clients...");
    let active_clients = Clients::get()?;

    // store as (workspace_id, raw_class) — compare against raw class
    // since that's what Hyprland reports, not the resolved command
    let open_classes: HashSet<(i32, String)> = active_clients
        .iter()
        .map(|c| (c.workspace.id, c.class.clone()))
        .collect();

    // DEBUG
    println!("[DEBUG] currently open: {:?}", open_classes);

    // launch sequentially — concurrent launches cause workspace rule races,
    // especially for ghostty whose daemon opens windows out-of-process,
    // breaking Hyprland's PID-based workspace rule tracking
    for (ws_id, apps) in layout.workspaces {
        let resolved: Vec<(String, String)> = apps
            .iter()
            .map(|class| (class.clone(), resolve_class(class, &class_map).to_string()))
            .collect();

        for (raw_class, cmd) in resolved {
            // skip if already open on this exact workspace
            // compare raw class, not resolved cmd — hyprland reports raw class
            if open_classes.contains(&(ws_id, raw_class.clone())) {
                // DEBUG
                println!("[DEBUG] skipping '{raw_class}' on ws{ws_id} — already open");
                continue;
            }

            // [workspace N silent] — open on workspace N without switching focus
            let exec_arg = format!("[workspace {ws_id} silent] {cmd}");

            // DEBUG
            println!("[DEBUG] dispatching: '{exec_arg}'");

            match Dispatch::call_async(DispatchType::Exec(&exec_arg)).await {
                Ok(_) => println!("  → ws{ws_id}: {cmd}"),
                Err(e) => eprintln!("  ✗ failed '{cmd}' on ws{ws_id}: {e}"),
            }

            // wait for app to register with Hyprland before firing the next one
            // 800ms prevents workspace rule races between sequential launches
            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
        }
    }

    println!("done.");
    notify(&format!("Layout '{name}' loaded"), "");

    if delete_after {
        std::fs::remove_file(&path)?;
        // DEBUG
        println!("[DEBUG] deleted layout file after load");
    }

    Ok(())
}

// ── List ──────────────────────────────────────────────────────────────────────

/// Print all saved layouts with their workspace/app breakdown to stdout.
pub fn list() -> Result<()> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;

    let mut entries: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("no layouts saved.");
        return Ok(());
    }

    for entry in entries {
        let content = std::fs::read_to_string(entry.path())?;
        if let Ok(layout) = toml::from_str::<Layout>(&content) {
            let app_count: usize = layout.workspaces.values().map(|v| v.len()).sum();
            println!(
                "{} — {} workspaces · {} apps",
                layout.name,
                layout.workspaces.len(),
                app_count
            );
            for (ws, apps) in &layout.workspaces {
                println!("  ws{ws}: {}", apps.join(", "));
            }
        }
    }

    Ok(())
}

// ── Delete ────────────────────────────────────────────────────────────────────

/// Delete a saved layout by name.
///
/// # Arguments
/// * `name` - The name of the layout to delete.
pub fn delete(name: &str) -> Result<()> {
    let path = layout_path(name)?;
    std::fs::remove_file(&path).with_context(|| format!("Layout '{name}' not found"))?;

    notify(&format!("Layout '{name}' deleted"), "");
    println!("deleted '{name}'");

    Ok(())
}
