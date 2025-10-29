#!/usr/bin/env bash
# TUI Runner - Browse and search all your Claude conversations
# Loads from ~/.claude/projects by default, showing 1000+ sessions

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/claude-session-tui && pwd)"

# Ensure we're in a TTY
if [ ! -t 0 ] || [ ! -t 1 ]; then
    echo "Error: TUI requires an interactive terminal (TTY)"
    echo "Please run this command directly in a terminal, not through pipes or redirection"
    exit 1
fi

# Set terminal environment if not set
export TERM="${TERM:-xterm-256color}"

# Build the TUI if needed
if [ ! -f "$SCRIPT_DIR/target/release/claude-session-tui" ]; then
    echo "Building TUI component (first run, this may take a minute)..."
    cd "$SCRIPT_DIR"
    cargo build --release --features tui --quiet
fi

# Run the TUI with default ~/.claude/projects
# Usage: ./run-tui.sh [--dir /custom/path]
cd "$SCRIPT_DIR"
exec ./target/release/claude-session-tui "$@"