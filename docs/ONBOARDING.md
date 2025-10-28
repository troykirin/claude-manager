# Claude Manager Onboarding Guide

## 🎯 Executive Summary

**Claude Manager** solves a critical problem in your Claude Code workflow: when you move, rename, or reorganize your project directories, Claude's session files contain hardcoded paths that break. This tool automatically migrates those paths and reorganizes sessions, keeping your conversation history intact and `/resume` functionality working.

**Time to proficiency**: 10 minutes | **Installation**: 2 minutes

---

## 🏗️ Architectural Context

### The Problem It Solves

Claude Code stores conversations in `~/.claude/projects/` with embedded file paths in `.jsonl` session files:

```
~/.claude/projects/
├── project-uuid-1/
│   ├── sessions/
│   │   └── session-uuid.jsonl  ← Contains: {"path": "/Users/tryk/dev/my-project"}
│   └── metadata.json
```

When you move `/Users/tryk/dev/my-project` → `/Users/tryk/NabiaTech/my-project`, these paths become stale:
- ❌ `/resume` stops working
- ❌ Claude can't find your working directory
- ❌ Session context becomes invalid

**Claude Manager fixes this by**:
1. Finding all affected session files
2. Updating path references automatically
3. Optionally moving sessions to new project directories
4. Creating backups for safety

### Architecture Design

The tool follows a **three-layer architecture**:

```
Layer 1: Command Router (claude-manager.sh)
  ↓ Routes to appropriate subcommand
  ├─→ Migrate: Path substitution in session files
  ├─→ Move: Session directory relocation
  └─→ Full: Combined migrate + move operation

Layer 2: Safety Infrastructure
  ├─→ Backup system (file-level or project-level)
  ├─→ Dry-run preview before changes
  └─→ Undo capability for last operation

Layer 3: Federation Integration
  └─→ Loki event emission (optional)
  └─→ Federation status tracking
```

### Integration with Nabi Ecosystem

Claude Manager fits into your broader architecture:

```
Your CLI Stack:
  nabi (Rust router)
    ↓ delegates to
  nabi-python (Bash router)
    ↓ can route to
  claude-manager (session management)

Memory Integration:
  - Coordination: memchain_mcp (session operation state)
  - Knowledge: memory-kb (MCP server for querying sessions)
  - Long-term: Anytype (archived session insights)
```

Currently **not integrated** with Loki/federation, but the foundation is laid in `federation-integration/`.

---

## 📦 Installation

### Prerequisites

- Claude Code already installed and used (creates `~/.claude/` directory)
- Bash/Zsh shell
- Standard Unix tools: `find`, `sed`, `grep`

### Quick Install (Recommended)

```bash
cd ~/nabia/tools/claude-manager
chmod +x install.sh
./install.sh
```

This will:
1. Copy script to `~/.local/bin/claude-manager` ✅
2. Add sourcing to `~/.zshrc` or `~/.bashrc` ✅
3. Create optional config file `~/.claude-manager.conf` ✅
4. Set up convenient aliases (`cm`, `cm-migrate`, `cm-move`) ✅

### Manual Install (If Preferred)

```bash
# Copy the script
cp claude-manager.sh ~/.local/bin/claude-manager
chmod +x ~/.local/bin/claude-manager

# Add to your shell RC file (~/.zshrc or ~/.bashrc)
echo 'source ~/.local/bin/claude-manager' >> ~/.zshrc
```

### Verify Installation

```bash
# Reload your shell or source manually
source ~/.zshrc

# Test the tool
cm list          # Should show Claude projects
cm config        # Should show configuration
```

---

## 🎮 Core Usage Patterns

### Pattern 1: You Renamed a Project Directory

**Scenario**: Renamed `/Users/tryk/dev/old-name` → `/Users/tryk/dev/new-name`

**Command**:
```bash
cm migrate "/Users/tryk/dev/old-name" "/Users/tryk/dev/new-name"
```

**What happens**:
1. Finds all sessions referencing the old path
2. Creates backups (`.bak` files by default)
3. Replaces old path with new path in session files
4. Shows a detailed change report
5. Saves undo information

