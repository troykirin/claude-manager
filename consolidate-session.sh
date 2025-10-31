#!/bin/bash
# Consolidate orphaned Claude session into permanent location
# Usage: consolidate-claude-session [UUID]

set -e

UUID="${1:-c54e2ee3-d405-4215-ac30-80546c0abfd1}"
CLAUDE_DIR="${CLAUDE_DIR:-$HOME/.claude}"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║ Claude Session Consolidation                                  ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Consolidating session: $UUID"
echo ""

# Run merge command with user confirmation
if ! cm organize merge "$UUID" \
  --from "$CLAUDE_DIR/projects/-Users-tryk-dev-git" \
  --to "$CLAUDE_DIR/projects/-Users-tryk--local-share-claude-sessions-personal-config"; then
  echo ""
  echo "❌ Merge failed or was cancelled - skipping cleanup"
  echo "Orphaned data preserved in: $CLAUDE_DIR/projects/-Users-tryk-dev-git/"
  exit 1
fi

echo ""
echo "=== Cleanup ==="
echo "Removing orphaned metadata..."
rm -rf "$CLAUDE_DIR/projects/-Users-tryk-dev-git"
echo "✅ Orphaned project directory removed"

if [[ -d "$HOME/dev/git" ]] && [[ ! -L "$HOME/dev/git" ]]; then
    if find "$HOME/dev/git" -mindepth 1 -type f 2>/dev/null | grep -q .; then
        echo "⚠️  ~/dev/git contains files (not deleting)"
    else
        rmdir "$HOME/dev/git" 2>/dev/null && echo "✅ ~/dev/git removed"
    fi
fi

echo ""
echo "✅ Session consolidation complete!"
echo ""
echo "Consolidated session:"
echo "  Location: $CLAUDE_DIR/projects/-Users-tryk--local-share-claude-sessions-personal-config/"
echo "  UUID:     $UUID"
