#!/usr/bin/env bash
set -e

echo "📷 Building pitu release binary..."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release

echo "⚡ Installing 'pitu' launcher executable into ~/.local/bin..."
./target/release/pitu install-launcher

echo "✨ Installation complete! You can now launch 'pitu' from any terminal tab."