**Example output**:
```
[INFO] Found 3 sessions to migrate
[INFO] Backing up session: abc-123.jsonl → abc-123.jsonl.bak
[SUCCESS] Updated 12 path references in abc-123.jsonl
[SUCCESS] Updated 8 path references in def-456.jsonl
[SUCCESS] Updated 5 path references in ghi-789.jsonl
[SUCCESS] Migration complete: 25 changes in 3 sessions
```

### Pattern 2: You Moved Sessions to a New Project Directory

**Scenario**: Project moved AND needs new Claude project ID

**Commands**:
```bash
# Option A: Dry run to preview
CLAUDE_DRY_RUN=true cm full \
  "/old/path" "/new/path" \
  "old-project-uuid" "new-project-uuid"

# Option B: Actually execute
cm full \
  "/old/path" "/new/path" \
  "old-project-uuid" "new-project-uuid"
```

**What happens**:
1. Migrates all paths (old → new)
2. Moves session files between project directories
3. Updates project metadata
4. Preserves session chronology

### Pattern 3: Organization Restructure (Multiple Projects)

**Scenario**: Moving multiple projects from `/dev/` → `/Production/`

```bash
# Migrate all sessions with path substitution
CLAUDE_BACKUP_STRATEGY=project cm migrate "/dev" "/Production"
```

This creates a project-level backup (`.tar.gz`) of your entire `~/.claude/projects/` directory before making changes.

### Pattern 4: Preview Before Committing

**Scenario**: Not sure if changes are correct? Use dry-run.

```bash
# See what WOULD change without making changes
CLAUDE_DRY_RUN=true cm migrate "/old/path" "/new/path"

# Then execute for real
cm migrate "/old/path" "/new/path"
```

---

## ⚙️ Configuration

### Environment Variables

```bash
# Set your preferences (or put in ~/.claude-manager.conf)
export CLAUDE_DIR="$HOME/.claude"           # Where Claude stores projects
export CLAUDE_BACKUP_STRATEGY="file"        # file or project
export CLAUDE_INTERACTIVE="true"            # Confirm before changes
export CLAUDE_DRY_RUN="false"               # Preview without applying
```

### Config File (~/.claude-manager.conf)

```bash
# Claude Manager Configuration
export CLAUDE_DIR="$HOME/.claude"
export CLAUDE_BACKUP_STRATEGY="project"     # Use full project backups
export CLAUDE_INTERACTIVE="true"            # Always confirm before changes
export CLAUDE_DRY_RUN="false"               # Actually apply changes
```

### Backup Strategies Explained

**File-Level (`file`)** - Default
- Creates `.bak` files for each modified session
- Minimal disk usage (only changed files)
- Good for small changes and iterative migrations
- Rollback: `mv session.jsonl.bak session.jsonl`

**Project-Level (`project`)** - Safer
- Creates complete `.tar.gz` snapshot before ANY changes
- Timestamp-based naming: `project_backup_20250108_143022.tar.gz`
- Complete restoration possible
- Recommended for major migrations
- Rollback: `tar -xzf project_backup_20250108_143022.tar.gz`

---

## 🔄 Command Reference

### Essential Commands

| Command | Aliases | Purpose |
|---------|---------|---------|
| `cm list` | `cm ls`, `cm l` | List all Claude projects and sessions |
| `cm config` | `cm cfg` | Show current configuration |
| `cm migrate` | `cm m` | Update paths in sessions |
| `cm move` | `cm mv` | Move sessions between projects |
| `cm full` | `cm f` | Complete migration (paths + move) |
| `cm undo` | — | Undo the last operation |

### Practical Examples

```bash
# List all your Claude projects
cm list

# Show what sessions exist for a project
cm list path/to/project

# See current configuration
cm config

# Migrate paths (interactive mode)
cm migrate

# Migrate paths (non-interactive with specific paths)
cm migrate "/old/path" "/new/path"

# Move sessions between projects
cm move "/old/project" "/new/project"

# Do everything: migrate AND move
cm full "/old/path" "/new/path" "/old-proj-id" "/new-proj-id"

# Preview without changing anything
CLAUDE_DRY_RUN=true cm migrate "/old" "/new"

# Undo the last operation
cm undo
```

---

## 🛡️ Safety Features

### Automatic Backups

Every operation creates backups before making changes:

