#!/usr/bin/env bash

# Claude Manager
# Manage Claude projects and update session paths

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
    # Use printf for better compatibility
    printf "%s (y/N): " "$prompt"
    read -n 1 -r reply
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

# Find session files in a project
_find_project_sessions() {
    local project_dir="$1"
    if [[ ! -d "$project_dir" ]]; then
        _log_error "Project directory not found: $project_dir"
        return 1
    fi

    find "$project_dir" -name "*.jsonl" -type f | sort
}

# Extract working directory path from session file
_extract_session_path() {
    local session_file="$1"
    if [[ ! -f "$session_file" ]]; then
        return 1
    fi

    # Look for cwd patterns in the jsonl file
    grep -m 1 '"cwd":' "$session_file" 2>/dev/null | \
        sed 's/.*"cwd":"\([^"]*\)".*/\1/' || return 1
}

# Backup a file
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

# Backup a project directory
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

# Migrate session paths
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
            local changes=$(grep -c "\"cwd\":\"$old_path\"" "$session" 2>/dev/null || echo 0)
            echo "  $(basename "$session"): $changes occurrences"
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
            ((total_changes += changes))
            ((success_count++))
            continue
        fi

        # Use sed to replace the old path with new path (macOS compatible)
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s|\"cwd\":\"$old_path\"|\"cwd\":\"$new_path\"|g" "$session"
        else
            sed -i "s|\"cwd\":\"$old_path\"|\"cwd\":\"$new_path\"|g" "$session"
        fi
        local changes=$?

        if [[ $changes -eq 0 ]]; then
            local occurrence_count=$(grep -c "\"cwd\":\"$new_path\"" "$session" 2>/dev/null || echo 0)
            ((total_changes += occurrence_count))
            ((success_count++))
            _log_success "Updated $(basename "$session"): $occurrence_count changes"
        else
            _log_error "Failed to update: $(basename "$session")"
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
    printf "Select project (1-%d): " "${#projects[@]}"
    read -r selection

    if [[ "$selection" =~ ^[0-9]+$ ]] && [[ "$selection" -ge 1 ]] && [[ "$selection" -le "${#projects[@]}" ]]; then
        echo "${projects[$((selection - 1))]}"
    else
        _log_error "Invalid selection: $selection"
        return 1
    fi
}

# Auto-detect migration path
_auto_detect_migration() {
    local project_dir="$1"
    local sessions=()
    local paths=()

    # Find all unique paths in sessions
    while IFS= read -r session; do
        local path=$(_extract_session_path "$session")
        if [[ -n "$path" ]] && [[ ! " ${paths[*]} " =~ " $path " ]]; then
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
    printf "Select source path (1-%d): " "${#paths[@]}"
    read -r selection

    if [[ "$selection" =~ ^[0-9]+$ ]] && [[ "$selection" -ge 1 ]] && [[ "$selection" -le "${#paths[@]}" ]]; then
        echo "${paths[$((selection - 1))]}"
    else
        _log_error "Invalid selection: $selection"
        return 1
    fi
}

# Move sessions between projects
_move_sessions() {
    local from_project="$1"
    local to_project="$2"

    if [[ ! -d "$to_project" ]]; then
        _log_info "Creating destination project directory: $to_project"
        if [[ "$DRY_RUN" == "true" ]]; then
            _log_debug "DRY RUN: Would create directory $to_project"
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
            _log_info "Moved: $(basename "$session") -> $to_project/"
        fi
    done

    _log_success "Move completed: ${#sessions[@]} sessions moved"
}

# Move source code directory from old_path to new_path
_move_src_dir() {
    local old_path="$1"
    local new_path="$2"

    if [[ -z "$old_path" || -z "$new_path" ]]; then
        _log_error "Source move requires <old_path> and <new_path>"
        return 1
    fi

    if [[ ! -d "$old_path" ]]; then
        _log_error "Source path not found or not a directory: $old_path"
        return 1
    fi

    local new_parent
    new_parent="$(dirname "$new_path")"
    if [[ ! -d "$new_parent" ]]; then
        if [[ "$DRY_RUN" == "true" ]]; then
            _log_debug "DRY RUN: Would create parent directory $new_parent"
        else
            mkdir -p "$new_parent"
        fi
    fi

    if [[ -e "$new_path" ]]; then
        _log_warn "Destination already exists: $new_path"
        if [[ "$INTERACTIVE" == "true" ]]; then
            if ! _confirm "Proceed and replace/merge destination?"; then
                _log_warn "Source directory move cancelled"
                return 1
            fi
        fi
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        _log_debug "DRY RUN: Would move $old_path to $new_path"
        return 0
    fi

    mv "$old_path" "$new_path"
    _log_success "Moved source directory: $old_path -> $new_path"
}

