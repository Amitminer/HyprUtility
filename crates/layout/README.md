# layout

Workspace session manager for Hyprland. Save, load, list, and delete named layouts.
A layout captures which apps are open on which workspaces and relaunches them on demand.

## Usage

```bash
hyprcli layout save <name>          # snapshot current workspaces
hyprcli layout load <name>          # launch everything
hyprcli layout load <name> --delete # load + delete after (session restore)
hyprcli layout list                 # show all saved layouts
hyprcli layout delete <name>        # remove a layout
```

## Class map

Hyprland reports window classes that don't always match the launch command.
For example, ghostty's class is `com.mitchellh.ghostty` but the binary is `ghostty`.

Create `~/.config/hyprutil/class_map.toml` to define overrides:

```toml
[map]
"com.mitchellh.ghostty" = "ghostty"
"dev.zed.Zed" = "zeditor"
"zen" = "zen-browser"
```

Apps where class == command (e.g. `discord`, `spotify`, `telegram-desktop`)
work with zero config — only add entries for mismatches.

## Session save/restore

Save current session and shut down:
```ini
# ~/.config/hypr/hyprland.conf
bind = SUPER SHIFT, S, exec, hyprcli layout save __session__ && uwsm stop
```

Auto-restore on boot (deletes the layout after loading so it doesn't restore on next normal boot):
```ini
exec-once = hyprcli layout load __session__ --delete
```

## Storage

Layouts are stored as TOML files at `~/.local/share/hyprutil/layouts/<name>.toml`.
They're human-readable and editable by hand if needed.

## Known limitations

- Apps using a daemon (ghostty, chromium-based browsers) may open on the wrong
  workspace if already running — a known Hyprland limitation with process-forking apps.
- App internal state (open tabs, files, etc.) is not saved — only which app on which workspace.
- Special workspaces (scratchpads) are excluded from saves.