```bash
# File-level backup (default)
~/.claude/projects/myproj/sessions/session.jsonl.bak

# Project-level backup
~/.claude/projects/project_backup_20250108_143022.tar.gz
```

### Dry-Run Preview

See exactly what WILL change before committing:

```bash
CLAUDE_DRY_RUN=true cm migrate "/old" "/new"

# Output shows:
# [DRY RUN] Would update 12 path references
# [DRY RUN] Would modify 3 session files
# [DRY RUN] No changes applied
```

### Interactive Confirmations

By default, you confirm before any changes:

```bash
# Shows changes → asks for confirmation
cm migrate "/old" "/new"

# Output:
# Found 3 sessions with old paths
# Would update 25 path references
# Continue? (y/n)
```

### Undo Capability

```bash
# Automatically saves undo information after each operation
# ~/.claude/.last_move_operation

# Revert to previous state
cm undo

# Shows what was undone:
# [INFO] Last operation: migrate at 2025-01-08 14:30:22
# [INFO] Restored from: /path/to/backup
```

---

## 🔍 Real-World Scenarios

### Scenario A: Renaming During Development

```bash
# You started with ~/dev/chatbot but want ~/dev/ai-chatbot
mv ~/dev/chatbot ~/dev/ai-chatbot

# Update Claude sessions
cm migrate "~/dev/chatbot" "~/dev/ai-chatbot"

# Now /resume works in Claude again! ✅
```

### Scenario B: Moving to Production Structure

```bash
# Moving from development to archived projects
mv ~/dev/archived-project ~/archive/projects/archived-project

# Use project-level backup for safety
CLAUDE_BACKUP_STRATEGY=project cm migrate \
  "~/dev/archived-project" \
  "~/archive/projects/archived-project"

# Verify changes
cm list ~/archive/projects/archived-project
```

### Scenario C: Full Reorganization with Backups

```bash
# Create full backup BEFORE major changes
CLAUDE_BACKUP_STRATEGY=project cm list

# Now do the migration with full safety
CLAUDE_BACKUP_STRATEGY=project cm full \
  "~/old/structure" "~/new/structure" \
  "old-uuid" "new-uuid"

# If something goes wrong:
tar -xzf ~/.claude/projects/project_backup_*.tar.gz
```

### Scenario D: Checking What Needs Migration

```bash
# See all Claude projects first
cm list

# Check specific project sessions
cm list ~/my/project/path

# Dry-run to see what would change
CLAUDE_DRY_RUN=true cm migrate "old" "new"
```

---

## 🐛 Troubleshooting

### Issue: "No sessions found"

```bash
# Check if Claude directory exists
ls -la ~/.claude/projects/

# If empty, Claude hasn't been used yet
# Start a conversation in Claude Code first
```

### Issue: Permission Denied

```bash
# Fix permissions on Claude directory
chmod -R 755 ~/.claude/

# Retry the operation
cm migrate "/old" "/new"
```

### Issue: Sessions Not Updating

```bash
# Enable debug mode to see detailed logs
CLAUDE_DEBUG=1 cm migrate "/old" "/new"

# Check for syntax errors
cm config
cat ~/.claude/projects/*/sessions/*.jsonl | head -c 200

# Try dry-run first
CLAUDE_DRY_RUN=true cm migrate "/old" "/new"
```

### Issue: Want to Restore from Backup

```bash
# From file-level backups
find ~/.claude -name "*.bak" -type f
mv ~/.claude/projects/session.jsonl.bak ~/.claude/projects/session.jsonl

# From project-level backup
cd ~/.claude/projects/
tar -tzf project_backup_20250108_143022.tar.gz  # List contents
tar -xzf project_backup_20250108_143022.tar.gz  # Extract
```

### Issue: Tool Not Found After Installation

```bash
# Reload shell configuration
source ~/.zshrc  # or ~/.bashrc

# Or start a new terminal

# Verify installation
which claude-manager
cm list
```

---

## 📚 Advanced Topics

### Custom Configuration File

Create `~/.claude-manager.conf` for persistent settings:

```bash
#!/bin/bash
# Claude Manager Configuration

# Your Claude installation directory
export CLAUDE_DIR="$HOME/.claude"

# Backup strategy: file or project
export CLAUDE_BACKUP_STRATEGY="project"

# Interactive mode (true/false)
export CLAUDE_INTERACTIVE="true"

# Dry-run by default (true/false)
export CLAUDE_DRY_RUN="false"

# Enable debug logging
export CLAUDE_DEBUG="0"
```

