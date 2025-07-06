# Claude Project Migrator

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
# Migrate session paths (interactive)
cpm migrate

# Move sessions between projects
cpm move

# Full migration (paths + move)
cpm full

# List projects and sessions
cpm list
```

### Advanced Usage

```bash
# Non-interactive path migration
cpm migrate "/old/path" "/new/path" "/project/dir"

# Move sessions with explicit paths
cpm move "/old/project" "/new/project"

# Full migration with all parameters
cpm full "/old/path" "/new/path" "/old/project" "/new/project"

# Use project backup strategy
CLAUDE_BACKUP_STRATEGY=project cpm migrate "/old" "/new"

# Dry run to preview changes
CLAUDE_DRY_RUN=true cpm migrate "/old/path" "/new/path"
```

### Command Reference

| Command | Aliases | Description |
|---------|---------|-------------|
| `migrate` | `m` | Update session paths |
| `move` | `mv` | Move sessions between projects |
| `full` | `f` | Complete migration (paths + move) |
| `list` | `ls`, `l` | List projects or sessions |
| `config` | `cfg` | Show current configuration |
| `help` | `h` | Show help information |

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
cpm migrate "/Users/tryk/dev/my-project" "/Users/tryk/dev/my-awesome-project"
```

### Scenario 2: Project Moved
```bash
# You moved /Users/tryk/dev/project to /Users/tryk/NabiaTech/project
cpm full "/Users/tryk/dev/project" "/Users/tryk/NabiaTech/project" \
         "/old/project/id" "/new/project/id"
```

### Scenario 3: Organization Restructure
```bash
# Multiple projects moved to new structure
CLAUDE_BACKUP_STRATEGY=project cpm migrate "/old/base/path" "/new/base/path"
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
cpm list

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
cpm config
CLAUDE_DRY_RUN=true cpm migrate "old" "new"
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

This project is open source and available under the MIT License.