#!/usr/bin/env bash
# examples/quick_start.sh
# Candor AI — Quick-start examples
#
# These examples demonstrate the main CLI commands after building or
# installing candor-ai.
#
# Prerequisites:
#   cargo install candor-ai
#   # OR from source:
#   cargo build --release && ./target/release/candor --help
#
# For voice features (optional):
#   sudo pacman -S piper-tts whisper.cpp  # Linux
#   brew install piper-tts whisper-cpp    # macOS

set -euo pipefail

CANDOR="${CANDOR:-./target/release/candor}"

echo "╔══════════════════════════════════════════════╗"
echo "║     Candor AI — Quick-Start Examples         ║"
echo "╚══════════════════════════════════════════════╝"
echo "Using: $CANDOR"
echo

# ── Example 1: Health check ──────────────────────────
echo "─── Example 1: Health Check ─────────────────"
echo "Run: $CANDOR health"
echo
$CANDOR health 2>&1 || echo "(candor not installed — build with 'cargo build --release' first)"
echo

# ── Example 2: One-shot task ─────────────────────────
echo "─── Example 2: One-Shot Task ────────────────"
echo "Run: $CANDOR task 'list files in current directory'"
echo
$CANDOR task "list files in current directory" 2>&1 || true
echo

# ── Example 3: Interactive session ───────────────────
echo "─── Example 3: Interactive Session ──────────"
echo "Run: $CANDOR chat"
echo "# (Start a conversation with the agent)"
echo

# ── Example 4: PDA Setup ─────────────────────────────
echo "─── Example 4: PDA Initialization ───────────"
echo "Run: $CANDOR pda init"
echo "# Initializes ~/.candor/ identity and memory store"
echo

# ── Example 5: REST API ──────────────────────────────
echo "─── Example 5: REST API Server ──────────────"
echo "Run: $CANDOR serve --port 31337"
echo "# Starts a REST API daemon at http://localhost:31337"
echo

echo "For full documentation, see:"
echo "  $CANDOR --help"
echo "  https://github.com/TurinTech-Solutions/candor-ai"