# Move entire Claude project directory to an explicit destination
_move_project_dir() {
    local from_project="$1"
    local to_project="$2"

    if [[ -z "$from_project" || -z "$to_project" ]]; then
        _log_error "Project move requires <from_project> and <to_project>"
        return 1
    fi

    if [[ ! -d "$from_project" ]]; then
        _log_error "Project directory not found: $from_project"
        return 1
    fi

    local dest_parent
    dest_parent="$(dirname "$to_project")"
    if [[ ! -d "$dest_parent" ]]; then
        if [[ "$DRY_RUN" == "true" ]]; then
            _log_debug "DRY RUN: Would create parent directory $dest_parent"
        else
            mkdir -p "$dest_parent"
        fi
    fi

    if [[ -e "$to_project" ]]; then
        _log_warn "Destination project already exists: $to_project"
        if [[ "$INTERACTIVE" == "true" ]]; then
            if ! _confirm "Proceed and replace/merge destination project?"; then
                _log_warn "Project directory move cancelled"
                return 1
            fi
        fi
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        _log_debug "DRY RUN: Would move $from_project to $to_project"
        return 0
    fi

    mv "$from_project" "$to_project"
    _log_success "Moved project directory: $from_project -> $to_project"
}

# Sync project directory name with new path (auto-rename if safe)
_sync_project_dir_name_with_path() {
    local project_dir="$1"
    local new_path="$2"

    if [[ -z "$project_dir" || -z "$new_path" ]]; then
        return 0
    fi

    local parent_dir
    parent_dir="$(dirname "$project_dir")"
    local target_dir="$parent_dir/$(basename "$new_path")"

    if [[ "$project_dir" == "$target_dir" ]]; then
        return 0
    fi

    if [[ -e "$target_dir" ]]; then
        _log_warn "Target project directory already exists: $target_dir. Skipping rename."
        return 0
    fi

    if [[ "$INTERACTIVE" == "true" ]]; then
        if ! _confirm "Rename project directory to match new path? $(basename "$project_dir") -> $(basename "$target_dir")"; then
            _log_warn "Project directory rename skipped"
            return 0
        fi
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        _log_debug "DRY RUN: Would rename $project_dir to $target_dir"
        return 0
    fi

    mv "$project_dir" "$target_dir"
    _log_success "Renamed project directory: $project_dir -> $target_dir"
}

