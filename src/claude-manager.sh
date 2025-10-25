#!/usr/bin/env bash

# Claude Manager
# Manage Claude projects and update session paths
# Standalone version - no external dependencies required

set -e

# Configuration with defaults
export CLAUDE_DIR="${CLAUDE_DIR:-$HOME/.claude}"
export BACKUP_STRATEGY="${CLAUDE_BACKUP_STRATEGY:-file}"  # file or project
export INTERACTIVE="${CLAUDE_INTERACTIVE:-true}"
export DRY_RUN="${CLAUDE_DRY_RUN:-false}"
export UNDO_FILE="$CLAUDE_DIR/.last_move_operation"

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
_log_debug() {
    if [[ "${CLAUDE_DEBUG:-0}" == "1" ]]; then
        _log "DEBUG" "$1" "$PURPLE" >&2
    fi
}

# Save undo information
_save_undo_info() {
    local operation="$1"
    shift
    local timestamp
    timestamp="$(date '+%Y-%m-%d %H:%M:%S')"

    {
        printf '%s\n' "$timestamp"
        printf '%s\n' "$operation"
        printf '%s\n' "$#"
        for arg in "$@"; do
            printf '%s\n' "$arg"
        done
    } > "$UNDO_FILE"
    _log_info "Undo information saved to $UNDO_FILE"
}

