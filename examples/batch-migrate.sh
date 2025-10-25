#!/usr/bin/env bash

# Example: Batch migration of multiple projects
# Useful when reorganizing entire directory structures

set -e

echo "=== Batch Migration Example ==="
echo ""
echo "This example migrates multiple projects from /dev to /projects"
echo ""

# Configuration
OLD_BASE="/Users/tryk/dev"
NEW_BASE="/Users/tryk/projects"

PROJECTS=(
    "project-a"
    "project-b"
    "project-c"
)

echo "Migrating ${#PROJECTS[@]} projects:"
for project in "${PROJECTS[@]}"; do
    echo "  • $project"
done
echo ""

# Preview all migrations
echo "Step 1: Preview all migrations"
for project in "${PROJECTS[@]}"; do
    old_path="$OLD_BASE/$project"
    new_path="$NEW_BASE/$project"

    echo ""
    echo "Preview: $old_path → $new_path"
    CLAUDE_DRY_RUN=true cm move "$old_path" "$new_path" || true
done

echo ""
echo "Step 2: Execute migrations with project-level backups"
echo ""

# Uncomment to execute:
# for project in "${PROJECTS[@]}"; do
#     old_path="$OLD_BASE/$project"
#     new_path="$NEW_BASE/$project"
#
#     echo "Migrating: $project"
#     CLAUDE_BACKUP_STRATEGY=project cm move "$old_path" "$new_path"
# done

echo "Batch migration completed!"
echo ""
echo "Backups stored as: project_backup_YYYYMMDD_HHMMSS.tar.gz"
