# Claude Manager

**Manage your Claude Code projects and session paths when you move or reorganize your development directories.**

## The Problem

When using Claude Code, your conversations reference specific working directories. These paths are embedded in `.jsonl` session files stored in `~/.claude/projects/`. When you move or rename your project directory:

1. Session paths become invalid
2. The `/resume` function breaks
3. Claude can't find your original working directories

## The Solution

**claude-manager** automates the complete migration workflow:
- ✅ Backup your sessions (file-level or project-level)
- ✅ Find and replace all path references in session files
- ✅ Move sessions to new project directories
- ✅ Atomic operations with undo support

## Features

- 🔄 **Complete Migration**: Handles paths and project moves in one command
- 🛡️ **Safety First**: Automatic backups with rollback support
- 🎯 **Interactive Mode**: Guided workflow with confirmations
- 🤖 **Automation Ready**: Non-interactive mode for scripting
- ⚡ **Dry Run Mode**: Preview changes before applying
- 🐍 **Python 3 Integration**: Safe JSON path replacement

## Quick Start

```bash
# Install
git clone https://github.com/yourusername/claude-manager.git
cd claude-manager
./install.sh

# Use
cm migrate "/old/path" "/new/path"
cm move "/old/src" "/new/src"
cm list
```

## Requirements

**Core Dependencies:**
- Bash 4.4+
- Standard Unix tools: `sed`, `grep`, `find`, `mv`, `cp`, `tar`
- Python 3.x (recommended for safe JSON handling)

**Platforms:**
- ✅ Linux (tested on Ubuntu, Debian, WSL)
- ✅ macOS (with GNU coreutils recommended)
- ⚠️  Windows (via WSL or Git Bash)

## Installation

See [Installation Guide](docs/installation/INSTALLATION.md) for detailed instructions.

```bash
# Quick install
./install.sh

# Manual install
cp src/claude-manager.sh ~/.local/bin/claude-manager
chmod +x ~/.local/bin/claude-manager
echo 'source ~/.local/bin/claude-manager' >> ~/.bashrc
```

## Usage

See [Usage Guide](docs/usage/USAGE.md) for comprehensive examples.

### Basic Commands

```bash
# Migrate session paths (interactive)
cm migrate

# Move directory and update sessions
cm move "/old/path" "/new/path"

# List projects and sessions
cm list

# Show configuration
cm config

# Undo last operation
cm undo

# Help
cm help
```

### Environment Variables

```bash
export CLAUDE_DIR="$HOME/.claude"           # Claude directory
export CLAUDE_BACKUP_STRATEGY="file"        # file or project
export CLAUDE_INTERACTIVE="true"            # true or false
export CLAUDE_DRY_RUN="false"              # Preview mode
export CLAUDE_DEBUG="0"                     # Debug logging
```

## Integration with Riff CLI

**claude-manager** pairs perfectly with [riff-cli](https://github.com/yourusername/riff-cli) for monitoring and managing Claude sessions.

See [Pairing Guide](docs/integration/PAIRING.md) for setup instructions.

## Optional: NabiÓS Federation

For advanced users running multi-agent federations, claude-manager can integrate with the NabiÓS platform for coordinated session management across distributed systems.

See [NabiÓS Integration](docs/integration/NABIOS.md) for details.

## Documentation

- [Installation Guide](docs/installation/INSTALLATION.md) - Setup and configuration
- [Usage Guide](docs/usage/USAGE.md) - Commands and examples
- [Pairing with riff-cli](docs/integration/PAIRING.md) - Session monitoring integration
- [NabiÓS Integration](docs/integration/NABIOS.md) - Federation features (optional)

## Examples

### Scenario 1: Renamed Project

```bash
# You renamed: /Users/tryk/dev/my-project → /Users/tryk/dev/my-awesome-project
cm migrate "/Users/tryk/dev/my-project" "/Users/tryk/dev/my-awesome-project"
```

### Scenario 2: Moved Project

```bash
# You moved: /Users/tryk/dev/project → /Users/tryk/projects/project
cm move "/Users/tryk/dev/project" "/Users/tryk/projects/project"
```

### Scenario 3: Dry Run Preview

```bash
# Preview changes without applying
CLAUDE_DRY_RUN=true cm move "/old/path" "/new/path"
```

## Safety Features

- ✅ Automatic backups before every operation
- ✅ Atomic moves with undo support
- ✅ Interactive confirmations for risky operations
- ✅ Dry run mode to preview changes
- ✅ Python-based JSON path replacement (whitespace tolerant)

## Troubleshooting

### No sessions found
```bash
# Check Claude directory
cm list

# Verify Claude has been used
ls ~/.claude/projects/
```

### Permission issues
```bash
# Fix permissions
chmod -R 755 ~/.claude/
```

### Restore from backup
```bash
# Restore .bak files
find ~/.claude -name "*.bak" -exec sh -c 'mv "$1" "${1%.bak}"' _ {} \;

# Restore from project backup
cd ~/.claude/projects/
tar -xzf project_backup_20240115_143022.tar.gz
```

## Contributing

Contributions welcome! Please submit issues and pull requests.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Links

- [GitHub Repository](https://github.com/yourusername/claude-manager)
- [Issue Tracker](https://github.com/yourusername/claude-manager/issues)
- [Riff CLI](https://github.com/yourusername/riff-cli) - Session monitoring companion

---

**Made with ❤️ by the Claude Code community**
