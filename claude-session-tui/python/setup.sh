#!/bin/bash
# Quick setup for Claude Manager TUI

echo "Setting up Claude Manager TUI..."

# Check for Python 3
if ! command -v python3 &> /dev/null; then
    echo "Python 3 is required but not found!"
    exit 1
fi

# Install dependencies
echo "Installing Python dependencies..."
pip3 install --user rich prompt_toolkit python-dateutil

echo "Testing the installation..."
python3 claude_manager_tui.py list

echo ""
echo "Setup complete! You can now run:"
echo "  python3 claude_manager_tui.py        # Interactive mode"
echo "  python3 claude_manager_tui.py list   # List all sessions"
echo "  python3 claude_manager_tui.py search <query>  # Search sessions"