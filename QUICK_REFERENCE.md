# Claude Manager Quick Reference

## 🚀 Installation (2 minutes)

```bash
cd ~/nabia/tools/claude-manager
chmod +x install.sh
./install.sh
```

## 📋 Core Commands

### List & Status
```bash
cm list              # List all Claude projects
cm list /path       # List sessions in specific project
cm config           # Show current configuration
```

### Migrate Paths
```bash
cm migrate           # Interactive (asks for old/new paths)
cm migrate "/old" "/new"  # Non-interactive
CLAUDE_DRY_RUN=true cm migrate "/old" "/new"  # Preview only
```

### Move Sessions Between Projects
```bash
cm move              # Interactive
cm move "/old/proj" "/new/proj"  # Non-interactive
```

### Full Migration (Paths + Move)
```bash
cm full              # Interactive
cm full "/old/path" "/new/path" "/old-proj" "/new-proj"  # Non-interactive
```

### Undo Last Operation
```bash
cm undo              # Revert previous operation
```

## ⚙️ Configuration

```bash
export CLAUDE_DIR="$HOME/.claude"           # Claude directory
export CLAUDE_BACKUP_STRATEGY="file"        # file or project
export CLAUDE_INTERACTIVE="true"            # Confirm before changes
export CLAUDE_DRY_RUN="false"               # Preview without applying
```

Save to `~/.claude-manager.conf` for persistence.

## 🎯 Common Scenarios

### Renamed a project directory
```bash
cm migrate "/Users/tryk/dev/old-name" "/Users/tryk/dev/new-name"
```

### Moved project to different location
```bash
cm full "/old/location/proj" "/new/location/proj" "old-uuid" "new-uuid"
```

### Reorganizing multiple projects
```bash
CLAUDE_BACKUP_STRATEGY=project cm migrate "/dev" "/Production"
```

### Preview before committing
```bash
CLAUDE_DRY_RUN=true cm migrate "/old" "/new"  # See what would change
cm migrate "/old" "/new"                       # Actually do it
```

## 🛡️ Safety

**Automatic backups created before changes:**
- File-level: `session.jsonl.bak`
- Project-level: `project_backup_YYYYMMDD_HHMMSS.tar.gz`

**Restore from backup:**
```bash
# File-level
mv ~/.claude/projects/session.jsonl.bak ~/.claude/projects/session.jsonl

# Project-level
cd ~/.claude/projects
tar -xzf project_backup_20250108_143022.tar.gz
```

## 🐛 Troubleshooting

| Issue | Solution |
|-------|----------|
| `cm: command not found` | `source ~/.zshrc` or start new terminal |
| No sessions found | `cm list` to verify Claude installed |
| Permission errors | `chmod -R 755 ~/.claude` |
| Not sure about changes | Use `CLAUDE_DRY_RUN=true` first |
| Want to go back | `cm undo` or restore from `.bak` file |

## 📊 Backup Strategies

**File-level (default)** - Minimal disk, granular rollback
```bash
CLAUDE_BACKUP_STRATEGY=file cm migrate "/old" "/new"
```

**Project-level (safer)** - Complete snapshot, full restoration
```bash
CLAUDE_BACKUP_STRATEGY=project cm migrate "/old" "/new"
```

## 🔗 Architecture

Claude Manager solves: **When you move a project directory, Claude's hardcoded session paths break.**

```
Problem:  /Users/tryk/dev/my-project → /Users/tryk/NabiaTech/my-project
          Session files still reference old path ❌

Solution: cm migrate "/Users/tryk/dev/my-project" "/Users/tryk/NabiaTech/my-project"
          Updates all session references ✅
          /resume works again ✅
```

## 📚 Full Documentation

- **ONBOARDING.md**: Complete guide with scenarios
- **README.md**: Technical reference
- **federation-integration/**: Future federation patterns
- **TODO.md**: Known limitations

## 💡 Pro Tips

1. **Always use dry-run first**: `CLAUDE_DRY_RUN=true cm migrate ...`
2. **Use project-level backups for major changes**: `CLAUDE_BACKUP_STRATEGY=project`
3. **Check configuration**: `cm config` before any operation
4. **Enable debug mode if stuck**: `CLAUDE_DEBUG=1 cm migrate ...`
5. **Keep backup files until verified**: Don't delete `.bak` files immediately

## 🚦 Typical Workflow

```bash
# 1. Check what you have
cm list

# 2. Preview the change
CLAUDE_DRY_RUN=true cm migrate "/old" "/new"

# 3. Create backup (optional, automatic)
CLAUDE_BACKUP_STRATEGY=project cm list

# 4. Execute migration
cm migrate "/old" "/new"

# 5. Verify in Claude Code
# - Open a project conversation
# - Use /resume to test
# - Check working directory is correct ✅

# 6. Clean up backups (optional, when confident)
rm ~/.claude/projects/*.bak
```

---

**Last Updated**: 2025-01-08 | **Version**: claude-manager v1.0
