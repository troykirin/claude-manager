#!/usr/bin/env bash
# Quick move script for directories without Claude projects

set -e

OLD_PATH="$1"
NEW_PATH="$2"

if [[ -z "$OLD_PATH" || -z "$NEW_PATH" ]]; then
    echo "Usage: $0 <old_path> <new_path>"
    exit 1
fi

if [[ ! -d "$OLD_PATH" ]]; then
    echo "Error: Directory not found: $OLD_PATH"
    exit 1
fi

echo "Moving: $OLD_PATH -> $NEW_PATH"
if mv "$OLD_PATH" "$NEW_PATH"; then
    echo "✓ Directory moved successfully"
    
    # Save undo info
    echo "$(date '+%Y-%m-%d %H:%M:%S')|simple_move|$OLD_PATH|$NEW_PATH" > ~/.claude/.last_move_operation
    echo "✓ Undo information saved (use 'cm-quick-undo' to revert)"
else
    echo "✗ Failed to move directory"
    exit 1
fi