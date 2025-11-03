# Claude Manager

A comprehensive CLI tool for migrating Claude projects and updating session paths when you move or rename your development directories.

## The Problem

When using Claude Code:
1. You have conversations that reference specific working directories
2. Claude stores these sessions in `~/.claude/projects/` with paths embedded in `.jsonl` files
3. When you move or rename your project directory, the session paths become invalid
4. The built-in `/resume` function no longer works because it can't find the original paths

## The Solution

This tool automates the complete migration workflow:
1. **Backup** your sessions (file-level `.bak` or project-level `.tar.gz`)
2. **Find and replace** all path references in session files
3. **Move sessions** to new project directories
4. **Verify** changes with detailed reporting

## Features

- 🔄 **Complete Migration**: Handles paths and project moves in one command
- 🛡️ **Safety First**: Multiple backup strategies with automatic rollback
- 🎯 **Interactive Mode**: Guided workflow with confirmations
- 🤖 **Automation Ready**: Non-interactive mode for scripting
- 📊 **Detailed Reporting**: See exactly what changed
- 🔍 **Auto-Detection**: Automatically finds existing paths in sessions
- ⚡ **Dry Run Mode**: Preview changes before applying
- 🏥 **Health Diagnostics**: Detect session corruption patterns and assess health

## Requirements

- Bash 4.0+
- Claude Code (stores sessions in `~/.claude/`)
- Optional: bats-core for running tests

## Optional Features

The tool includes **optional** federation integration capabilities for advanced multi-system coordination:
- Session recovery audit trails (Loki logging)
- Issue creation from sessions (Linear integration)
- Event streaming (NATS messaging)
- Distributed state management (SurrealDB)

**These are completely OPTIONAL.** The core tool works perfectly standalone for all session management, migration, and diagnostics features without any external services.

To enable federation features, see `federation-integration/README.md`.

## Installation

### Option 1: Full Installation (Recommended)

```bash
# Clone or download the files
chmod +x install.sh
./install.sh
```

This will:
- Install the tool to `~/.local/bin/claude-project-migrator`
- Add it to your shell rc file (`.bashrc` or `.zshrc`)
- Set up configuration with your preferences
- Create convenient aliases

### Option 2: Manual Installation

```bash
# Copy the main script
cp claude-project-migrator.sh ~/.local/bin/claude-project-migrator
chmod +x ~/.local/bin/claude-project-migrator

# Add to your shell rc file
echo 'source ~/.local/bin/claude-project-migrator' >> ~/.bashrc  # or ~/.zshrc
```

## Configuration

The tool can be configured via environment variables:

```bash
export CLAUDE_DIR="$HOME/.claude"                    # Claude directory
export CLAUDE_BACKUP_STRATEGY="file"                 # file or project
export CLAUDE_INTERACTIVE="true"                     # true or false
export CLAUDE_DRY_RUN="false"                       # true or false
```

Or create a config file at `~/.claude-project-migrator.conf`:

```bash
# Claude Project Migrator Configuration
export CLAUDE_DIR="$HOME/.claude"
export CLAUDE_BACKUP_STRATEGY="project"
export CLAUDE_INTERACTIVE="true"
export CLAUDE_DRY_RUN="false"
```

## Usage

### Basic Commands

```bash
# Diagnose session health (detects corruption patterns)
cm diagnose [session-uuid]

# Migrate session paths (interactive)
cm migrate

# Move sessions between projects
cm move

# Full migration (paths + move)
cm full

# List projects and sessions
cm list

# Verify project path consistency
cm verify <project-dir>

# System health check
cm health
```

### Health Diagnostics

The `diagnose` command detects session corruption patterns and provides a health assessment:

```bash
# Diagnose current active session
cm diagnose

# Diagnose specific session by UUID
cm diagnose 12345678-1234-1234-1234-123456789abc

# Get JSON output for automation
DIAGNOSE_JSON=true cm diagnose <session-uuid>
```

**Detected Corruption Patterns:**

1. **Branch Collision**: Session exists in multiple project directories
2. **Migration Race**: Multiple copies or backup files indicate interrupted migration
3. **Cross-System Inconsistency**: Session UUID present in some systems but not others
4. **Path Mismatch**: Project directory name doesn't match paths in session files
5. **Orphaned Todos**: Todo files exist without corresponding project
6. **Timestamp Drift**: Large time difference between project and todo files (>1 hour)

