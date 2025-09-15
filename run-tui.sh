#!/usr/bin/env bash
# TUI Runner with proper terminal handling

# Ensure we're in a TTY
if [ ! -t 0 ] || [ ! -t 1 ]; then
    echo "Error: TUI requires an interactive terminal (TTY)"
    echo "Please run this command directly in a terminal, not through pipes or redirection"
    exit 1
fi

# Set terminal environment if not set
export TERM="${TERM:-xterm-256color}"

# Build the TUI if needed
if [ ! -f "target/debug/claude-session-tui" ]; then
    echo "Building TUI component..."
    cargo build --bin claude-session-tui --features tui
fi

# Run with proper terminal setup
exec cargo run --bin claude-session-tui --features tui -- "$@"