# Perform undo of last operation
_undo_last_operation() {
    if [[ ! -f "$UNDO_FILE" ]]; then
        _log_error "No undo information found"
        return 1
    fi

    mapfile -t undo_lines < "$UNDO_FILE"

    if [[ ${#undo_lines[@]} -lt 3 ]]; then
        _log_error "Undo information is corrupted: insufficient data"
        return 1
    fi

    local timestamp="${undo_lines[0]}"
    local operation="${undo_lines[1]}"
    local arg_count="${undo_lines[2]}"

    if [[ ! "$arg_count" =~ ^[0-9]+$ ]]; then
        _log_error "Undo information is corrupted: invalid argument count"
        return 1
    fi

    local expected_total=$((3 + arg_count))
    if [[ ${#undo_lines[@]} -lt $expected_total ]]; then
        _log_error "Undo information is corrupted: missing arguments"
        return 1
    fi

    local args=()
    local idx
    for ((idx = 0; idx < arg_count; idx++)); do
        args+=("${undo_lines[$((3 + idx))]}")
    done

    _log_info "Last operation: $operation at $timestamp"
    if [[ ${#args[@]} -gt 0 ]]; then
        local quoted=()
        for arg in "${args[@]}"; do
            quoted+=("'${arg}'")
        done
        _log_info "Arguments: ${quoted[*]}"
    else
        _log_info "Arguments: <none>"
    fi

    case "$operation" in
        "move")
            local old_path="${args[0]:-}"
            local new_path="${args[1]:-}"
            local from_project="${args[2]:-}"
            local to_project="${args[3]:-}"
            local source_moved="${args[4]:-true}"

            if [[ -z "$source_moved" ]]; then
                source_moved="true"
            else
                source_moved=$(printf '%s' "$source_moved" | tr '[:upper:]' '[:lower:]')
            fi

            if [[ -z "$old_path" || -z "$new_path" ]]; then
                _log_error "Undo information missing required paths"
                return 1
            fi

            _log_info "Undoing move operation..."

            # Handle simple directory move (no project)
            if [[ "$from_project" == "none" ]]; then
                if [[ "$source_moved" == "false" ]]; then
                    _log_info "Original operation skipped filesystem move; nothing to restore on disk"
                elif [[ -d "$new_path" ]]; then
                    mv "$new_path" "$old_path"
                    _log_success "Restored directory: $new_path -> $old_path"
                else
                    _log_warn "Directory not found: $new_path"
                fi
            else
                # Reverse the operations in opposite order
                if [[ -n "$to_project" && -d "$to_project" ]]; then
                    mv "$to_project" "$from_project"
                    _log_success "Restored project: $to_project -> $from_project"
                fi

                if [[ "$source_moved" != "false" ]]; then
                    if [[ -d "$new_path" ]]; then
                        mv "$new_path" "$old_path"
                        _log_success "Restored source: $new_path -> $old_path"
                    else
                        _log_warn "Source directory not found during undo: $new_path"
                    fi
                else
                    _log_info "Original operation skipped filesystem move; leaving source directories untouched"
                fi

                if [[ -n "$from_project" ]]; then
                    _migrate_project "$new_path" "$old_path" "$from_project"
                fi
            fi

            rm -f "$UNDO_FILE"
            _log_success "Undo completed successfully"
            ;;
        *)
            _log_error "Unknown operation: $operation"
            return 1
            ;;
    esac
}

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

# Count occurrences of a cwd path within a JSON/JSONL file (whitespace tolerant)
_count_cwd_occurrences() {
    local file="$1" target="$2"

    if [[ -z "$file" || -z "$target" ]]; then
        echo 0
        return 0
    fi

    if ! command -v python3 >/dev/null 2>&1; then
        echo "ERROR:python3-missing"
        return 0
    fi

    python3 - "$file" "$target" <<'PYHELP'
import sys
import re
from pathlib import Path

file_path, target = sys.argv[1], sys.argv[2]
pattern = re.compile(r'\"cwd\"\s*:\s*\"' + re.escape(target) + r'\"')
try:
    text = Path(file_path).read_text(encoding='utf-8')
except Exception as exc:  # pragma: no cover - surfaced via shell logging
    print(f"ERROR:{exc}")
    sys.exit(0)
print(len(pattern.findall(text)))
PYHELP
}

# Replace cwd path occurrences within a JSON/JSONL file (whitespace tolerant)
_replace_cwd_path() {
    local file="$1" old_path="$2" new_path="$3"

    if [[ -z "$file" || -z "$old_path" || -z "$new_path" ]]; then
        echo 0
        return 0
    fi

    if ! command -v python3 >/dev/null 2>&1; then
        echo "ERROR:python3-missing"
        return 0
    fi

    python3 - "$file" "$old_path" "$new_path" <<'PYHELP'
import sys
import re
from pathlib import Path

file_path, old_path, new_path = sys.argv[1], sys.argv[2], sys.argv[3]
pattern = re.compile(r'\"cwd\"\s*:\s*\"' + re.escape(old_path) + r'\"')

try:
    text = Path(file_path).read_text(encoding='utf-8')
except Exception as exc:  # pragma: no cover - surfaced via shell logging
    print(f"ERROR:read:{exc}")
    sys.exit(0)

updated, replacements = pattern.subn(f'\"cwd\":\"{new_path}\"', text)

if replacements:
    try:
        Path(file_path).write_text(updated, encoding='utf-8')
    except Exception as exc:  # pragma: no cover
        print(f"ERROR:write:{exc}")
        sys.exit(0)

print(replacements)
PYHELP
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

    MIGRATION_LAST_UPDATED_FILES=0
    MIGRATION_LAST_TOTAL_CHANGES=0

    if [[ -z "$project_dir" || ! -d "$project_dir" ]]; then
        _log_error "Project directory not found: $project_dir"
        return 1
    fi

    local sessions=()
    local session_counts=()

    while IFS= read -r session; do
        local count_raw
        count_raw=$(_count_cwd_occurrences "$session" "$old_path")
        count_raw=$(printf '%s' "$count_raw" | tr -d '\n')
        if [[ "$count_raw" == ERROR:* ]]; then
            _log_warn "Skipping $(basename "$session") due to read error: ${count_raw#ERROR:}"
            continue
        fi
        if [[ "$count_raw" =~ ^[0-9]+$ ]] && (( count_raw > 0 )); then
            sessions+=("$session")
            session_counts+=("$count_raw")
        fi
    done < <(_find_project_sessions "$project_dir")

    if [[ ${#sessions[@]} -eq 0 ]]; then
        _log_warn "No sessions found with path: $old_path"
        return 0
    fi

    _log_info "Found ${#sessions[@]} sessions to migrate"

    if [[ "$INTERACTIVE" == "true" ]]; then
        _log_info "Preview of changes:"
        local idx
        for idx in "${!sessions[@]}"; do
            local session="${sessions[$idx]}"
            local count="${session_counts[$idx]}"
            echo "  $(basename "$session"): ${count} occurrences"
        done
        echo

        if ! _confirm "Proceed with migration?"; then
            _log_warn "Migration cancelled"
            return 1
        fi
    fi

    case "$BACKUP_STRATEGY" in
        "project")
            _backup_project "$project_dir"
            ;;
        "file")
            local session_path
            for session_path in "${sessions[@]}"; do
                _backup_file "$session_path"
            done
            ;;
        *)
            _log_error "Unknown backup strategy: $BACKUP_STRATEGY"
            return 1
            ;;
    esac

    local success_count=0
    local total_changes=0

    local i
    for i in "${!sessions[@]}"; do
        local session="${sessions[$i]}"
        _log_info "Migrating: $(basename "$session")"

        local planned_raw
        planned_raw=$(_count_cwd_occurrences "$session" "$old_path")
        planned_raw=$(printf '%s' "$planned_raw" | tr -d '\n')

        if [[ "$planned_raw" == ERROR:* ]]; then
            _log_error "Failed to inspect $(basename "$session"): ${planned_raw#ERROR:}"
            continue
        fi

        local planned_count="$planned_raw"

        if [[ "$DRY_RUN" == "true" ]]; then
            _log_debug "DRY RUN: Would replace $planned_count occurrences in $session"
            ((total_changes += planned_count))
            ((success_count++))
            continue
        fi

        if (( planned_count == 0 )); then
            _log_debug "No changes needed for $(basename "$session")"
            continue
        fi

        local replace_raw
        replace_raw=$(_replace_cwd_path "$session" "$old_path" "$new_path")
        replace_raw=$(printf '%s' "$replace_raw" | tr -d '\n')

        if [[ "$replace_raw" == ERROR:write:* ]]; then
            _log_error "Failed to update $(basename "$session"): ${replace_raw#ERROR:write:}"
            continue
        fi

        if [[ "$replace_raw" == ERROR:read:* ]]; then
            _log_error "Failed to read $(basename "$session"): ${replace_raw#ERROR:read:}"
            continue
        fi

        if [[ "$replace_raw" == ERROR:python3-missing ]]; then
            _log_error "python3 is required for safe JSON path replacement"
            return 1
        fi

        if [[ ! "$replace_raw" =~ ^[0-9]+$ ]]; then
            _log_error "Unexpected replace count for $(basename "$session"): $replace_raw"
            continue
        fi

        local replaced="$replace_raw"
        ((total_changes += replaced))
        ((success_count++))
        _log_success "Updated $(basename "$session"): $replaced changes"
    done

    MIGRATION_LAST_UPDATED_FILES=$success_count
    MIGRATION_LAST_TOTAL_CHANGES=$total_changes

    _log_success "Migration completed: $success_count/${#sessions[@]} files, $total_changes total changes"
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

# Suggest a Claude project directory path for a given source path
_suggest_project_dir_for() {
    local new_path="$1"

    # Expand ~ to full home path
    new_path="${new_path/#\~/$HOME}"

    # Convert absolute path to Claude Code project naming convention
    # Claude uses double dashes for dots and single dash for path separators
    local encoded_name
    encoded_name=$(echo "$new_path" | sed 's|^/||' | sed 's|\.|-|g' | sed 's|/|-|g')
    echo "$CLAUDE_DIR/projects/-${encoded_name}"
}

# Resolve an input path to absolute
_resolve_absolute_path() {
    local base_dir="$1"
    local input_path="$2"

    if [[ -z "$input_path" ]]; then
        echo ""
        return 0
    fi

    # Expand ~ to home directory
    if [[ "$input_path" == ~* ]]; then
        input_path="${input_path/#\~/$HOME}"
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
            ;;

        "move"|"mv")
            # Move source directory and update Claude sessions
            local old_path="$2"
            local new_path="$3"

            if [[ -z "$old_path" || -z "$new_path" ]]; then
                echo "Usage: cm move <old_path> <new_path>"
                return 1
            fi

            # Normalize paths
            old_path="${old_path%/}"
            new_path="${new_path%/}"

            local from_project to_project
            from_project=$(_suggest_project_dir_for "$old_path")
            to_project=$(_suggest_project_dir_for "$new_path")

            _log_info "=== Move Operation ==="
            _log_info "Source:      $old_path"
            _log_info "Destination: $new_path"
            _log_info "Project:     $from_project -> $to_project"

            if [[ "$INTERACTIVE" == "true" ]] && ! _confirm "Proceed with move?"; then
                _log_warn "Move cancelled"
                return 1
            fi

            # Save undo info
            _save_undo_info "move" "$old_path" "$new_path" "$from_project" "$to_project" "true"

            # Move source directory
            if [[ -d "$old_path" ]]; then
                mkdir -p "$(dirname "$new_path")"
                mv "$old_path" "$new_path"
                _log_success "Moved source: $old_path -> $new_path"
            fi

            # Move project directory if exists
            if [[ -d "$from_project" ]]; then
                mkdir -p "$(dirname "$to_project")"
                mv "$from_project" "$to_project"
                _log_success "Moved project: $(basename "$from_project") -> $(basename "$to_project")"

                # Update session paths
                _migrate_project "$old_path" "$new_path" "$to_project"
            fi

            _log_success "Move completed. Use 'cm undo' to revert if needed."
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

        "undo")
            _undo_last_operation
            ;;

        "config"|"cfg")
            _log_info "Current configuration:"
            echo "  CLAUDE_DIR: $CLAUDE_DIR"
            echo "  BACKUP_STRATEGY: $BACKUP_STRATEGY"
            echo "  INTERACTIVE: $INTERACTIVE"
            echo "  DRY_RUN: $DRY_RUN"
            ;;

        "help"|"h"|"")
            cat <<'HELP'
Claude Manager - Manage Claude projects and session paths

USAGE:
    cm <command> [arguments]

COMMANDS:
    migrate <old_path> <new_path> [project_dir]
        Update session paths after moving/renaming source directory

    move <old_path> <new_path>
        Move source directory and update Claude sessions atomically

    list [project_dir]
        List all projects or sessions within a specific project

    undo
        Undo the last move operation

    config
        Display current configuration values

    help
        Show this help message

ALIASES:
    cm, cm-migrate, cm-move, cm-list

CONFIGURATION (Environment Variables):
    CLAUDE_DIR              Claude directory (default: ~/.claude)
    CLAUDE_BACKUP_STRATEGY  file|project (default: file)
    CLAUDE_INTERACTIVE      true|false (default: true)
    CLAUDE_DRY_RUN          true|false (default: false)
    CLAUDE_DEBUG            0|1 (default: 0)

EXAMPLES:
    # Migrate paths in sessions
    cm migrate "/Users/old/project" "/Users/new/project"

    # Move directory and update sessions
    cm move "/old/src" "/new/src"

    # Dry run to preview changes
    CLAUDE_DRY_RUN=true cm move "/old" "/new"

    # List all projects
    cm list

    # Undo last operation
    cm undo

For more information, see the documentation at:
https://github.com/yourusername/claude-manager
HELP
            ;;

        *)
            echo "Unknown command: $cmd. Use 'cm help' for usage."
            return 1
            ;;
    esac
}

# Convenient aliases
alias cm='claude_manager'
alias cm-migrate='claude_manager migrate'
alias cm-move='claude_manager move'
alias cm-list='claude_manager list'

# Execute main function if script is run directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    claude_manager "$@"
fi
