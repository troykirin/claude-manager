#!/usr/bin/env bash

# Claude Manager - Installation Script
# Standalone installation with minimal dependencies

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
CLAUDE_DIR="${CLAUDE_DIR:-$HOME/.claude}"
SCRIPT_NAME="claude-manager"

echo "=== Claude Manager Installation ==="
echo ""

# Check Bash version
bash_version="${BASH_VERSION%%[^0-9.]*}"
bash_major="${bash_version%%.*}"
bash_minor="${bash_version#*.}"
bash_minor="${bash_minor%%.*}"

if [[ "$bash_major" -lt 4 ]] || [[ "$bash_major" -eq 4 && "$bash_minor" -lt 4 ]]; then
    echo "❌ Error: Bash 4.4+ required (found: $BASH_VERSION)"
    echo "   Please upgrade Bash and try again."
    exit 1
fi

echo "✓ Bash $BASH_VERSION detected"

# Check for required tools
required_tools=("sed" "grep" "find" "mv" "cp" "tar")
missing_tools=()

for tool in "${required_tools[@]}"; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        missing_tools+=("$tool")
    fi
done

if [[ ${#missing_tools[@]} -gt 0 ]]; then
    echo "❌ Error: Missing required tools: ${missing_tools[*]}"
    echo "   Please install these tools and try again."
    exit 1
fi

echo "✓ Required tools available"

# Check for python3 (highly recommended)
if command -v python3 >/dev/null 2>&1; then
    echo "✓ Python 3 available (recommended for JSON handling)"
else
    echo "⚠️  Warning: Python 3 not found"
    echo "   Python 3 is recommended for safe JSON path replacement"
    echo "   Some operations may fail without it."
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 1
    fi
fi

# Create installation directory
echo ""
echo "Installing to: $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

# Copy the main script
if [[ -f "$SCRIPT_DIR/src/claude-manager.sh" ]]; then
    cp "$SCRIPT_DIR/src/claude-manager.sh" "$INSTALL_DIR/$SCRIPT_NAME"
elif [[ -f "$SCRIPT_DIR/claude-manager.sh" ]]; then
    cp "$SCRIPT_DIR/claude-manager.sh" "$INSTALL_DIR/$SCRIPT_NAME"
else
    echo "❌ Error: claude-manager.sh not found"
    exit 1
fi

chmod +x "$INSTALL_DIR/$SCRIPT_NAME"
echo "✓ Installed: $INSTALL_DIR/$SCRIPT_NAME"

# Detect shell
SHELL_RC=""
SHELL_NAME=""

if [[ -n "$ZSH_VERSION" ]] || [[ "$SHELL" == *"zsh"* ]]; then
    SHELL_RC="$HOME/.zshrc"
    SHELL_NAME="zsh"
elif [[ -n "$BASH_VERSION" ]] || [[ "$SHELL" == *"bash"* ]]; then
    SHELL_RC="$HOME/.bashrc"
    SHELL_NAME="bash"
fi

# Add to shell rc if detected
if [[ -n "$SHELL_RC" && -f "$SHELL_RC" ]]; then
    echo ""
    echo "Detected shell: $SHELL_NAME"
    read -p "Add claude-manager to $SHELL_RC? (Y/n): " -r
    echo

    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        # Check if already added
        if ! grep -q "claude-manager" "$SHELL_RC"; then
            cat >> "$SHELL_RC" << EOF

# Claude Manager
source "$INSTALL_DIR/$SCRIPT_NAME"
EOF
            echo "✓ Added to $SHELL_RC"
        else
            echo "✓ Already exists in $SHELL_RC"
        fi
    fi
else
    echo ""
    echo "⚠️  Could not detect shell configuration file"
    echo "   Please manually add to your shell rc:"
    echo "   source $INSTALL_DIR/$SCRIPT_NAME"
fi

# Configuration setup
echo ""
read -p "Configure environment variables? (Y/n): " -r
echo

if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    CONFIG_FILE="$HOME/.claude-manager.conf"

    # Claude directory
    read -p "Claude directory (default: $CLAUDE_DIR): " user_claude_dir
    claude_dir="${user_claude_dir:-$CLAUDE_DIR}"

    # Backup strategy
    echo ""
    echo "Backup strategy:"
    echo "  1. file    - Create .bak files for each session (minimal disk usage)"
    echo "  2. project - Create .tar.gz backup of entire project (complete snapshot)"
    read -p "Choose backup strategy (1-2, default: 1): " backup_choice

    case "$backup_choice" in
        "2") backup_strategy="project" ;;
        *) backup_strategy="file" ;;
    esac

    # Interactive mode
    echo ""
    read -p "Enable interactive mode by default? (Y/n): " interactive_choice
    case "$interactive_choice" in
        [Nn]*) interactive_mode="false" ;;
        *) interactive_mode="true" ;;
    esac

    # Write configuration
    cat > "$CONFIG_FILE" << EOF
# Claude Manager Configuration
# Generated: $(date)

export CLAUDE_DIR="$claude_dir"
export CLAUDE_BACKUP_STRATEGY="$backup_strategy"
export CLAUDE_INTERACTIVE="$interactive_mode"
export CLAUDE_DRY_RUN="false"
export CLAUDE_DEBUG="0"
EOF

    echo ""
    echo "✓ Configuration saved to: $CONFIG_FILE"

    # Add config to shell rc
    if [[ -n "$SHELL_RC" && -f "$SHELL_RC" ]]; then
        if ! grep -q "claude-manager.conf" "$SHELL_RC"; then
            cat >> "$SHELL_RC" << EOF

# Claude Manager Configuration
[[ -f "$CONFIG_FILE" ]] && source "$CONFIG_FILE"
EOF
            echo "✓ Configuration added to $SHELL_RC"
        fi
    fi
fi

# Verify installation
echo ""
echo "=== Installation Complete ==="
echo ""

# Check if installed directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Warning: $INSTALL_DIR is not in your PATH"
    echo ""
    echo "Add to your shell rc file:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    echo ""
fi

# Provide usage instructions
echo "To use immediately in this session:"
echo "  source $INSTALL_DIR/$SCRIPT_NAME"
[[ -n "$CONFIG_FILE" ]] && echo "  source $CONFIG_FILE"
echo ""

echo "Usage examples:"
echo "  cm help                             # Show help"
echo "  cm list                             # List projects"
echo "  cm migrate \"/old/path\" \"/new/path\"  # Migrate paths"
echo "  cm move \"/old/src\" \"/new/src\"       # Move directory and sessions"
echo ""

# Check Claude directory
if [[ -d "$claude_dir" ]]; then
    if [[ -d "$claude_dir/projects" ]]; then
        project_count=$(find "$claude_dir/projects" -type d -mindepth 1 -maxdepth 1 2>/dev/null | wc -l)
        echo "✓ Found Claude directory with $project_count projects"
    else
        echo "⚠️  Claude projects directory not found: $claude_dir/projects"
    fi
else
    echo "⚠️  Claude directory not found: $claude_dir"
    echo "   Make sure Claude Code is installed and has been used at least once."
fi

echo ""
echo "For documentation, see:"
echo "  https://github.com/yourusername/claude-manager"
echo ""
