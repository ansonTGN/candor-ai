#!/usr/bin/env bash
# Candor AI — Install script
# Usage: curl -fsSL https://raw.githubusercontent.com/iknowkungfubar/candor-ai/main/install.sh | bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${CYAN}${BOLD}"
echo "   ___                _              ___    ___ "
echo "  / __|__ _ _ _  __ _| |___ _ _     /_\ \  |_ _|"
echo " | (__/ _\` | ' \/ _\` | / _ \ '_|   / _ \  | | "
echo "  \___\__,_|_||_\__,_|_\___/_|    /_/ \_\|___|"
echo -e "${NC}"
echo -e "${BOLD}Candor AI — Lawful Good Agentic Operating System${NC}"
echo -e "Version: 1.0.0"
echo ""

# ── Prerequisites ──
check_cmd() {
    if ! command -v "$1" &>/dev/null; then
        echo -e "${RED}Missing: $1. Please install it first.${NC}"
        echo "  $2"
        exit 1
    fi
}

check_cmd "cargo" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_cmd "git"  "sudo apt install git  # or brew install git"

echo -e "${GREEN}✓${NC} Rust toolchain found: $(rustc --version)"

# ── Clone or update ──
INSTALL_DIR="${CANDOR_INSTALL_DIR:-$HOME/.candor-ai}"
REPO="https://github.com/iknowkungfubar/candor-ai.git"

if [ -d "$INSTALL_DIR/.git" ]; then
    echo "Updating existing installation at $INSTALL_DIR..."
    cd "$INSTALL_DIR"
    git pull --ff-only origin main
else
    echo "Cloning to $INSTALL_DIR..."
    git clone "$REPO" "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# ── Build ──
echo ""
echo "Building candor (release mode)..."
cargo build --release 2>&1 | tail -3

BINARY="$INSTALL_DIR/target/release/candor"

if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Build failed. Check Rust toolchain.${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Binary built: $BINARY"

# ── Symlink to PATH ──
LINK_DIR="${CANDOR_BIN_DIR:-$HOME/.local/bin}"
mkdir -p "$LINK_DIR"
ln -sf "$BINARY" "$LINK_DIR/candor"

if [[ ":$PATH:" != *":$LINK_DIR:"* ]]; then
    echo ""
    echo -e "${CYAN}Add to your shell profile (~/.bashrc or ~/.zshrc):${NC}"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo ""
echo -e "${GREEN}${BOLD}Candor AI installed successfully!${NC}"
echo ""
echo "  Quick start:"
echo "    candor --health                  # Check all subsystems"
echo "    candor --task \"build a CLI tool\" # Run agent task"
echo "    candor --port 31337              # Start REST daemon"
echo ""
echo "  LLM setup (pick one):"
echo "    export LM_STUDIO_URL=\"http://localhost:1234/v1\""
echo "    export OPENAI_API_KEY=\"sk-...\""
echo "    export ANTHROPIC_API_KEY=\"sk-ant-...\""
echo ""
echo "  Run tests:"
echo "    cd $INSTALL_DIR && cargo test"
echo ""
echo -e "${GREEN}Done. Run 'candor --health' to verify.${NC}"
