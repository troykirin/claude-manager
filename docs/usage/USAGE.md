# Usage Guide

## Command Reference

### `cm migrate` - Update Session Paths

Update path references in Claude session files without moving directories.

**Usage:**
```bash
cm migrate <old_path> <new_path> [project_dir]
```

**Interactive mode (no arguments):**
```bash
cm migrate
# Will prompt for:
# 1. Project selection
# 2. Old path detection
# 3. New path input
```

**Examples:**
```bash
# Update paths after renaming directory
cm migrate "/Users/tryk/dev/old-name" "/Users/tryk/dev/new-name"

# Migrate specific project
cm migrate "/old/path" "/new/path" "$HOME/.claude/projects/-Users-tryk-dev-project"

# Dry run
CLAUDE_DRY_RUN=true cm migrate "/old" "/new"
```

### `cm move` - Move Directory and Sessions

Atomically move source directory and update Claude sessions.

**Usage:**
```bash
cm move <old_path> <new_path>
```

**Examples:**
```bash
# Move project directory
cm move "/Users/tryk/dev/project" "/Users/tryk/projects/project"

# Non-interactive
CLAUDE_INTERACTIVE=false cm move "/old" "/new"

# Preview changes
CLAUDE_DRY_RUN=true cm move "/old" "/new"
```

**What it does:**
1. Moves source directory: `old_path` → `new_path`
2. Moves Claude project directory
3. Updates all session path references
4. Saves undo information

### `cm list` - List Projects and Sessions

**Usage:**
```bash
# List all projects
cm list

# List sessions in a project
cm list <project_dir>
```

**Examples:**
```bash
# Show all projects with session counts
cm list

# Show sessions in specific project
cm list ~/.claude/projects/-Users-tryk-dev-project
```

**Output:**
```
Claude projects:
  -Users-tryk-dev-project-a (12 sessions)
  -Users-tryk-dev-project-b (5 sessions)
```

### `cm undo` - Undo Last Operation

Revert the last `move` operation.

**Usage:**
```bash
cm undo
```

**What it undoes:**
- Restores moved directories
- Restores moved Claude project directories
- Re-updates session paths to original values

**Note:** Only the most recent operation can be undone.

### `cm config` - Show Configuration

Display current environment variable configuration.

**Usage:**
```bash
cm config
```

**Output:**
```
Current configuration:
  CLAUDE_DIR: /home/tryk/.claude
  BACKUP_STRATEGY: file
  INTERACTIVE: true
  DRY_RUN: false
```

### `cm help` - Show Help

Display command reference and examples.

**Usage:**
```bash
cm help
```

## Environment Variables

### `CLAUDE_DIR`

Claude Code directory location.

**Default:** `$HOME/.claude`

**Example:**
```bash
export CLAUDE_DIR="$HOME/.claude"
cm list
```

### `CLAUDE_BACKUP_STRATEGY`

Backup strategy for sessions.

**Values:** `file` | `project`
**Default:** `file`

**File-level (`file`):**
- Creates `.bak` for each modified session
- Minimal disk usage
- Granular rollback

**Project-level (`project`):**
- Creates timestamped `.tar.gz` of entire project
- Complete snapshot
- Easy full restoration

**Example:**
```bash
CLAUDE_BACKUP_STRATEGY=project cm migrate "/old" "/new"
```

### `CLAUDE_INTERACTIVE`

Enable interactive prompts and confirmations.

**Values:** `true` | `false`
**Default:** `true`

**Example:**
```bash
# Skip all prompts
CLAUDE_INTERACTIVE=false cm move "/old" "/new"
```

### `CLAUDE_DRY_RUN`

Preview mode - show what would happen without making changes.

**Values:** `true` | `false`
**Default:** `false`

**Example:**
```bash
# Preview migration
CLAUDE_DRY_RUN=true cm migrate "/old" "/new"
```

### `CLAUDE_DEBUG`

Enable debug logging.

**Values:** `0` | `1`
**Default:** `0`

**Example:**
```bash
CLAUDE_DEBUG=1 cm migrate "/old" "/new"
```

## Common Workflows

### Workflow 1: Rename Project Directory

**Scenario:** Renamed `/Users/tryk/dev/my-project` to `/Users/tryk/dev/awesome-project`

```bash
# Option A: Migrate paths only (directory already renamed)
cm migrate "/Users/tryk/dev/my-project" "/Users/tryk/dev/awesome-project"

# Option B: Let claude-manager handle the rename
mv /Users/tryk/dev/awesome-project /Users/tryk/dev/my-project  # Rename back
cm move "/Users/tryk/dev/my-project" "/Users/tryk/dev/awesome-project"
```

### Workflow 2: Move Project to New Location

**Scenario:** Moving `/Users/tryk/dev/project` to `/Users/tryk/projects/project`

```bash
cm move "/Users/tryk/dev/project" "/Users/tryk/projects/project"
```

### Workflow 3: Reorganize Multiple Projects

**Scenario:** Moving all projects from `/Users/tryk/dev/*` to `/Users/tryk/projects/*`

```bash
# Preview first
CLAUDE_DRY_RUN=true cm move "/Users/tryk/dev/project-a" "/Users/tryk/projects/project-a"

# Execute with project-level backups
CLAUDE_BACKUP_STRATEGY=project cm move "/Users/tryk/dev/project-a" "/Users/tryk/projects/project-a"
CLAUDE_BACKUP_STRATEGY=project cm move "/Users/tryk/dev/project-b" "/Users/tryk/projects/project-b"
```

### Workflow 4: Undo Mistake

**Scenario:** Moved to wrong location

```bash
# Move operation
cm move "/old/path" "/wrong/path"

# Realize mistake - undo immediately
cm undo

# Now move to correct location
cm move "/old/path" "/correct/path"
```

## Aliases

The following aliases are available:

```bash
cm              # claude_manager
cm-migrate      # claude_manager migrate
cm-move         # claude_manager move
cm-list         # claude_manager list
```

## Tips and Best Practices

### 1. Always Test with Dry Run First

```bash
CLAUDE_DRY_RUN=true cm move "/old" "/new"
```

### 2. Use Project Backups for Major Changes

```bash
CLAUDE_BACKUP_STRATEGY=project cm migrate "/old" "/new"
```

### 3. Verify Changes

```bash
# After migration
cm list <project_dir>
# Check that paths are correct
```

### 4. Keep Backups Safe

Backups are stored alongside sessions:
- File backups: `session-uuid.jsonl.bak`
- Project backups: `project_backup_20240115_143022.tar.gz`

### 5. Use Interactive Mode for Complex Migrations

```bash
# Let claude-manager guide you
cm migrate
```

## Troubleshooting

### Migration Not Finding Sessions

```bash
# List projects to find correct path
cm list

# Check what paths are in sessions
cm list <project_dir>
```

### Python 3 Required Error

```bash
# Install python3
sudo apt install python3  # Linux
brew install python3      # macOS
```

### Backup Restoration

**Restore file-level backup:**
```bash
mv session-uuid.jsonl.bak session-uuid.jsonl
```

**Restore project-level backup:**
```bash
cd ~/.claude/projects/
tar -xzf project_backup_20240115_143022.tar.gz
```

## Next Steps

- [Pairing with riff-cli](../integration/PAIRING.md) - Session monitoring
- [NabiÓS Integration](../integration/NABIOS.md) - Federation features
