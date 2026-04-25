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
    MyTool { /* args */ },
}

Commands::MyTool { .. } => my_tool::run(),
```

## Deps

| Crate         | Why                  |
|---------------|----------------------|
| `clap`        | arg parsing (derive) |
| `anyhow`      | error propagation    |
| `prev-window` | workspace toggle     |
