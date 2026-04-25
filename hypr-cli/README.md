# hypr-cli

Single binary entry point for HyprUtility. Parses args via `clap`, delegates to utility crates.
Zero business logic lives here.

## Binary

```bash
hyprcli --help
hyprcli <command> --help
hyprcli --version
```

## Commands

#### `prev-window`
```bash
hyprcli prev-window           # go to previous workspace
hyprcli prev-window --id <n>  # go to <n>, or bounce back if already there
```

#### `layout`
```bash
hyprcli layout save <name>          # snapshot current windows
hyprcli layout load <name>          # restore windows from snapshot
hyprcli layout load <name> --delete # load then delete (session restore)
hyprcli layout list                 # list saved layouts with breakdown
hyprcli layout delete <name>        # remove a layout
```

## Wiring a new utility

```toml
# hypr-cli/Cargo.toml
[dependencies]
my-tool = { workspace = true }
```

```rust
// src/main.rs
#[derive(Subcommand)]
enum Commands {
    PrevWindow { id: Option<i32> },
    Layout { action: LayoutAction },
    MyTool { /* args */ },
}

// src/commands/mod.rs
pub mod my_tool;
```

## Deps

| Crate         | Why                       |
|---------------|---------------------------|
| `clap`        | arg parsing (derive)      |
| `anyhow`      | error propagation         |
| `prev-window` | workspace toggle          |
| `layout`      | workspace snapshot/restore |
