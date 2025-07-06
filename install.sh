#!/usr/bin/env bash

# Claude Project Migrator Installation Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="$HOME/.local/bin"
CLAUDE_DIR="$HOME/.claude"

echo "Installing Claude Project Migrator..."

# Create necessary directories
mkdir -p "$INSTALL_DIR"

# Copy the main script
cp "$SCRIPT_DIR/claude-project-migrator.sh" "$INSTALL_DIR/claude-project-migrator"
chmod +x "$INSTALL_DIR/claude-project-migrator"

# Detect shell
SHELL_RC=""
if [[ "$SHELL" == *"zsh"* ]] || [[ -n "$ZSH_VERSION" ]]; then
    SHELL_RC="$HOME/.zshrc"
elif [[ "$SHELL" == *"bash"* ]] || [[ -n "$BASH_VERSION" ]]; then
    SHELL_RC="$HOME/.bashrc"
else
    echo "Warning: Could not detect shell type. Please manually source the script."
fi

# Add to shell rc if detected
if [[ -n "$SHELL_RC" && -f "$SHELL_RC" ]]; then
    echo ""
    echo "Would you like to add Claude Project Migrator to your $SHELL_RC? (y/n)"
    read -r response
    
    if [[ "$response" =~ ^[Yy]$ ]]; then
        # Check if already added
        if ! grep -q "claude-project-migrator" "$SHELL_RC"; then
            echo "" >> "$SHELL_RC"
            echo "# Claude Project Migrator" >> "$SHELL_RC"
            echo "source \"$INSTALL_DIR/claude-project-migrator\"" >> "$SHELL_RC"
            echo "Added to $SHELL_RC"
        else
            echo "Already exists in $SHELL_RC"
        fi
    fi
fi

# Configuration setup
echo ""
echo "Configuration setup:"
echo "Would you like to set up configuration? (y/n)"
read -r config_response

if [[ "$config_response" =~ ^[Yy]$ ]]; then
    CONFIG_FILE="$HOME/.claude-project-migrator.conf"
    
    # Claude directory
    read -p "Claude directory (default: $CLAUDE_DIR): " user_claude_dir
    claude_dir="${user_claude_dir:-$CLAUDE_DIR}"
    
    # Backup strategy
    echo "Backup strategy:"
    echo "  1. file - Create .bak files for each session"
    echo "  2. project - Create .tar.gz backup of entire project"
    read -p "Choose backup strategy (1-2, default: 1): " backup_choice
    
    case "$backup_choice" in
        "2") backup_strategy="project" ;;
        *) backup_strategy="file" ;;
    esac
    
    # Interactive mode
    read -p "Enable interactive mode by default? (Y/n): " interactive_choice
    case "$interactive_choice" in
        [Nn]*) interactive_mode="false" ;;
        *) interactive_mode="true" ;;
    esac
    
    # Write configuration
    cat > "$CONFIG_FILE" << EOF
# Claude Project Migrator Configuration
export CLAUDE_DIR="$claude_dir"
export CLAUDE_BACKUP_STRATEGY="$backup_strategy"
export CLAUDE_INTERACTIVE="$interactive_mode"
export CLAUDE_DRY_RUN="false"
EOF
    
    echo "Configuration saved to: $CONFIG_FILE"
    
    # Add config to shell rc
    if [[ -n "$SHELL_RC" ]]; then
        if ! grep -q "claude-project-migrator.conf" "$SHELL_RC"; then
            echo "source \"$CONFIG_FILE\"" >> "$SHELL_RC"
            echo "Configuration added to $SHELL_RC"
        fi
    fi
fi

echo ""
echo "Installation complete!"
echo ""
echo "To use immediately in this session, run:"
echo "  source $INSTALL_DIR/claude-project-migrator"
echo ""
echo "Usage examples:"
echo "  cpm migrate \"/old/path\" \"/new/path\""
echo "  cpm move \"/old/project\" \"/new/project\""
echo "  cpm full \"/old/path\" \"/new/path\" \"/old/project\" \"/new/project\""
echo ""

# Check Claude directory
if [[ -d "$CLAUDE_DIR" ]]; then
    project_count=$(find "$CLAUDE_DIR/projects" -type d -mindepth 1 -maxdepth 1 2>/dev/null | wc -l)
    echo "Found Claude directory with $project_count projects"
else
    echo "⚠️  Claude directory not found at: $CLAUDE_DIR"
    echo "Make sure Claude is installed and has been used at least once."
fi
