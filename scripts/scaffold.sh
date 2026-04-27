#!/bin/bash

# Exit on error
set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <utility-name>"
    echo "Example: $0 my-new-tool"
    exit 1
fi

RAW_NAME=$1
# Convert kebab-case to snake_case for modules/files
SNAKE_NAME=$(echo "$RAW_NAME" | tr '-' '_')
# Convert kebab-case to PascalCase for enum variants
PASCAL_NAME=$(echo "$RAW_NAME" | sed -r 's/(^|-)([a-z])/\U\2/g')

echo "🚀 Scaffolding new utility: $RAW_NAME"

# 1. Create the new crate
cargo new "crates/$RAW_NAME" --lib

# 2. Update crates/$RAW_NAME/Cargo.toml to use workspace
cat > "crates/$RAW_NAME/Cargo.toml" <<EOF
[package]
name = "$RAW_NAME"
version.workspace = true
edition.workspace = true

[dependencies]
hyprland.workspace = true
anyhow.workspace = true
EOF

# 3. Add to root Cargo.toml workspace.dependencies
# Add to [workspace.dependencies]
echo "$RAW_NAME = { path = \"crates/$RAW_NAME\" }" >> Cargo.toml

# 4. Add to hypr-cli/Cargo.toml dependencies
sed -i "/\[dependencies\]/a $RAW_NAME = { workspace = true }" hypr-cli/Cargo.toml

# 5. Create hypr-cli command implementation boilerplate
cat > "hypr-cli/src/commands/$SNAKE_NAME.rs" <<EOF
use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Running $RAW_NAME...");
    Ok(())
}
EOF

# 6. Register in hypr-cli/src/commands/mod.rs
echo "pub mod $SNAKE_NAME;" >> hypr-cli/src/commands/mod.rs

# 7. Update hypr-cli/src/main.rs
# Add to Commands enum
sed -i "/enum Commands {/a \    /// TODO: Description for $RAW_NAME\n    $PASCAL_NAME,\n" hypr-cli/src/main.rs
# Add to match arms in main
sed -i "/match cli.command {/a \        Commands::$PASCAL_NAME => commands::$SNAKE_NAME::run()," hypr-cli/src/main.rs

echo "✅ Done! Scaffolding complete for $RAW_NAME."
echo "   - Created crates/$RAW_NAME"
echo "   - Created hypr-cli/src/commands/$SNAKE_NAME.rs"
echo "   - Updated Cargo.tomls and hypr-cli/src/main.rs"
echo "   - Run 'cargo check' to verify."
