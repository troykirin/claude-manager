# Pairing with riff-cli

**claude-manager** and **riff-cli** are complementary tools for managing Claude Code sessions:

- **claude-manager**: Migrates and organizes session files when you move projects
- **riff-cli**: Monitors active Claude sessions and provides real-time insights

## What is riff-cli?

[riff-cli](https://github.com/yourusername/riff-cli) is a session monitoring tool that:
- Watches Claude Code sessions in real-time
- Provides UUID management and session tracking
- Offers transcript analysis and flow visualization
- Complements claude-manager's migration capabilities

## Installation

```bash
# Install claude-manager
git clone https://github.com/yourusername/claude-manager.git
cd claude-manager && ./install.sh

# Install riff-cli (separate repository)
git clone https://github.com/yourusername/riff-cli.git
cd riff-cli && ./install.sh
```

## Typical Workflow

### 1. Monitor with riff-cli

```bash
# Start monitoring Claude sessions
riff watch

# List active sessions
riff list
```

### 2. Move Project with claude-manager

When you reorganize your projects:

```bash
# Move project directory and update sessions
cm move "/Users/tryk/dev/old-project" "/Users/tryk/projects/new-project"
```

### 3. Verify with riff-cli

```bash
# Check updated sessions
riff list

# Verify paths are correct
riff show <session-uuid>
```

## Use Cases

### Use Case 1: Pre-Migration Check

```bash
# Before moving a project, check active sessions
riff list --project "old-project"

# Close active sessions in Claude Code
# Then migrate
cm move "/old/path" "/new/path"

# Verify sessions updated
riff list --project "new-project"
```

### Use Case 2: Session Recovery

```bash
# Find sessions referencing a specific path
riff find --path "/old/project/path"

# Migrate those sessions
cm migrate "/old/project/path" "/new/project/path"
```

### Use Case 3: Bulk Organization

```bash
# Use riff to analyze session distribution
riff stats

# Organize projects with claude-manager
cm move "/dev/project-a" "/projects/project-a"
cm move "/dev/project-b" "/projects/project-b"

# Verify with riff
riff stats
```

## Integration Points

| Task | Tool | Command |
|------|------|---------|
| Monitor active sessions | riff-cli | `riff watch` |
| List all sessions | riff-cli | `riff list` |
| Find sessions by path | riff-cli | `riff find --path <path>` |
| Move project directory | claude-manager | `cm move <old> <new>` |
| Update session paths | claude-manager | `cm migrate <old> <new>` |
| Verify migration | riff-cli | `riff list` |
| Undo move | claude-manager | `cm undo` |

## Configuration

Both tools use `~/.claude/` as the default Claude directory. Set once for both:

```bash
# In ~/.claude-manager.conf and ~/.riff-cli.conf
export CLAUDE_DIR="$HOME/.claude"
```

## Example: Complete Migration Workflow

```bash
# 1. Check current state with riff
riff list --project "my-project"
# Output: 5 sessions found

# 2. Close any active Claude sessions
# (Verify in Claude Code UI)

# 3. Preview migration
CLAUDE_DRY_RUN=true cm move "/Users/tryk/dev/my-project" "/Users/tryk/projects/my-project"

# 4. Execute migration
cm move "/Users/tryk/dev/my-project" "/Users/tryk/projects/my-project"

# 5. Verify with riff
riff list --project "my-project"
# Output: 5 sessions found (paths updated)

# 6. Open Claude Code and test /resume
```

## Troubleshooting

### Sessions Not Updating

```bash
# Check with riff which sessions reference old path
riff find --path "/old/path"

# Migrate specific project
cm migrate "/old/path" "/new/path" <project-dir>

# Verify
riff find --path "/new/path"
```

### Active Session Lock

```bash
# riff can show active sessions
riff watch

# Close Claude Code before migrating
# Then migrate
cm move "/old" "/new"
```

## Future Enhancements

Planned integrations between claude-manager and riff-cli:

- **Automatic session closure detection** - riff warns if sessions active during migration
- **Path validation** - riff validates session paths after migration
- **Session analytics** - riff provides migration impact reports
- **Shared configuration** - Unified config file for both tools

## Links

- [riff-cli GitHub](https://github.com/yourusername/riff-cli)
- [claude-manager GitHub](https://github.com/yourusername/claude-manager)
- [Usage Guide](../usage/USAGE.md)

---

**Together, claude-manager and riff-cli provide complete Claude Code session management.**
