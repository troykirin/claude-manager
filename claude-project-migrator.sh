
#!/usr/bin/env bash

# Claude Project Migrator
# Migrate Claude projects and update session paths

set -e

# Configuration with defaults
export CLAUDE_DIR="$HOME/.claude"
export BACKUP_STRATEGY="${CLAUDE_BACKUP_STRATEGY:-file}"  # file or project
export INTERACTIVE="${CLAUDE_INTERACTIVE:-true}"
export DRY_RUN="${CLAUDE_DRY_RUN:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging
_log() {
    local level="$1" message="$2" color="$3"
    echo -e "${color}[$(date '+%H:%M:%S')] [$level] $message${NC}"
}

_log_info() { _log "INFO" "$1" "$BLUE"; }
_log_warn() { _log "WARN" "$1" "$YELLOW"; }
_log_error() { _log "ERROR" "$1" "$RED"; }
_log_success() { _log "SUCCESS" "$1" "$GREEN"; }
_log_debug() { _log "DEBUG" "$1" "$PURPLE"; }

# Confirmation prompts
_confirm() {
    local prompt="$1" default="${2:-n}"
    if [[ "$INTERACTIVE" == "false" ]]; then
        echo "Auto-confirming: $prompt"
        return 0
    fi
    
    local reply
    read -p "$prompt (y/N): " -n 1 -r reply
    echo
    [[ $reply =~ ^[Yy]$ ]]
}

# Find Claude project directories
_find_claude_projects() {
    if [[ ! -d "$CLAUDE_DIR/projects" ]]; then
        _log_error "Claude projects directory not found: $CLAUDE_DIR/projects"
        return 1
    fi
    
    find "$CLAUDE_DIR/projects" -type d -mindepth 1 -maxdepth 1 | sort
}

# Find sessions in a project
_find_project_sessions() {
    local project_dir="$1"
    if [[ ! -d "$project_dir" ]]; then
        _log_error "Project directory not found: $project_dir"
        return 1
    fi
    
    find "$project_dir" -name "*.jsonl" -type f | sort
}

# Extract path from session content
_extract_session_path() {
    local session_file="$1"
    if [[ ! -f "$session_file" ]]; then
        return 1
    fi
    
    # Look for cwd patterns in the jsonl file
    grep -m 1 '"cwd":' "$session_file" 2>/dev/null | \
        sed 's/.*"cwd":"\([^"]*\)".*/\1/' || return 1
}

# Backup strategies
_backup_file() {
    local file="$1"
    local backup_file="${file}.bak"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        _log_debug "DRY RUN: Would backup $file to $backup_file"
        return 0
    fi
    
    cp "$file" "$backup_file"
    _log_info "Backed up: $file -> $backup_file"
}

_backup_project() {
    local project_dir="$1"
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local backup_file="${project_dir}_backup_${timestamp}.tar.gz"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        _log_debug "DRY RUN: Would backup $project_dir to $backup_file"
        return 0
    fi
    
    tar -czf "$backup_file" -C "$(dirname "$project_dir")" "$(basename "$project_dir")"
    _log_info "Project backed up: $backup_file"
}

# Main migration function
_migrate_project() {
    local old_path="$1"
    local new_path="$2"
    local project_dir="$3"
    
    _log_info "Starting migration from '$old_path' to '$new_path'"
    
    # Find all sessions with the old path
    local sessions=()
    while IFS= read -r session; do
        if grep -q "\"cwd\":\"$old_path\"" "$session" 2>/dev/null; then
            sessions+=("$session")
        fi
    done < <(_find_project_sessions "$project_dir")
    
    if [[ ${#sessions[@]} -eq 0 ]]; then
        _log_warn "No sessions found with path: $old_path"
        return 0
    fi
    
    _log_info "Found ${#sessions[@]} sessions to migrate"
    
    # Show preview of changes
    if [[ "$INTERACTIVE" == "true" ]]; then
        _log_info "Preview of changes:"
        for session in "${sessions[@]}"; do
            echo "  📄 $(basename "$session")"
            grep -n "\"cwd\":\"$old_path\"" "$session" 2>/dev/null | head -2 | \
                sed 's/^/    /'
        done
        echo
        
        if ! _confirm "Proceed with migration?"; then
            _log_warn "Migration cancelled"
            return 1
        fi
    fi
    
    # Backup based on strategy
    case "$BACKUP_STRATEGY" in
        "project")
            _backup_project "$project_dir"
            ;;
        "file")
            for session in "${sessions[@]}"; do
                _backup_file "$session"
            done
            ;;
        *)
            _log_error "Unknown backup strategy: $BACKUP_STRATEGY"
            return 1
            ;;
    esac
    
    # Perform the migration
    local success_count=0
    local total_changes=0
    
    for session in "${sessions[@]}"; do
        _log_info "Migrating: $(basename "$session")"
        
        if [[ "$DRY_RUN" == "true" ]]; then
            local changes=$(grep -c "\"cwd\":\"$old_path\"" "$session" 2>/dev/null || echo 0)
            _log_debug "DRY RUN: Would replace $changes occurrences in $session"
            total_changes=$((total_changes + changes))
        else
            # Use sed to replace the paths
            if sed -i".tmp" "s|\"cwd\":\"$old_path\"|\"cwd\":\"$new_path\"|g" "$session"; then
                rm "${session}.tmp" 2>/dev/null || true
                local changes=$(grep -c "\"cwd\":\"$new_path\"" "$session" 2>/dev/null || echo 0)
                _log_success "✅ Replaced $changes occurrences in $(basename "$session")"
                success_count=$((success_count + 1))
                total_changes=$((total_changes + changes))
            else
                _log_error "❌ Failed to migrate: $session"
                rm "${session}.tmp" 2>/dev/null || true
            fi
        fi
    done
    
    _log_success "Migration completed: $success_count/$((${#sessions[@]})) files, $total_changes total changes"
}

