# Installation Guide

## Prerequisites

### Required
- **Bash 4.4+** - Check with `bash --version`
- **Python 3.x** - Highly recommended for safe JSON handling
- **Standard Unix tools**: `sed`, `grep`, `find`, `mv`, `cp`, `tar`

### Platform-Specific

**macOS:**
```bash
# Upgrade bash if needed (macOS ships with old Bash 3.x)
brew install bash

# Optional: GNU coreutils for better compatibility
brew install coreutils
```

**Linux/WSL:**
```bash
# Most distributions already have required tools
# Verify python3
python3 --version
```

## Installation Methods

### Method 1: Automated Installation (Recommended)

```bash
# Clone repository
git clone https://github.com/yourusername/claude-manager.git
cd claude-manager

# Run installer
./install.sh
```

The installer will:
1. Check for required dependencies
2. Install `claude-manager` to `~/.local/bin/`
3. Add to your shell rc file (`.bashrc` or `.zshrc`)
4. Optionally configure environment variables

### Method 2: Manual Installation

```bash
# Copy script
cp src/claude-manager.sh ~/.local/bin/claude-manager
chmod +x ~/.local/bin/claude-manager

# Add to shell rc
echo 'source ~/.local/bin/claude-manager' >> ~/.bashrc  # or ~/.zshrc

# Add to PATH if needed
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc

# Reload shell
source ~/.bashrc
```

### Method 3: Direct Usage (No Install)

```bash
# Clone repository
git clone https://github.com/yourusername/claude-manager.git
cd claude-manager

# Use directly
bash src/claude-manager.sh help
bash src/claude-manager.sh list
```

## Configuration

### Environment Variables

Create `~/.claude-manager.conf`:

```bash
# Claude Manager Configuration
export CLAUDE_DIR="$HOME/.claude"
export CLAUDE_BACKUP_STRATEGY="file"  # file or project
export CLAUDE_INTERACTIVE="true"      # true or false
export CLAUDE_DRY_RUN="false"        # Preview mode
export CLAUDE_DEBUG="0"               # Debug logging (0 or 1)
```

Source in your shell rc:
```bash
echo 'source ~/.claude-manager.conf' >> ~/.bashrc
```

### Backup Strategies

**File-Level (`file`)** - Default
- Creates `.bak` files for each modified session
- Minimal disk usage
- Granular rollback
- Good for small changes

**Project-Level (`project`)**
- Creates timestamped `.tar.gz` of entire project
- Complete snapshot
- Easy full restoration
- Good for major migrations

## Verification

```bash
# Check installation
which claude-manager
# Should output: /home/user/.local/bin/claude-manager

# Test command
cm help

# Check Claude directory
cm list

# Verify configuration
cm config
```

## Troubleshooting

### Bash version too old

**macOS:**
```bash
brew install bash
# Add to /etc/shells
sudo echo $(which bash) >> /etc/shells
# Change default shell
chsh -s $(which bash)
```

**Linux:**
```bash
# Update bash via package manager
sudo apt update && sudo apt install bash  # Debian/Ubuntu
sudo yum update bash                       # RHEL/CentOS
```

### Python 3 not found

**macOS:**
```bash
brew install python3
```

**Linux:**
```bash
sudo apt install python3  # Debian/Ubuntu
sudo yum install python3  # RHEL/CentOS
```

### PATH not updated

Add to `~/.bashrc` or `~/.zshrc`:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

Reload:
```bash
source ~/.bashrc  # or source ~/.zshrc
```

### Claude directory not found

Ensure Claude Code has been used at least once:
```bash
ls -la ~/.claude/projects/
```

If missing, start Claude Code and create a session.

## Uninstallation

```bash
# Remove script
rm ~/.local/bin/claude-manager

# Remove configuration
rm ~/.claude-manager.conf

# Remove from shell rc
# Manually edit ~/.bashrc or ~/.zshrc to remove:
# - source ~/.local/bin/claude-manager
# - source ~/.claude-manager.conf
```

## Next Steps

- [Usage Guide](../usage/USAGE.md) - Learn the commands
- [Pairing with riff-cli](../integration/PAIRING.md) - Session monitoring
