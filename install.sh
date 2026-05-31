#!/usr/bin/env bash
# Candor AI — One-command install
# Usage: curl -sfL https://raw.githubusercontent.com/iknowkungfubar/candor-ai/main/install.sh | sh
set -euo pipefail

REPO="iknowkungfubar/candor-ai"
BIN_NAME="candor"
INSTALL_DIR="${CANDOR_INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS_RAW="linux" ;;
    Darwin) OS_RAW="macos" ;;
    *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64) ARCH_RAW="x86_64" ;;
    aarch64|arm64) ARCH_RAW="aarch64" ;;
    *)            echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Check for existing binary
if command -v "$BIN_NAME" &>/dev/null; then
    echo "  Candor AI already installed at $(which $BIN_NAME)"
    echo "  To upgrade, uninstall first or use: cargo install candor-ai"
    exit 0
fi

# Check for Rust toolchain as fallback
if command -v cargo &>/dev/null; then
    echo "  Rust toolchain detected — installing via cargo..."
    cargo install candor-ai 2>/dev/null && {
        echo "  ✅ Candor AI installed via cargo"
        echo "  Run 'candor doctor' to verify"
        exit 0
    }
    echo "  Cargo install failed — trying pre-built binary..."
fi

# Pre-built binary (future: GitHub Releases)
echo "  No pre-built binary available for $OS_RAW/$ARCH_RAW yet."
echo ""
echo "  Install Rust and run:"
echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo "    cargo install candor-ai"
echo ""
echo "  Or build from source:"
echo "    git clone https://github.com/iknowkungfubar/candor-ai"
echo "    cd candor-ai && cargo build --release"
exit 1
