#!/usr/bin/env bash

# Example: Basic path migration
# Updates session paths after renaming a directory

set -e

echo "=== Basic Migration Example ==="
echo ""
echo "This example shows how to migrate session paths after renaming a directory."
echo ""

# Configuration
OLD_PATH="/Users/tryk/dev/my-old-project"
NEW_PATH="/Users/tryk/dev/my-new-project"

echo "Scenario:"
echo "  You renamed: $OLD_PATH"
echo "            → $NEW_PATH"
echo ""

# Preview first
echo "Step 1: Preview migration (dry run)"
echo "Command: CLAUDE_DRY_RUN=true cm migrate \"$OLD_PATH\" \"$NEW_PATH\""
echo ""

CLAUDE_DRY_RUN=true cm migrate "$OLD_PATH" "$NEW_PATH"

echo ""
echo "Step 2: Execute migration"
echo "Command: cm migrate \"$OLD_PATH\" \"$NEW_PATH\""
echo ""

# Uncomment to execute:
# cm migrate "$OLD_PATH" "$NEW_PATH"

echo "Migration completed!"
echo ""
echo "To undo: cm undo"
