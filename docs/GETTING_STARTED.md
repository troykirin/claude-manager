# Getting Started with Claude Manager

Welcome! This guide will have you up and running in **5 minutes**.

## Step 1: Installation (2 minutes)

```bash
# Navigate to the tool directory
cd ~/nabia/tools/claude-manager

# Run the installer
chmod +x install.sh
./install.sh
```

The installer will:
- ✅ Copy the script to `~/.local/bin/claude-manager`
- ✅ Add sourcing to your shell RC (`~/.zshrc` or `~/.bashrc`)
- ✅ Ask if you want to configure preferences
- ✅ Show summary and next steps

**Note**: Answer "y" to the shell RC question for easy access.

## Step 2: Verify Installation (30 seconds)

```bash
# Start a fresh terminal OR source your RC file
source ~/.zshrc  # or ~/.bashrc

# Verify the tool works
cm list
```

You should see:
```
[INFO] Scanning Claude projects in ~/.claude/projects/
Found X project(s)
```

If you see "Claude directory not found", you haven't used Claude Code yet. Start a conversation in Claude Code first.

## Step 3: Understand Your Current State (1 minute)

```bash
# See all your Claude projects and sessions
cm list

# Show configuration that will be used
cm config
```

This shows:
- How many projects you have
- Default backup strategy
- Where backups will be stored

## Step 4: Try a Dry Run (1 minute)

Before making any real changes, preview what claude-manager would do:

```bash
# Pick a project directory and a "new" name
# DO NOT actually move/rename the directory yet!

CLAUDE_DRY_RUN=true cm migrate \
  "/Users/tryk/dev/my-actual-project" \
  "/Users/tryk/dev/my-actual-project-renamed"
```

Output will show:
```
[DRY RUN] Would update X sessions
[DRY RUN] Would replace Y path references
[DRY RUN] No changes applied
```

This is **completely safe** - nothing changes.

## Step 5: Your First Real Migration (if needed)

If you actually have a project to migrate:

```bash
# FIRST: Move the actual directory
mv /Users/tryk/dev/old-name /Users/tryk/dev/new-name

# THEN: Update Claude sessions
cm migrate \
  "/Users/tryk/dev/old-name" \
  "/Users/tryk/dev/new-name"
```

It will:
1. Create a backup (`.bak` files by default)
2. Show what's being changed
3. Ask for confirmation (interactive mode)
4. Apply changes
5. Show success summary

## ✅ Verification Checklist

After any migration, verify it worked:

```bash
# 1. Check Claude sessions were updated
cm list /Users/tryk/dev/new-name

# 2. Open Claude Code
# 3. Navigate to a project with migrated sessions
# 4. Try /resume on a previous conversation
# 5. Verify the working directory is correct
```

✅ If `/resume` works and shows the correct path → **Success!**

## 🎯 Quick Command Reference

```bash
# See your projects
cm list

# View current config
cm config

# Test a migration (safe, no changes)
CLAUDE_DRY_RUN=true cm migrate "/old/path" "/new/path"

# Actually migrate paths
cm migrate "/old/path" "/new/path"

# Undo if something went wrong
cm undo

# Move sessions between project directories (advanced)
cm move "/old/project" "/new/project"
```

## 🛡️ Safety Reminders

1. **Always use dry-run first**: `CLAUDE_DRY_RUN=true cm migrate ...`
2. **Backups are automatic**: `.bak` files created for each modified session
3. **Undo works**: `cm undo` reverts the last operation
4. **Don't panic**: If something goes wrong, restore from backup (see Troubleshooting)

## 🐛 Troubleshooting

### Issue: "cm: command not found"
```bash
# Solution: Reload shell configuration
source ~/.zshrc
```

### Issue: "No sessions found"
```bash
# Solution: You haven't used Claude Code yet
# Start a conversation in Claude Code, then try again
cm list
```

### Issue: "Permission denied"
```bash
# Solution: Fix Claude directory permissions
chmod -R 755 ~/.claude
```

### Issue: "I want to undo my last operation"
```bash
# Solution: Use the undo command
cm undo
```

### Issue: "I accidentally deleted a backup"
```bash
# Solution: Check if project-level backup exists
ls ~/.claude/projects/project_backup_*.tar.gz
tar -xzf ~/.claude/projects/project_backup_*.tar.gz
```

## 📚 Next Steps

Now that you're installed:

1. **Read the Quick Reference**: See `QUICK_REFERENCE.md` for command cheat sheet
2. **Read the Full Onboarding**: See `ONBOARDING.md` for detailed scenarios and advanced usage
3. **Configure Preferences** (optional):
   - Edit `~/.claude-manager.conf`
   - Change backup strategy, interactive mode, etc.
4. **Integrate with Nabi** (future):
   - Currently standalone, but federation-ready

## 🎓 Learning Goals

- ✅ Understand what claude-manager does (saves broken Claude sessions when you move directories)
- ✅ Know how to install it (2-minute installer)
- ✅ Know how to use it (4 main commands)
- ✅ Know how to stay safe (dry-run, backups, undo)

## 💡 Pro Tips

1. **Use dry-run for everything first**: Helps you understand what will change
2. **Save full backups before big changes**: `CLAUDE_BACKUP_STRATEGY=project`
3. **Keep terminal open**: Save your undo file location if something goes wrong
4. **Test with `/resume`**: Verify migration worked by resuming a conversation

## 🚀 You're Ready!

You've completed the getting started guide. Now you can:
- Install the tool (done!)
- Use it safely (dry-run)
- Migrate your projects (when needed)
- Recover if something goes wrong (undo, backups)

For more details, see:
- **QUICK_REFERENCE.md** - Command cheat sheet
- **ONBOARDING.md** - Complete guide with real scenarios
- **README.md** - Technical reference

---

**Questions?** Check the Troubleshooting section or enable debug mode:
```bash
CLAUDE_DEBUG=1 cm list
```

**Happy migrating!** 🎉
