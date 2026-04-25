# HyprUtility

My personal collection of Hyprland utilities written in Rust, built for my own
Arch Linux + Hyprland setup. Not a general-purpose tool — just things I needed
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
    └── prev-window/      # smart workspace toggle
```

## Utilities

| Crate          | Description                                              |
|----------------|----------------------------------------------------------|
| `prev-window`  | Smart workspace toggle — go to target or bounce back     |

## Stack

| Field    | Value         |
|----------|---------------|
| Language | Rust 2024     |
| IPC      | [hyprland-rs](https://github.com/hyprland-community/hyprland-rs) |
| CLI      | clap (derive) |
| Errors   | anyhow        |

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

If running via uwsm, add to your `~/.zshrc`:

```bash
export XDG_RUNTIME_DIR=/run/user/$(id -u)

if [ -z "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
    export HYPRLAND_INSTANCE_SIGNATURE=$(systemctl --user show-environment \
        | grep HYPRLAND_INSTANCE_SIGNATURE \
        | cut -d= -f2)
fi
```

## Adding a New Utility

```bash
# 1. scaffold
cargo new crates/my-tool --lib

# 2. add to workspace members in root Cargo.toml
# 3. add to [workspace.dependencies]
#    my-tool = { path = "crates/my-tool" }
# 4. pull into hypr-cli Cargo.toml
#    my-tool = { workspace = true }
# 5. add a subcommand in hypr-cli/src/main.rs
```

## Stack

| Field    | Value    |
|----------|----------|
| Language | Rust     |
| Edition  | 2024     |
| Resolver | 3        |
| MSRV     | 1.85.0   |
