#!/usr/bin/env bash
# Quick undo for directory moves

UNDO_FILE="$HOME/.claude/.last_move_operation"

if [[ ! -f "$UNDO_FILE" ]]; then
    echo "No undo information found"
    exit 1
fi

IFS='|' read -r timestamp operation old_path new_path <<< "$(cat "$UNDO_FILE")"

echo "Last operation: $operation at $timestamp"
echo "Reverting: $new_path -> $old_path"

if [[ -d "$new_path" ]]; then
    if mv "$new_path" "$old_path"; then
        echo "✓ Directory restored successfully"
        rm -f "$UNDO_FILE"
    else
        echo "✗ Failed to restore directory"
        exit 1
    fi
else
    echo "✗ Directory not found: $new_path"
    exit 1
fi