**Health Scores:**
- **100-90**: HEALTHY ✅ - No issues detected
- **89-70**: MINOR_ISSUES ⚠ - Minor problems that should be monitored
- **69-50**: DEGRADED ⚠ - Multiple issues affecting functionality
- **49-30**: CORRUPTED ❌ - Significant corruption requiring repair
- **29-0**: CRITICAL ❌ - Severe corruption requiring manual intervention

**Example Output:**

```bash
$ cm diagnose current

=== Session Health Diagnosis ===
Session UUID: 12345678-1234-1234-1234-123456789abc

Scanning for corruption patterns...
  ✓ No branch collision
  ✓ No migration race
  ✗ Cross-system issues: orphaned todos (3 files)
  ✓ Path consistency OK
  ✗ Orphaned todos: 3 orphaned todo files
  ✓ Timestamps consistent

Running validation checks...
  ✓ Process safety OK
  ⚠ Cross-system state: no todos
  ✓ Path consistency OK

=== Health Assessment ===
Health Score: 60/100 - DEGRADED ⚠
Session has multiple issues that may affect functionality
Recommendation: Consider running recovery procedures
```

### Advanced Usage

```bash
# Non-interactive path migration
cm migrate "/old/path" "/new/path" "/project/dir"

# Move sessions with explicit paths
cm move "/old/project" "/new/project"

# Full migration with all parameters
cm full "/old/path" "/new/path" "/old/project" "/new/project"

# Use project backup strategy
CLAUDE_BACKUP_STRATEGY=project cm migrate "/old" "/new"

# Dry run to preview changes
CLAUDE_DRY_RUN=true cm migrate "/old/path" "/new/path"
```

### Command Reference

| Command | Aliases | Description |
|---------|---------|-------------|
| `diagnose` | `diag` | Analyze session health and detect corruption |
| `migrate` | `m`, `cm-migrate` | Update session paths |
| `move` | `mv`, `cm-move` | Move sessions between projects |
| `full` | `f`, `cm-full` | Complete migration (paths + move) |
| `list` | `ls`, `l`, `cm-list` | List projects or sessions |
| `verify` | `v` | Check project path consistency |
| `health` | `doctor` | System health check |
| `config` | `cfg` | Show current configuration |
| `help` | `h` | Show help information |

Aliases:
  cm, cm-migrate, cm-move, cm-full, cm-list

## Backup Strategies

### File-Level Backup (`file`)
- Creates `.bak` files for each modified session
- Allows granular rollback
- Minimal disk usage
- Good for small changes

### Project-Level Backup (`project`)
- Creates `.tar.gz` backup of entire project directory
- Complete snapshot with timestamp
- Easy full restoration
- Good for major migrations

## Workflow Examples

### Scenario 1: Project Renamed
```bash
# You renamed /Users/tryk/dev/my-project to /Users/tryk/dev/my-awesome-project
cm migrate "/Users/tryk/dev/my-project" "/Users/tryk/dev/my-awesome-project"
```

### Scenario 2: Project Moved
```bash
# You moved /Users/tryk/dev/project to /Users/tryk/NabiaTech/project
cm full "/Users/tryk/dev/project" "/Users/tryk/NabiaTech/project" \
         "/old/project/id" "/new/project/id"
```

### Scenario 3: Organization Restructure
```bash
# Multiple projects moved to new structure
CLAUDE_BACKUP_STRATEGY=project cm migrate "/old/base/path" "/new/base/path"
```

## Safety Features

- **Automatic Backups**: Every operation creates backups
- **Dry Run Mode**: Preview changes without applying them
- **Interactive Confirmations**: Review changes before applying
- **Detailed Logging**: See exactly what's happening
- **Rollback Support**: Easy restoration from backups

## Troubleshooting

### No sessions found
```bash
# Check if Claude directory exists
cm list

# Verify Claude has been used
ls ~/.claude/projects/
```

### Permission issues
```bash
# Fix permissions
chmod -R 755 ~/.claude/
```

### Sessions not updating
```bash
# Check for syntax errors in sessions
cm config
CLAUDE_DRY_RUN=true cm migrate "old" "new"
```

### Backup restoration
```bash
# Restore from .bak files
find ~/.claude -name "*.bak" -exec sh -c 'mv "$1" "${1%.bak}"' _ {} \;

# Restore from project backup
cd ~/.claude/projects/
tar -xzf project_backup_20240115_143022.tar.gz
```

## Contributing

Feel free to submit issues and pull requests to improve the tool.

## License

This project is open source and available under the Apache 2.0 License.