# Resolve an input path to absolute, allowing relative inputs based on a base directory
_resolve_absolute_path() {
    local base_dir="$1"
    local input_path="$2"

    if [[ -z "$input_path" ]]; then
        echo ""
        return 0
    fi

    # Absolute path
    if [[ "$input_path" == /* ]]; then
        echo "$input_path"
        return 0
    fi

    # Relative path resolution using python for portability
    if command -v python3 >/dev/null 2>&1; then
        python3 - "$base_dir" "$input_path" <<'PY'
import os, sys
base, rel = sys.argv[1], sys.argv[2]
print(os.path.abspath(os.path.join(base, rel)))
PY
        return 0
    fi

    # Fallback: best-effort join without canonicalization
    echo "$base_dir/$input_path"
}

# Find Claude project directories that contain sessions referencing a given source path
_find_projects_by_session_path() {
    local source_path="$1"
    local matches=()

    while IFS= read -r project; do
        if find "$project" -name "*.jsonl" -type f -exec grep -q "\"cwd\":\"$source_path\"" {} \; 2>/dev/null; then
            matches+=("$project")
        fi
    done < <(_find_claude_projects)

    for m in "${matches[@]}"; do
        echo "$m"
    done
}

# Suggest a Claude project directory path for a given new source path
_suggest_project_dir_for() {
    local new_path="$1"
    echo "$CLAUDE_DIR/projects/$(basename "$new_path")"
}

# Main CLI function
claude_manager() {
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
                        _log_info "Select project to migrate:"
                        project_dir=$(_select_project)
                        [[ $? -ne 0 ]] && return 1
                    fi

                    if [[ -z "$old_path" ]]; then
                        old_path=$(_auto_detect_migration "$project_dir")
                        [[ $? -ne 0 ]] && return 1
                    fi

                    if [[ -z "$new_path" ]]; then
                        printf "Enter new path: "
                        read -r new_path
                    fi
                else
                    echo "Usage: claude_manager migrate <old_path> <new_path> [project_dir]"
                    return 1
                fi
            fi

            _migrate_project "$old_path" "$new_path" "$project_dir"
            # After migrating paths in sessions, rename the Claude project dir to
            # match the new path's basename if safe/applicable.
            _sync_project_dir_name_with_path "$project_dir" "$new_path"
            ;;

        "move"|"mv")
            # Move both source code directory and Claude project, and update session paths
            local old_path="$2"
            local new_path="$3"
            local from_project="$4"
            local to_project="$5"

            if [[ -z "$old_path" || -z "$new_path" ]]; then
                if [[ "$INTERACTIVE" == "true" ]]; then
                    if [[ -z "$from_project" ]]; then
                        _log_info "Select source project:"
                        from_project=$(_select_project)
                        [[ $? -ne 0 ]] && return 1
                    fi
                    if [[ -z "$old_path" ]]; then
                        old_path=$(_auto_detect_migration "$from_project")
                        [[ $? -ne 0 ]] && return 1
                    fi
                    if [[ -z "$new_path" ]]; then
                        printf "Enter new source path: "
                        read -r new_path
                    fi
                    if [[ -z "$to_project" ]]; then
                        printf "Enter destination project path: "
                        read -r to_project
                    fi
                else
                    echo "Usage: claude_manager move <old_path> <new_path> <from_project> <to_project>"
                    return 1
                fi
            fi

            _log_info "=== Moving source and project ==="
            _move_src_dir "$old_path" "$new_path"
            _migrate_project "$old_path" "$new_path" "$from_project"
            _move_project_dir "$from_project" "$to_project"
            ;;

        "full"|"f")
            # Interactive helper: run from within the source directory you want to move
            # Allows relative or absolute destination, auto-detects Claude project
            local cwd
            cwd="$(pwd)"
            local old_path="$2"
            local new_path_input="$3"
            local new_path
            local project_candidates=()
            local selected_project=""
            local to_project=""

            _log_info "=== Interactive Full Move ==="
            _log_info "Current directory: $cwd"

            # Determine old_path
            if [[ -z "$old_path" ]]; then
                if [[ -d "$cwd" ]]; then
                    old_path="$cwd"
                else
                    _log_error "Unable to determine source directory."
                    return 1
                fi
            fi

            # Determine new_path (allow relative)
            if [[ -z "$new_path_input" ]]; then
                read -p "Enter new destination path (relative or absolute): " -r new_path_input
            fi
            new_path=$(_resolve_absolute_path "$cwd" "$new_path_input")

            # Auto-detect Claude project(s) referencing this old_path
            while IFS= read -r p; do
                project_candidates+=("$p")
            done < <(_find_projects_by_session_path "$old_path")

            if [[ ${#project_candidates[@]} -eq 0 ]]; then
                _log_error "No Claude projects reference: $old_path"
                return 1
            elif [[ ${#project_candidates[@]} -eq 1 ]]; \
                 then selected_project="${project_candidates[0]}"
                 _log_info "Detected project: $(basename "$selected_project")"
            else
                _log_info "Multiple projects reference this path:"
                for i in "${!project_candidates[@]}"; do
                    echo "  $((i + 1)). $(basename "${project_candidates[$i]}")"
                done
                printf "Select project (1-%d): " "${#project_candidates[@]}"
                read -r selection
                if [[ "$selection" =~ ^[0-9]+$ ]] && [[ "$selection" -ge 1 ]] && [[ "$selection" -le "${#project_candidates[@]}" ]]; then
                    selected_project="${project_candidates[$((selection - 1))]}"
                else
                    _log_error "Invalid selection: $selection"
                    return 1
                fi
            fi

            # Suggest destination Claude project directory
            local suggested
            suggested=$(_suggest_project_dir_for "$new_path")
            printf "Enter destination project directory [%s]: " "$suggested"
            read -r to_project
            if [[ -z "$to_project" ]]; then
                to_project="$suggested"
            fi

            _log_info "=== Plan ==="
            echo "  Source dir: $old_path"
            echo "  New dir:    $new_path"
            echo "  Project:    $selected_project"
            echo "  New proj:   $to_project"

            if ! _confirm "Proceed with full move?"; then
                _log_warn "Cancelled"
                return 1
            fi

            _move_src_dir "$old_path" "$new_path"
            _migrate_project "$old_path" "$new_path" "$selected_project"
            _move_project_dir "$selected_project" "$to_project"
            ;;

        "list"|"ls"|"l")
            local project_dir="$2"
            if [[ -z "$project_dir" ]]; then
                _log_info "Claude projects:"
                while IFS= read -r project; do
                    local session_count=$(find "$project" -name "*.jsonl" -type f | wc -l)
                    echo "  $(basename "$project") ($session_count sessions)"
                done < <(_find_claude_projects)
            else
                _log_info "Sessions in $(basename "$project_dir"):"
                while IFS= read -r session; do
                    local path=$(_extract_session_path "$session")
                    echo "  $(basename "$session") - $path"
                done < <(_find_project_sessions "$project_dir")
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
Claude Manager - Usage:

Commands:
  migrate <old_path> <new_path> [project_dir]
    Purpose: After you have already moved/renamed your source directory,
             update .jsonl session paths and optionally rename the Claude project
             directory to match the new path's basename (safe, non-destructive).
    Parameters:
      - old_path: Absolute filesystem path to replace (e.g. "/Users/oldname/project")
      - new_path: Absolute filesystem path to use instead (e.g. "/Users/newname/project")
      - project_dir: [Optional] Full path to Claude project directory containing
                     sessions that reference old_path (if omitted, you will be prompted)
    Notes:
      - No session IDs are required; this operates on all .jsonl in the project.
      - If the destination Claude project dir doesn't exist, no creation is needed here; 
        this command only updates session paths and may rename the existing project dir.

  move <old_path> <new_path> <from_project_dir> <to_project_dir>
    Purpose: Move BOTH your source code directory and the associated Claude project,
             updating all .jsonl session paths to the new source path.
    Parameters:
      - old_path: Absolute path to the existing source directory
      - new_path: Absolute path where the source directory should be moved
      - from_project_dir: Full path to the current Claude project directory
      - to_project_dir: Full path to the new Claude project directory location
    Notes:
      - This will: move source dir -> update sessions -> move Claude project dir.
      - Use this when the source directory has NOT yet been moved.

  full [new_path]
    Purpose: Interactive helper intended to be run FROM the source directory you
             want to move. Auto-detects the referring Claude project, accepts a
             relative or absolute destination path, updates sessions, and moves
             both the source and project directories.
    Parameters:
      - new_path: [Optional] Destination path; if omitted, you will be prompted and
                  may enter a relative path which will be resolved against CWD.

  list [project_dir]
    Purpose: List all projects or sessions within a specific project
    Parameters:
      - project_dir: [Optional] Full path to Claude project directory
                    (if omitted, will list all available projects)

  config
    Purpose: Display current configuration values for the Claude Manager

Aliases:
  cm, cm-migrate, cm-move, cm-full, cm-list

Configuration (Environment Variables):
  CLAUDE_DIR           - Claude directory (default: ~/.claude)
  CLAUDE_BACKUP_STRATEGY - Backup strategy: file|project (default: file)
                         file: backup each modified session file individually
                         project: create a tarball of the entire project before changes
  CLAUDE_INTERACTIVE   - Interactive mode: true|false (default: true)
                         true: prompts for missing parameters and confirmations
                         false: requires all parameters or exits with error
  CLAUDE_DRY_RUN       - Dry run mode: true|false (default: false)
                         true: shows what would happen without making changes

Examples:
  cm migrate "/Users/oldname/project" "/Users/newname/project"
  cm move "/Users/name/.claude/projects/old-project" "/Users/name/.claude/projects/new-project"
  cm full "/Users/old/path" "/Users/new/path" "/Users/name/.claude/projects/old" "/Users/name/.claude/projects/new"
  CLAUDE_BACKUP_STRATEGY=project cm migrate "/old" "/new"
EOF
            ;;

        *)
            echo "Unknown command: $cmd. Use 'claude_manager help' for usage."
            return 1
            ;;
    esac
}

# Convenient aliases
alias cm='claude_manager'
alias cm-migrate='claude_manager migrate'
alias cm-move='claude_manager move'
alias cm-full='claude_manager full'
alias cm-list='claude_manager list'
