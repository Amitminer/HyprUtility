# HyprUtility

My personal collection of Hyprland utilities written in Rust using 
[hyprland-rs](https://github.com/hyprland-community/hyprland-rs) for IPC. 
Built for my own Arch Linux + Hyprland setup — just things I needed 
that didn't exist or weren't exactly how I wanted them.

More utilities will be added over time as I find more things to automate or improve
in my workflow.

## Structure

```
HyprUtility/
├── Cargo.toml            # workspace root
├── Cargo.lock
├── hypr-cli/             # binary crate — hyprcli
└── crates/               # utility lib crates
    ├── prev-window/      # smart workspace toggle
    └── layout/           # workspace snapshot and restore
```

## Utilities

| Crate          | Description                                              |
|----------------|----------------------------------------------------------|
| `prev-window`  | Smart workspace toggle — go to target or bounce back     |
| `layout`       | Workspace snapshot and restore — preserve your workflows |

*(Note: I just realized `bind:workspace_back_and_forth = true` exists for quick toggling workspaces, which makes `prev-window` pretty much useless. Lmfao, but anyways.)*

## Building

```bash
cargo build --release
```

Binary ends up at `./target/release/hyprcli`.

Install to `$PATH`:
```bash
cargo install --path hypr-cli
```

## Environment

If running via `uwsm`, ensure these are in your `~/.zshrc`:

```bash
export XDG_RUNTIME_DIR=/run/user/$(id -u)

if [ -z "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
    export HYPRLAND_INSTANCE_SIGNATURE=$(systemctl --user show-environment \
        | grep HYPRLAND_INSTANCE_SIGNATURE \
        | cut -d= -f2)
fi
```

## Development

### Adding a new utility

#### Method 1: Scaffolding Script (Recommended)
Automatically set up a new utility:
```bash
./scripts/scaffold.sh <utility-name>
```

#### Method 2: Manual Setup
1. **Scaffold**: Create a new lib crate: `cargo new crates/<name> --lib`
2. **Workspace**: Add to `members` and `[workspace.dependencies]` in root `Cargo.toml`.
3. **CLI**: Add the dependency to `hypr-cli/Cargo.toml`.
4. **Dispatch**: 
    - Add a subcommand to `Commands` in `hypr-cli/src/main.rs`.
    - Implement the handler in `hypr-cli/src/commands/<name>.rs`.
    - Register the module in `hypr-cli/src/commands/mod.rs`.
