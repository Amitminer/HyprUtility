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
│   └── src/main.rs
└── crates/               # utility lib crates
    └── prev-window/      # smart workspace toggle
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
| IPC      | [hyprland-rs](https://github.com/hyprland-community/hyprland-rs) `0.3.13` |
| CLI      | clap `4.5` (derive)                                                   |
| Errors   | anyhow `1.0.102`                                                      |

---

## Root Cargo.toml (source of truth)

```toml
[workspace]
members  = ["hypr-cli", "crates/prev-window"]
resolver = "3"

[workspace.package]
edition = "2024"
version = "0.0.1"

[workspace.dependencies]
prev-window        = { path = "crates/prev-window" }
clap               = { version = "4.5", features = ["derive"] }
anyhow             = "1.0.102"
hyprland           = "0.3.13"
tokio              = { version = "1.52.1", features = ["full"] }
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
- No business logic — parse args, delegate to lib crates, that's it
- Owns error display via `anyhow::Result` in `main`

### `prev-window`
- Pure lib crate — no `main.rs`
- Public API:
  - `smart_switch(id: i32)` — go to `id`, or bounce to previous if already there
  - `switch_prev()` — always go to previous workspace, no toggle logic
- Uses `hyprland::data::Workspace::get_active()` to query current workspace
- Uses `hyprland::dispatch::Dispatch::call()` for IPC

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
    MyTool { /* args */ },
}

Commands::MyTool { .. } => my_tool::run(),
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
| _next tool_   | 📋 Backlog | Add as new `crates/` lib crate |