# Interactive project selection
_select_project() {
    local projects=()
    while IFS= read -r project; do
        projects+=("$project")
    done < <(_find_claude_projects)
    
    if [[ ${#projects[@]} -eq 0 ]]; then
        _log_error "No Claude projects found"
        return 1
    fi
    
    _log_info "Available Claude projects:"
    for i in "${!projects[@]}"; do
        local project="${projects[$i]}"
        local session_count=$(find "$project" -name "*.jsonl" -type f | wc -l)
        echo "  $((i + 1)). $(basename "$project") ($session_count sessions)"
    done
    
    echo
    read -p "Select project (1-${#projects[@]}): " -r selection
    
    if [[ "$selection" =~ ^[0-9]+$ ]] && [[ "$selection" -ge 1 ]] && [[ "$selection" -le "${#projects[@]}" ]]; then
        echo "${projects[$((selection - 1))]}"
    else
        _log_error "Invalid selection"
        return 1
    fi
}

# Auto-detect path migration
_auto_detect_migration() {
    local project_dir="$1"
    local sessions=()
    local paths=()
    
    # Find all unique paths in sessions
    while IFS= read -r session; do
        local path=$(_extract_session_path "$session")
        if [[ -n "$path" ]] && [[ ! " ${paths[@]} " =~ " $path " ]]; then
            paths+=("$path")
            sessions+=("$session")
        fi
    done < <(_find_project_sessions "$project_dir")
    
    if [[ ${#paths[@]} -eq 0 ]]; then
        _log_error "No session paths found for auto-detection"
        return 1
    fi
    
    _log_info "Detected paths in sessions:"
    for i in "${!paths[@]}"; do
        echo "  $((i + 1)). ${paths[$i]}"
    done
    
    echo
    read -p "Select source path (1-${#paths[@]}): " -r selection
    
    if [[ "$selection" =~ ^[0-9]+$ ]] && [[ "$selection" -ge 1 ]] && [[ "$selection" -le "${#paths[@]}" ]]; then
        echo "${paths[$((selection - 1))]}"
    else
        _log_error "Invalid selection"
        return 1
    fi
}

# Move sessions to new project
_move_sessions() {
    local from_project="$1"
    local to_project="$2"
    
    if [[ ! -d "$to_project" ]]; then
        _log_info "Creating destination project directory: $to_project"
        if [[ "$DRY_RUN" == "true" ]]; then
            _log_debug "DRY RUN: Would create $to_project"
        else
            mkdir -p "$to_project"
        fi
    fi
    
    local sessions=()
    while IFS= read -r session; do
        sessions+=("$session")
    done < <(_find_project_sessions "$from_project")
    
    if [[ ${#sessions[@]} -eq 0 ]]; then
        _log_warn "No sessions to move"
        return 0
    fi
    
    _log_info "Moving ${#sessions[@]} sessions to new project"
    
    for session in "${sessions[@]}"; do
        local dest="$to_project/$(basename "$session")"
        if [[ "$DRY_RUN" == "true" ]]; then
            _log_debug "DRY RUN: Would move $session to $dest"
        else
            mv "$session" "$dest"
            _log_info "Moved: $(basename "$session")"
        fi
    done
}

# Main CLI function
claude_project_migrator() {
    local cmd="$1"
    
    case "$cmd" in
        "migrate"|"m")
            local old_path="$2"
            local new_path="$3"
            local project_dir="$4"
            
            if [[ -z "$old_path" || -z "$new_path" ]]; then
                if [[ "$INTERACTIVE" == "true" ]]; then
                    # Interactive mode
                    if [[ -z "$project_dir" ]]; then
                        project_dir=$(_select_project)
                        [[ $? -ne 0 ]] && return 1
                    fi
                    
                    if [[ -z "$old_path" ]]; then
                        old_path=$(_auto_detect_migration "$project_dir")
                        [[ $? -ne 0 ]] && return 1
                    fi
                    
                    if [[ -z "$new_path" ]]; then
                        read -p "Enter new path: " -r new_path
                    fi
                else
                    echo "Usage: claude_project_migrator migrate <old_path> <new_path> [project_dir]"
                    return 1
                fi
            fi
            
            _migrate_project "$old_path" "$new_path" "$project_dir"
            ;;
        
        "move"|"mv")
            local from_project="$2"
            local to_project="$3"
            
            if [[ -z "$from_project" || -z "$to_project" ]]; then
                if [[ "$INTERACTIVE" == "true" ]]; then
                    if [[ -z "$from_project" ]]; then
                        _log_info "Select source project:"
                        from_project=$(_select_project)
                        [[ $? -ne 0 ]] && return 1
                    fi
                    
                    if [[ -z "$to_project" ]]; then
                        read -p "Enter destination project path: " -r to_project
                    fi
                else
                    echo "Usage: claude_project_migrator move <from_project> <to_project>"
                    return 1
                fi
            fi
            
            _move_sessions "$from_project" "$to_project"
            ;;
        
        "full"|"f")
            local old_path="$2"
            local new_path="$3"
            local old_project="$4"
            local new_project="$5"
            
            if [[ "$INTERACTIVE" == "true" ]] && [[ -z "$old_path" ]]; then
                _log_info "=== Claude Project Full Migration ==="
                echo
                
                if [[ -z "$old_project" ]]; then
                    _log_info "Select source project:"
                    old_project=$(_select_project)
                    [[ $? -ne 0 ]] && return 1
                fi
                
                if [[ -z "$old_path" ]]; then
                    old_path=$(_auto_detect_migration "$old_project")
                    [[ $? -ne 0 ]] && return 1
                fi
                
                if [[ -z "$new_path" ]]; then
                    read -p "Enter new path: " -r new_path
                fi
                
                if [[ -z "$new_project" ]]; then
                    read -p "Enter new project directory: " -r new_project
                fi
            fi
            
            _log_info "=== Full Migration: $old_path -> $new_path ==="
            _migrate_project "$old_path" "$new_path" "$old_project"
            _move_sessions "$old_project" "$new_project"
            ;;
        
        "list"|"ls"|"l")
            local project_dir="$2"
            if [[ -z "$project_dir" ]]; then
                _log_info "Claude projects:"
                _find_claude_projects | while read -r project; do
                    local session_count=$(find "$project" -name "*.jsonl" -type f | wc -l)
                    echo "  📁 $(basename "$project") ($session_count sessions)"
                done
            else
                _log_info "Sessions in $project_dir:"
                _find_project_sessions "$project_dir" | while read -r session; do
                    local path=$(_extract_session_path "$session")
                    echo "  📄 $(basename "$session") -> $path"
                done
            fi
            ;;
        
        "config"|"cfg")
            _log_info "Current configuration:"
            echo "  CLAUDE_DIR: $CLAUDE_DIR"
            echo "  BACKUP_STRATEGY: $BACKUP_STRATEGY"
            echo "  INTERACTIVE: $INTERACTIVE"
            echo "  DRY_RUN: $DRY_RUN"
            ;;
        
        "help"|"h"|"")
            cat << 'EOF'
Claude Project Migrator - Usage:

Commands:
  migrate <old> <new> [project]  - Migrate session paths
  move <from> <to>               - Move sessions between projects  
  full <old> <new> <from> <to>   - Full migration (paths + move)
  list [project]                 - List projects or sessions
  config                         - Show current configuration

Aliases: 
  cpm, cpm-migrate, cpm-move, cpm-full, cpm-list

Configuration (Environment Variables):
  CLAUDE_DIR           - Claude directory (default: ~/.claude)
  CLAUDE_BACKUP_STRATEGY - Backup strategy: file|project (default: file)
  CLAUDE_INTERACTIVE   - Interactive mode: true|false (default: true)
  CLAUDE_DRY_RUN       - Dry run mode: true|false (default: false)

Examples:
  cpm migrate "/old/path" "/new/path"
  cpm move "/old/project" "/new/project"  
  cpm full "/old/path" "/new/path" "/old/project" "/new/project"
  CLAUDE_BACKUP_STRATEGY=project cpm migrate "/old" "/new"
EOF
            ;;
        
        *)
            echo "Unknown command: $cmd. Use 'claude_project_migrator help' for usage."
            return 1
            ;;
    esac
}

# Convenient aliases
alias cpm='claude_project_migrator'
alias cpm-migrate='claude_project_migrator migrate'
alias cpm-move='claude_project_migrator move'
alias cpm-full='claude_project_migrator full'
alias cpm-list='claude_project_migrator list'