Then source it in your shell RC:
```bash
source ~/.claude-manager.conf
```

### Automating Migrations

For scripting (non-interactive):

```bash
#!/bin/bash
set -e

CLAUDE_INTERACTIVE=false \
CLAUDE_BACKUP_STRATEGY=project \
cm full \
  "/Users/tryk/dev/legacy-project" \
  "/Users/tryk/nabia/projects/migrated-project" \
  "old-uuid-here" \
  "new-uuid-here"

echo "Migration complete!"
```

### Monitoring Session Health

```bash
# List all projects
cm list

# Check specific project
cm list /path/to/project

# Count total sessions
find ~/.claude/projects -name "*.jsonl" | wc -l

# Find sessions with stale paths
CLAUDE_DEBUG=1 cm list 2>&1 | grep -i "path\|not found"
```

---

## 🔗 Integration Opportunities

### With Your Nabi CLI

Currently, claude-manager is standalone. Future integration could be:

```bash
# Hypothetical future command
nabi claude migrate "/old/path" "/new/path"

# Would route to: nabi-python → claude-manager
```

This would require adding to `~/.config/nabi/tools/claude-manager.toml`.

### With Federation Events (Future)

The `federation-integration/` directory has TypeScript foundations for:
- Emitting migration events to Loki
- Tracking session changes in memory-kb
- Federation coordination on RPi

Currently not activated, but the structure exists.

---

## 🎓 Learning Path

### Day 1: Get Comfortable
1. Install the tool: `./install.sh`
2. List your projects: `cm list`
3. Preview a migration: `CLAUDE_DRY_RUN=true cm migrate "old" "new"`

### Day 2: Use It for Real
1. Execute an actual migration: `cm migrate "old" "new"`
2. Verify the change worked
3. Test `/resume` in Claude Code
4. Try `cm undo` to understand rollback

### Day 3: Advanced Usage
1. Create `~/.claude-manager.conf` with your preferences
2. Try project-level backups: `CLAUDE_BACKUP_STRATEGY=project cm migrate ...`
3. Use dry-run for complex scenarios
4. Explore the federation integration structure

---

## 📊 Key Files Overview

| File | Purpose |
|------|---------|
| `claude-manager.sh` | Main script (2100 lines) |
| `claude-session-context.sh` | Session metadata handling |
| `claude-session-tui/` | Rust TUI for interactive selection |
| `cm-quick-move.sh` | Quick move shortcut |
| `cm-quick-undo.sh` | Quick undo shortcut |
| `federation-integration/` | Loki event emission (future) |
| `python/` | Python utilities |
| `docs/` | Documentation |

---

## ✅ You're Ready!

You now understand:
- ✅ Why claude-manager exists (Claude paths break when directories move)
- ✅ How it works (finds sessions, updates paths, creates backups)
- ✅ How to install it (5 minutes)
- ✅ How to use it (4 main commands)
- ✅ How to stay safe (backups, dry-run, undo)
- ✅ How it fits in your architecture (federation-ready, nabi-compatible)

### Next Steps

1. **Install**: `cd ~/nabia/tools/claude-manager && ./install.sh`
2. **Explore**: `cm list` to see your current projects
3. **Practice**: Use dry-run on a real migration
4. **Integrate**: Consider adding to your nabi tool registry

---

## 📖 Additional Resources

- **README.md**: Quick reference and command syntax
- **federation-integration/README.md**: Future federation patterns
- **docs/REPOSITORY_ANALYSIS_2025-09-24.md**: Detailed technical analysis
- **TODO.md**: Known limitations and future enhancements

---

## 🤝 Support & Feedback

For issues or improvements:
1. Check **Troubleshooting** section above
2. Review `federation-integration/README.md` for architecture insights
3. Enable `CLAUDE_DEBUG=1` for detailed logging
4. Check the TODO.md for known limitations

**Architecture Note**: This tool is federation-ready but not yet integrated with Loki/memchain. The hooks system and federation patterns are laid out in `federation-integration/` if you want to extend it.
