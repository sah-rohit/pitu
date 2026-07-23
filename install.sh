#!/usr/bin/env bash
# pitu One-Click Installer for macOS and Linux

set -e

echo "📷 Building & Installing pitu CLI Image Workbench..."

# Detect OS
OS_TYPE="$(uname -s)"

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

if command -v cargo >/dev/null 2>&1; then
    echo "⚡ Compiling release binary with Cargo..."
    cargo build --release
    rm -f "$INSTALL_DIR/pitu" || true
    cp target/release/pitu "$INSTALL_DIR/pitu" || install -m 755 target/release/pitu "$INSTALL_DIR/pitu"
    chmod +x "$INSTALL_DIR/pitu"
else
    echo "❌ Error: Cargo is required to build pitu. Please install Rust from https://rustup.rs"
    exit 1
fi

# Detect shell config file
SHELL_RC=""
if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
    SHELL_RC="$HOME/.zshrc"
elif [ -f "$HOME/.bashrc" ]; then
    SHELL_RC="$HOME/.bashrc"
elif [ -f "$HOME/.bash_profile" ]; then
    SHELL_RC="$HOME/.bash_profile"
fi

# Add INSTALL_DIR to PATH if missing
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    if [ -n "$SHELL_RC" ]; then
        echo "⚙️ Adding $INSTALL_DIR to $SHELL_RC..."
        echo '' >> "$SHELL_RC"
        echo 'export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"' >> "$SHELL_RC"
    fi
fi

echo "✔ Installed launcher binary to: $INSTALL_DIR/pitu"
echo "✨ Installation complete! Restart your terminal or run 'source $SHELL_RC' and launch 'pitu'."
