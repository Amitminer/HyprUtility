# HyprUtility — Agent Guide

> Personal Rust workspace of Hyprland utilities for an Arch Linux + Hyprland setup.
> Each utility = a `lib` crate under `crates/`. One binary = `hyprcli` via `hypr-cli/`.

---

## Workspace Layout

```
HyprUtility/
├── Cargo.toml            # workspace root — versions, members, shared metadata
├── Cargo.lock            # always committed
├── AGENT.md
├── README.md
├── .github/
│   └── workflows/
│       └── ci.yml        # fmt + clippy + check on push/PR
├── hypr-cli/             # binary crate — hyprcli
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── main.rs
│       └── commands/      # subcommand implementations
└── crates/               # utility lib crates
    ├── prev-window/      # smart workspace toggle
    │   ├── Cargo.toml
    │   └── src/lib.rs
    └── layout/           # workspace snapshot and restore
        ├── Cargo.toml
        └── src/lib.rs
```

---

## Stack

| Field    | Value                                                                 |
|----------|-----------------------------------------------------------------------|
| Language | Rust 2024                                                             |
| Resolver | `3`                                                                   |
| MSRV     | `1.85.0`                                                              |
| IPC      | [hyprland-rs](https://github.com/hyprland-community/hyprland-rs) (git master) |
| CLI      | clap `4.6` (derive)                                                   |
| Errors   | anyhow `1.0.102`                                                      |

---

## Root Cargo.toml (source of truth)

```toml
[workspace]
members  = ["crates/layout", "crates/prev-window", "hypr-cli"]
resolver = "3"

[workspace.package]
edition = "2024"
version = "0.0.1"

[workspace.dependencies]
anyhow             = "1.0.102"
clap               = { version = "4.6.1", features = ["derive"] }
hyprland           = { git = "https://github.com/hyprland-community/hyprland-rs", branch = "master" }
layout             = { path = "crates/layout" }
prev-window        = { path = "crates/prev-window" }
serde              = { version = "1.0.228", features = ["derive"] }
tokio              = { version = "1.52.1", features = ["full"] }
toml               = "1.1.2"
tracing            = "0.1.44"
tracing-subscriber = "0.3.23"
```

Edition and version flow from `[workspace.package]` — never repeat them in
individual crates, always use `.workspace = true`.

---

## Crate Responsibilities

### `hypr-cli`
- Only crate with a binary — `[[bin]] name = "hyprcli"`
- Uses `clap` derive macros for subcommand parsing
- No business logic — delegates to submodules in `src/commands/` which call lib crates
- Owns error display via `anyhow::Result` in `main`

### `prev-window`
- Pure lib crate — no `main.rs`
- Public API:
  - `smart_switch(id: i32)` — go to `id`, or bounce to previous if already there
  - `switch_prev()` — always go to previous workspace, no toggle logic
- Uses `hyprland::data::Workspace::get_active()` to query current workspace
- Uses `hyprland::dispatch::Dispatch::call()` for IPC

### `layout`
- Pure lib crate
- Public API:
  - `save(name: &str)` — snapshot windows to TOML (excludes special workspaces)
  - `load(name: &str, delete_after: bool)` — restore windows via `exec` rules
  - `list()` — print saved layouts with workspace/app breakdown
  - `delete(name: &str)` — remove layout file
- Features:
  - **Class Map**: Resolves window classes to launch commands via `~/.config/hyprutil/class_map.toml`.
  - **Sequential Launch**: Uses 800ms delay between launches to prevent workspace rule races.
  - **Deduplication**: Skips apps already running on their target workspace.
- Persists to `$XDG_DATA_HOME/hyprutil/layouts/`
- Uses `notify-send` for status updates (3s duration)

---

## Hyprland IPC

Socket lives at `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/`.
Both env vars must be set. uwsm sessions may not propagate them to terminals.

Fix — add to `~/.zshrc`:

```bash
export XDG_RUNTIME_DIR=/run/user/$(id -u)

if [ -z "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
    export HYPRLAND_INSTANCE_SIGNATURE=$(systemctl --user show-environment \
        | grep HYPRLAND_INSTANCE_SIGNATURE \
        | cut -d= -f2)
fi
```

---

## CI

GitHub Actions runs on every push and PR to `master`:
1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo check --workspace`

Note: `cargo build` is intentionally skipped in CI — hyprland-rs links against
Hyprland's IPC socket which doesn't exist on GitHub runners.

---

## Cargo Commands

```bash
cargo build                       # build everything
cargo build -p hypr-cli --release # release binary → target/release/hyprcli
cargo check --workspace           # fast check, no artifacts
cargo clippy --workspace -- -D warnings
cargo fmt --all
cargo test                        # test all crates
cargo test -p prev-window         # test one crate
cargo add <dep> -p <crate>        # add dep to specific crate
cargo install --path hypr-cli     # install hyprcli to $PATH
```

---

## Adding a New Utility

### Recommended: Scaffolding Script
```bash
./scripts/scaffold.sh <utility-name>
```
The script automates crate creation, workspace registration, and CLI boilerplate.

### Alternative: Manual Setup
```bash
cargo new crates/my-tool --lib
```

```toml
# root Cargo.toml
members = [..., "crates/my-tool"]

[workspace.dependencies]
my-tool = { path = "crates/my-tool" }

# hypr-cli/Cargo.toml
[dependencies]
my-tool = { workspace = true }
```

```rust
// hypr-cli/src/main.rs
#[derive(Subcommand)]
enum Commands {
    PrevWindow { id: Option<i32> },
    Layout { action: LayoutAction },
    MyTool { /* args */ },
}

// hypr-cli/src/commands/mod.rs
pub mod my_tool;

// hypr-cli/src/commands/my_tool.rs
pub fn run() -> Result<()> { ... }
```

No README for utility crates — only root and `hypr-cli` get docs.

---

## Code Conventions

- `anyhow::Result` everywhere — no typed errors unless a lib needs a public error API
- No `unwrap()` in lib code — use `?`
- No `println!` in lib code — use `tracing::{info, debug, warn, error}`
- Clippy clean at `-D warnings` before every commit
- Conventional commits: `feat(prev-window): ...`, `fix(hypr-cli): ...`

---

## Git

- Branch: `master`
- Always commit `Cargo.lock`

---

## Roadmap

| Crate         | Status     | Description                    |
|---------------|------------|--------------------------------|
| `prev-window` | ✅ Done    | Smart workspace toggle         |
| `layout`      | ✅ Done    | Workspace snapshot and restore |
| _next tool_   | 📋 Backlog | Add as new `crates/` lib crate |
