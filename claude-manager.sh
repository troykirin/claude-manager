#!/usr/bin/env bash

# Claude Manager
# Manage Claude projects and update session paths
# XDG-compliant configuration and state management

set -e

# XDG Base Directory Specification (https://specifications.freedesktop.org/basedir-spec/)
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
export XDG_STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"
export XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"

# Configuration with defaults
# Note: CLAUDE_DIR points to where Claude Code stores sessions (~/.claude)
# This is NOT XDG-compliant by Claude Code's design, but we reference it for compatibility
export CLAUDE_DIR="${CLAUDE_DIR:-$HOME/.claude}"
export BACKUP_STRATEGY="${CLAUDE_BACKUP_STRATEGY:-file}"  # file or project
export INTERACTIVE="${CLAUDE_INTERACTIVE:-true}"
export DRY_RUN="${CLAUDE_DRY_RUN:-false}"

# Undo file stored in XDG-compliant state directory
mkdir -p "${XDG_STATE_HOME}/nabi"
export UNDO_FILE="${XDG_STATE_HOME}/nabi/claude-manager.last_move_operation"

# Backup directory (XDG-compliant)
BACKUP_BASE_DIR="${XDG_STATE_HOME}/nabi/backups"
mkdir -p "$BACKUP_BASE_DIR"

# Agent mode flags (for non-interactive autonomous execution)
export AGENT_NO_CONFIRM="${AGENT_NO_CONFIRM:-false}"
export AGENT_MODE="${AGENT_MODE:-false}"
export AGENT_VERBOSE="${AGENT_VERBOSE:-false}"
export FORCE_BACKUP_STRATEGY=""

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
    if [[ "$CLAUDE_DEBUG" == "1" ]]; then
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
        printf '%s
' "$timestamp"
        printf '%s
' "$operation"
        printf '%s
' "$#"
        for arg in "$@"; do
            printf '%s
' "$arg"
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
        count_raw=$(printf '%s' "$count_raw" | tr -d '
')
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
        planned_raw=$(printf '%s' "$planned_raw" | tr -d '
')

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
        replace_raw=$(printf '%s' "$replace_raw" | tr -d '
')

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

    # Use the proper path encoder to transform new_path to Claude's naming convention
    local target_dir
    target_dir=$(_suggest_project_dir_for "$new_path")

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
    else
        # In non-interactive mode, log what's happening
        _log_debug "Auto-renaming project directory: $(basename "$project_dir") -> $(basename "$target_dir")"
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

# Find Claude projects with priority: name-based first, then session-based
_find_projects_prioritized() {
    local source_path="$1"
    local priority_matches=()
    local session_matches=()
    
    # Priority 1: Check if there's a project that matches the directory structure
    local expected_project
    expected_project=$(_suggest_project_dir_for "$source_path")
    if [[ -d "$expected_project" ]]; then
        priority_matches+=("$expected_project")
    fi
    
    # Priority 2: Find projects containing sessions with this path
    local temp_matches
    temp_matches=$(timeout 10 find "$CLAUDE_DIR/projects" -name "*.jsonl" -type f -exec grep -l "\"cwd\":\"$source_path\"" {} \; 2>/dev/null | head -5 || true)
    
    if [[ -n "$temp_matches" ]]; then
        while IFS= read -r session_file; do
            local project_dir
            project_dir=$(dirname "$session_file")
            # Skip if already in priority matches
            local in_priority=false
            for p in "${priority_matches[@]}"; do
                if [[ "$p" == "$project_dir" ]]; then
                    in_priority=true
                    break
                fi
            done
            
            if [[ "$in_priority" == false ]]; then
                # Add to session matches if not already there
                local already_added=false
                for m in "${session_matches[@]}"; do
                    if [[ "$m" == "$project_dir" ]]; then
                        already_added=true
                        break
                    fi
                done
                if [[ "$already_added" == false ]]; then
                    session_matches+=("$project_dir")
                fi
            fi
        done <<< "$temp_matches"
    fi
    
    # Return priority matches first, then session matches
    for p in "${priority_matches[@]}"; do
        echo "$p"
    done
    for s in "${session_matches[@]}"; do
        echo "$s"
    done
}

# Find Claude project directories that contain sessions referencing a given source path
_find_projects_by_session_path() {
    local source_path="$1"
    local matches=()

    # Faster approach: limit search time and results
    local temp_matches
    # Use timeout to prevent hanging on large searches
    temp_matches=$(timeout 10 find "$CLAUDE_DIR/projects" -name "*.jsonl" -type f -exec grep -l "\"cwd\":\"$source_path\"" {} \; 2>/dev/null | head -5 || true)
    
    if [[ -n "$temp_matches" ]]; then
        while IFS= read -r session_file; do
            local project_dir
            project_dir=$(dirname "$session_file")
            # Add project if not already in matches
            local already_added=false
            for m in "${matches[@]}"; do
                if [[ "$m" == "$project_dir" ]]; then
                    already_added=true
                    break
                fi
            done
            if [[ "$already_added" == "false" ]]; then
                matches+=("$project_dir")
            fi
        done <<< "$temp_matches"
    fi

    for m in "${matches[@]}"; do
        echo "$m"
    done
}

# Suggest a Claude project directory path for a given new source path
_suggest_project_dir_for() {
    local new_path="$1"
    
    # Expand ~ to full home path
    new_path="${new_path/#\~/$HOME}"
    
    # Convert absolute path to Claude Code project naming convention
    # Claude uses double dashes for dots in paths and single dash for path separators
    # Example: /Users/tryk/.claude/sync -> -Users-tryk--claude-sync
    local encoded_name
    encoded_name=$(echo "$new_path" | sed 's|^/||' | sed 's|\.|-|g' | sed 's|/|-|g')
    echo "$CLAUDE_DIR/projects/-${encoded_name}"
}

# Parse agent-mode flags from command arguments
_parse_agent_flags() {
    local -n args_ref=$1
    local -n flags_ref=$2

    # flags_ref is an associative array to store parsed flags
    local i
    for ((i = 0; i < ${#args_ref[@]}; i++)); do
        local arg="${args_ref[$i]}"
        case "$arg" in
            --no-confirm)
                flags_ref["no_confirm"]="true"
                AGENT_NO_CONFIRM="true"
                INTERACTIVE="false"
                ;;
            --agent-mode)
                flags_ref["agent_mode"]="true"
                AGENT_MODE="true"
                INTERACTIVE="false"
                ;;
            --verbose)
                flags_ref["verbose"]="true"
                AGENT_VERBOSE="true"
                CLAUDE_DEBUG="1"
                ;;
            --backup-strategy=*)
                local strategy="${arg#*=}"
                flags_ref["backup_strategy"]="$strategy"
                FORCE_BACKUP_STRATEGY="$strategy"
                ;;
            --force-backup)
                flags_ref["force_backup"]="true"
                ;;
        esac
    done
}

# Create backup file in XDG-compliant location
_create_project_backup() {
    local project_dir="$1"
    local project_name
    project_name=$(basename "$project_dir")

    local backup_file="$BACKUP_BASE_DIR/$(date +%Y%m%d_%H%M%S)_${project_name}.tar.gz"

    _log_info "Creating backup: $backup_file"

    # Use -- to separate tar flags from filenames (project names start with -)
    if tar -czf "$backup_file" -C "$CLAUDE_DIR/projects" -- "$project_name" 2>/dev/null; then
        _log_success "Backup created: $backup_file"
        echo "$backup_file"
        return 0
    else
        _log_error "Failed to create backup at: $backup_file"
        return 1
    fi
}

# Output agent-mode metadata
_output_migration_metadata() {
    local status="$1"
    local old_path="$2"
    local new_path="$3"
    local project_dir="$4"
    local sessions_updated="${5:-0}"
    local total_changes="${6:-0}"
    local backup_path="${7:-}"

    if [[ "$AGENT_MODE" == "true" ]] || [[ "$AGENT_NO_CONFIRM" == "true" ]]; then
        echo ""
        echo "[MIGRATION_METADATA]"
        echo "MIGRATION_STATUS=$status"
        echo "MIGRATION_OLD_PATH=$old_path"
        echo "MIGRATION_NEW_PATH=$new_path"
        echo "MIGRATION_PROJECT=$(basename "$project_dir")"
        echo "MIGRATION_SESSIONS_UPDATED=$sessions_updated"
        echo "MIGRATION_TOTAL_CHANGES=$total_changes"
        if [[ -n "$backup_path" ]]; then
            echo "MIGRATION_BACKUP_PATH=$backup_path"
            echo "MIGRATION_BACKUP_VERIFIED=true"
        fi
        echo "MIGRATION_UNDO_FILE=$UNDO_FILE"
        echo "[/MIGRATION_METADATA]"
        echo ""
    fi
}

# Main CLI function
claude_manager() {
    local cmd="$1"
    shift  # Remove command from arguments

    # Parse agent-mode flags and collect positional arguments
    declare -A flags
    local positional_args=()
    for arg in "$@"; do
        case "$arg" in
            --no-confirm)
                flags["no_confirm"]="true"
                AGENT_NO_CONFIRM="true"
                INTERACTIVE="false"
                ;;
            --agent-mode)
                flags["agent_mode"]="true"
                AGENT_MODE="true"
                INTERACTIVE="false"
                ;;
            --verbose)
                flags["verbose"]="true"
                AGENT_VERBOSE="true"
                CLAUDE_DEBUG="1"
                ;;
            --backup-strategy=*)
                local strategy="${arg#*=}"
                flags["backup_strategy"]="$strategy"
                FORCE_BACKUP_STRATEGY="$strategy"
                ;;
            --force-backup)
                flags["force_backup"]="true"
                ;;
            --*)
                # Unknown flag, skip
                ;;
            *)
                positional_args+=("$arg")
                ;;
        esac
    done

    case "$cmd" in
        "migrate"|"m")
            local old_path="${positional_args[0]:-}"
            local new_path="${positional_args[1]:-}"
            local project_dir="${positional_args[2]:-}"

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
            else
                # Auto-detect project directory if not provided
                if [[ -z "$project_dir" ]]; then
                    _log_debug "Auto-detecting project directory for path: $old_path"
                    # Find the project containing sessions with this old_path
                    local projects=()
                    local found_projects
                    found_projects=$(_find_projects_prioritized "$old_path")
                    if [[ -n "$found_projects" ]]; then
                        while IFS= read -r proj; do
                            projects+=("$proj")
                        done <<< "$found_projects"
                        project_dir="${projects[0]}"
                        _log_info "Auto-detected project: $(basename "$project_dir")"
                    else
                        _log_error "Could not auto-detect project directory for path: $old_path"
                        return 1
                    fi
                fi
            fi

            # Agent mode: Create mandatory backup before migration
            local backup_path=""
            if [[ "$AGENT_NO_CONFIRM" == "true" ]] || [[ "$AGENT_MODE" == "true" ]] || [[ "${flags[force_backup]}" == "true" ]]; then
                backup_path=$(_create_project_backup "$project_dir")
                if [[ $? -ne 0 ]]; then
                    _output_migration_metadata "failure" "$old_path" "$new_path" "$project_dir" 0 0 ""
                    return 1
                fi
            fi

            # Perform the actual migration
            if ! _migrate_project "$old_path" "$new_path" "$project_dir"; then
                _output_migration_metadata "failure" "$old_path" "$new_path" "$project_dir" 0 0 "$backup_path"
                return 1
            fi

            # Get migration stats for metadata output
            local sessions_updated="$MIGRATION_LAST_UPDATED_FILES"
            local total_changes="$MIGRATION_LAST_TOTAL_CHANGES"

            # After migrating paths in sessions, rename the Claude project dir to
            # match the new path's basename if safe/applicable.
            _sync_project_dir_name_with_path "$project_dir" "$new_path"

            # Output metadata for agent consumption
            _output_migration_metadata "success" "$old_path" "$new_path" "$project_dir" "$sessions_updated" "$total_changes" "$backup_path"
            ;;

        "move"|"mv")
            # Hardened move flow with comprehensive edge case handling
            local old_path="$2"
            local new_path="$3"

            if [[ -n "$old_path" && -n "$new_path" ]]; then
                _log_info "=== Move Operation (hardened) ==="
                
                # ========== INPUT NORMALIZATION & VALIDATION ==========
                _log_debug "Normalizing and validating input paths..."
                
                # Resolve to absolute paths
                if command -v realpath >/dev/null 2>&1 && realpath -m /tmp >/dev/null 2>&1; then
                    # GNU realpath with -m support
                    local resolved_old
                    resolved_old=$(realpath "$old_path" 2>/dev/null || true)
                    if [[ -z "$resolved_old" ]]; then
                        resolved_old=$(_resolve_absolute_path "$(pwd)" "$old_path")
                    fi
                    if [[ -z "$resolved_old" ]]; then
                        _log_error "Invalid old_path: $2"
                        return 1
                    fi

                    local resolved_new
                    resolved_new=$(realpath -m "$new_path" 2>/dev/null || true)
                    if [[ -z "$resolved_new" ]]; then
                        resolved_new=$(_resolve_absolute_path "$(pwd)" "$new_path")
                    fi
                    if [[ -z "$resolved_new" ]]; then
                        _log_error "Invalid new_path: $3"
                        return 1
                    fi

                    old_path="$resolved_old"
                    new_path="$resolved_new"
                else
                    # Fallback path resolution (macOS realpath or no realpath)
                    if command -v realpath >/dev/null 2>&1; then
                        # macOS realpath (existing paths only)
                        local resolved_old
                        resolved_old=$(realpath "$old_path" 2>/dev/null || true)
                        if [[ -z "$resolved_old" ]]; then
                            resolved_old=$(_resolve_absolute_path "$(pwd)" "$old_path")
                        fi
                        if [[ -z "$resolved_old" ]]; then
                            _log_error "Invalid old_path: $2"
                            return 1
                        fi
                        old_path="$resolved_old"
                        new_path=$(_resolve_absolute_path "$(pwd)" "$new_path")
                    else
                        # No realpath available
                        old_path=$(_resolve_absolute_path "$(pwd)" "$old_path")
                        new_path=$(_resolve_absolute_path "$(pwd)" "$new_path")
                    fi
                fi
                
                # Normalize paths - remove trailing slashes for consistency
                old_path="${old_path%/}"
                new_path="${new_path%/}"
                
                # Basic path validation
                if [[ "$old_path" == "$new_path" ]]; then
                    _log_error "Source and destination paths are identical: $old_path"
                    return 1
                fi
                
                # Check for nesting (moving into itself)
                if [[ "$new_path" == "$old_path"/* ]]; then
                    _log_error "Cannot move directory into itself: $old_path -> $new_path"
                    return 1
                fi
                
                # ========== PRE-FLIGHT CHECKS ==========
                _log_debug "Performing pre-flight validation..."

                local source_exists="true"
                if [[ ! -e "$old_path" ]]; then
                    _log_warn "Source path not found: $old_path"
                    source_exists="false"
                elif [[ ! -d "$old_path" ]]; then
                    _log_error "Source path is not a directory: $old_path"
                    return 1
                fi

                if [[ "$source_exists" == "true" && ! -r "$old_path" ]]; then
                    _log_error "Source directory is not readable: $old_path"
                    return 1
                fi
                
                # Handle destination existence
                if [[ -e "$new_path" ]]; then
                    if [[ ! -d "$new_path" ]]; then
                        _log_error "Destination exists but is not a directory: $new_path"
                        return 1
                    fi
                    
                    local dest_empty=true
                    if [[ -n "$(find "$new_path" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]]; then
                        dest_empty=false
                    fi
                    
                    if [[ "$dest_empty" == "false" ]]; then
                        if [[ "$INTERACTIVE" == "true" ]]; then
                            if ! _confirm "Destination directory exists and is not empty. Merge/replace contents?"; then
                                _log_warn "Move cancelled - destination directory exists"
                                return 1
                            fi
                        else
                            _log_error "Destination directory exists and is not empty (non-interactive mode): $new_path"
                            return 1
                        fi
                    fi
                fi
                
                # ========== CONCURRENCY GUARD ==========
                if [[ "${FORCE:-false}" != "true" ]]; then
                    _log_debug "Checking for running Claude processes..."
                    local claude_processes
                    claude_processes=$(pgrep -f "[Cc]laude" 2>/dev/null | wc -l) || claude_processes=0
                    if [[ "$claude_processes" -gt 0 ]]; then
                        _log_warn "Detected $claude_processes running Claude processes"
                        if [[ "$INTERACTIVE" == "true" ]]; then
                            if ! _confirm "Continue with active Claude processes? (may cause data corruption)"; then
                                _log_warn "Move cancelled - Claude processes active"
                                return 1
                            fi
                        else
                            _log_error "Active Claude processes detected (use FORCE=true to override)"
                            return 1
                        fi
                    fi
                fi
                
                # ========== PROJECT MAPPING & COUNTING ==========
                local from_project to_project
                from_project=$(_suggest_project_dir_for "$old_path")
                to_project=$(_suggest_project_dir_for "$new_path")
                
                # Count sessions and occurrences before any changes
                local session_count=0 occurrence_count=0
                if [[ -d "$from_project" ]]; then
                    _log_debug "Scanning project for sessions to update..."
                    while IFS= read -r session_file; do
                        if [[ -f "$session_file" ]]; then
                            local file_occurrences
                            # Simplified grep pattern for better performance
                            file_occurrences=$(grep -c "\"cwd\":\"$old_path\"" "$session_file" 2>/dev/null || echo 0)
                            # Ensure we have a clean integer (strip any whitespace/newlines)
                            file_occurrences="${file_occurrences//[^0-9]/}"
                            [[ -z "$file_occurrences" ]] && file_occurrences=0
                            if [[ "$file_occurrences" -gt 0 ]]; then
                                session_count=$((session_count + 1))
                                occurrence_count=$((occurrence_count + file_occurrences))
                            fi
                        fi
                    done < <(find "$from_project" -name "*.jsonl" -type f 2>/dev/null)
                fi
                
                # ========== OPERATION PLAN ==========
                _log_info "=== Operation Plan ==="
                _log_info "Source:      $old_path"
                _log_info "Destination: $new_path"
                _log_info "Project:     $from_project -> $to_project"
                _log_info "Sessions:    $session_count files with $occurrence_count cwd occurrences"

                if [[ "$source_exists" == "false" ]]; then
                    _log_warn "Source directory missing - will only update Claude session metadata"
                fi
                
                if [[ ! -d "$from_project" ]]; then
                    _log_warn "Project directory not found - will perform source-only move"
                fi
                
                # ========== DRY RUN SUPPORT ==========
                if [[ "$DRY_RUN" == "true" ]]; then
                    _log_info "=== DRY RUN - No changes will be made ==="
                    if [[ "$source_exists" == "true" ]]; then
                        _log_debug "Would move: $old_path -> $new_path"
                    else
                        _log_debug "Would skip filesystem move (source missing): $old_path"
                    fi
                    if [[ -d "$from_project" ]]; then
                        _log_debug "Would move project: $from_project -> $to_project"
                        _log_debug "Would update $session_count session files ($occurrence_count total replacements)"
                    fi
                    _log_info "DRY RUN completed - use DRY_RUN=false to execute"
                    return 0
                fi
                
                # ========== INTERACTIVE CONFIRMATION ==========
                if [[ "$INTERACTIVE" == "true" ]]; then
                    if ! _confirm "Proceed with move operation?"; then
                        _log_warn "Move cancelled by user"
                        return 1
                    fi
                fi
                
                # ========== TRANSACTIONAL EXECUTION ==========
                _log_info "=== Executing Move Operation ==="

                local source_moved="false"

                # Step 1: Move source directory (if present)
                _log_info "Step 1/3: Moving source directory..."
                if [[ "$source_exists" == "true" ]]; then
                    local new_parent
                    new_parent="$(dirname "$new_path")"
                    if [[ ! -d "$new_parent" ]]; then
                        if ! mkdir -p "$new_parent"; then
                            _log_error "Failed to create parent directory: $new_parent"
                            rm -f "$UNDO_FILE"
                            return 1
                        fi
                    fi

                    if ! mv "$old_path" "$new_path"; then
                        _log_error "Failed to move source directory"
                        rm -f "$UNDO_FILE"
                        return 1
                    fi
                    _log_success "Moved source directory: $old_path -> $new_path"
                    source_moved="true"
                else
                    _log_warn "Source directory missing; skipping filesystem move step"
                fi

                # Save undo info after establishing move state
                _save_undo_info "move" "$old_path" "$new_path" "$from_project" "$to_project" "$source_moved"
                
                # Step 2: Move project directory (if exists)
                _log_info "Step 2/3: Moving project directory..."
                local project_moved="false"
                if [[ -d "$from_project" ]]; then
                    local project_parent
                    project_parent="$(dirname "$to_project")"
                    if [[ ! -d "$project_parent" ]]; then
                        mkdir -p "$project_parent"
                    fi
                    
                    if mv "$from_project" "$to_project"; then
                        _log_success "Moved project: $(basename "$from_project") -> $(basename "$to_project")"
                        project_moved="true"
                    else
                        _log_error "Failed to move project directory - attempting rollback"
                        # Rollback source move
                        if mv "$new_path" "$old_path" 2>/dev/null; then
                            _log_warn "Rolled back source directory move"
                        else
                            _log_error "CRITICAL: Failed to rollback source move - manual intervention required"
                        fi
                        rm -f "$UNDO_FILE"
                        return 1
                    fi
                else
                    _log_warn "Project directory not found - source-only move completed"
                fi
                
                # Step 3: Update JSON sessions (robust)
                _log_info "Step 3/3: Updating session files..."
                local target_project
                target_project="$to_project"
                if [[ "$project_moved" != "true" && -d "$from_project" ]]; then
                    target_project="$from_project"
                fi

                if [[ -d "$target_project" ]]; then
                    if ! _migrate_project "$old_path" "$new_path" "$target_project"; then
                        _log_warn "Session update encountered issues in $(basename "$target_project")"
                    else
                        local updated_files="$MIGRATION_LAST_UPDATED_FILES"
                        local total_replacements="$MIGRATION_LAST_TOTAL_CHANGES"
                        _log_success "Updated $updated_files session files ($total_replacements total replacements)"
                        if (( total_replacements == 0 )); then
                            _log_info "No session references required updates"
                        fi
                    fi
                else
                    _log_info "No project directory to update - source move completed"
                fi
                
                if [[ -d "$target_project" ]]; then
                    local updated_files=0 total_replacements=0
                    local old_esc new_esc
                    old_esc=$(printf '%s' "$old_path" | sed -e 's/[\/&]/\\&/g')
                    new_esc=$(printf '%s' "$new_path" | sed -e 's/[\/&]/\\&/g')
                    
                    while IFS= read -r session_file; do
                        if [[ -f "$session_file" ]]; then
                            # Count occurrences before replacement
                            local before_count
                            before_count=$(grep -cE "\"cwd\"[[:space:]]*:[[:space:]]*\"$old_esc\"" "$session_file" 2>/dev/null || echo 0)
                            # Sanitize before_count to ensure it's a clean integer
                            before_count=$(echo "$before_count" | tr -d '\n' | sed 's/[^0-9]//g')
                            [[ -z "$before_count" ]] && before_count=0
                            
                            if [[ "$before_count" -gt 0 ]]; then
                                # Backup the file
                                cp "$session_file" "${session_file}.pre-move-backup" 2>/dev/null || true
                                
                                # Perform replacement with whitespace tolerance
                                if [[ "$OSTYPE" == "darwin"* ]]; then
                                    sed -i '' -E "s|\"cwd\"[[:space:]]*:[[:space:]]*\"$old_esc\"|\"cwd\":\"$new_esc\"|g" "$session_file"
                                else
                                    sed -i -E "s|\"cwd\"[[:space:]]*:[[:space:]]*\"$old_esc\"|\"cwd\":\"$new_esc\"|g" "$session_file"
                                fi
                                
                                # Verify replacement
                                local after_count
                                after_count=$(grep -cE "\"cwd\"[[:space:]]*:[[:space:]]*\"$new_esc\"" "$session_file" 2>/dev/null || echo 0)
                                
                                if [[ "$after_count" -eq "$before_count" ]]; then
                                    ((updated_files++))
                                    ((total_replacements += after_count))
                                    # Remove backup on success
                                    rm -f "${session_file}.pre-move-backup" 2>/dev/null
                                else
                                    _log_warn "Replacement mismatch in $(basename "$session_file"): expected $before_count, got $after_count"
                                fi
                            fi
                        fi
                    done < <(find "$target_project" -name "*.jsonl" -type f 2>/dev/null)
                    
                    _log_success "Updated $updated_files session files ($total_replacements total replacements)"
                    
                    # ========== POST-VERIFICATION ==========
                    _log_debug "Performing post-verification scan..."
                    local remaining_count=0
                    while IFS= read -r session_file; do
                        local remaining
                        remaining=$(grep -cE "\"cwd\"[[:space:]]*:[[:space:]]*\"$old_esc\"" "$session_file" 2>/dev/null || echo 0)
                        ((remaining_count += remaining))
                    done < <(find "$target_project" -name "*.jsonl" -type f 2>/dev/null)
                    
                    if [[ "$remaining_count" -gt 0 ]]; then
                        _log_warn "Post-verification: $remaining_count residual old path references remain"
                        _log_info "Use 'cm verify $target_project' to investigate"
                    else
                        _log_success "Post-verification: All path references updated successfully"
                    fi
                else
                    _log_info "No project directory to update - source move completed"
                fi
                
                _log_success "=== Move Operation Completed Successfully ==="
                _log_info "Summary:"
                if [[ "$source_moved" == "true" ]]; then
                    _log_info "  • Source moved: $old_path -> $new_path"
                else
                    _log_info "  • Source directory missing; filesystem move skipped"
                fi
                if [[ "$project_moved" == "true" ]]; then
                    _log_info "  • Project moved: $(basename \"$from_project\") -> $(basename \"$to_project\")"
                fi
                if [[ -d "$target_project" ]]; then
                    _log_info "  • Sessions updated: ${MIGRATION_LAST_UPDATED_FILES:-0} files, ${MIGRATION_LAST_TOTAL_CHANGES:-0} replacements"
                fi
                _log_info "Use 'cm undo' to revert if needed"
                return 0
            fi

            # Fallback to interactive full flow when args are not provided
            _log_info "=== Move Operation ==="
            _log_info "This will move both source directory and Claude project"
            claude_manager full "$2" "$3"
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

            # Auto-detect Claude project(s) referencing this old_path (prioritized)
            while IFS= read -r p; do
                project_candidates+=("$p")
            done < <(_find_projects_prioritized "$old_path")

            if [[ ${#project_candidates[@]} -eq 0 ]]; then
                _log_warn "No Claude projects directly reference: $old_path"
                _log_info "This appears to be a subdirectory without its own Claude project"
                
                # Option to just move the directory without updating sessions
                if _confirm "Move directory without updating Claude sessions?"; then
                    _log_info "=== Simple directory move ==="
                    if [[ ! -d "$old_path" ]]; then
                        _log_error "Directory not found: $old_path"
                        return 1
                    fi
                    
                    _save_undo_info "move" "$old_path" "$new_path" "none" "none" "true"
                    
                    if mv "$old_path" "$new_path"; then
                        _log_success "Moved directory: $old_path -> $new_path"
                        _log_info "Note: No Claude sessions were updated"
                        return 0
                    else
                        _log_error "Failed to move directory"
                        rm -f "$UNDO_FILE"
                        return 1
                    fi
                else
                    _log_info "Cancelled - no directory was moved"
                    return 1
                fi
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

            # Auto-generate destination Claude project directory
            local to_project
            to_project=$(_suggest_project_dir_for "$new_path")
            _log_info "Destination project: $(basename "$to_project")"

            _log_info "=== Plan ==="
            echo "  Source dir: $old_path"
            echo "  New dir:    $new_path"
            echo "  Project:    $selected_project"
            echo "  New proj:   $to_project"

            if ! _confirm "Proceed with full move?"; then
                _log_warn "Cancelled"
                return 1
            fi

            # First validate we can update sessions before moving anything
            _log_info "=== Pre-flight validation ==="
            local session_count=0
            while IFS= read -r session; do
                if grep -q "$old_path" "$session" 2>/dev/null; then
                    ((session_count++))
                fi
            done < <(_find_project_sessions "$selected_project")
            
            if [[ $session_count -eq 0 ]]; then
                _log_warn "No sessions found with path: $old_path"
                if ! _confirm "Continue anyway?"; then
                    _log_warn "Cancelled"
                    return 1
                fi
            else
                _log_success "Found $session_count sessions to update"
            fi
            
            # Save undo information before making changes
            _save_undo_info "move" "$old_path" "$new_path" "$selected_project" "$to_project" "true"
            
            # Now perform operations in safe order:
            # 1. Update sessions first (while paths still exist)
            if ! _migrate_project "$old_path" "$new_path" "$selected_project"; then
                _log_error "Failed to migrate sessions - aborting move"
                rm -f "$UNDO_FILE"
                return 1
            fi
            
            # 2. Move source directory
            if ! _move_src_dir "$old_path" "$new_path"; then
                _log_error "Failed to move source directory - rolling back"
                _migrate_project "$new_path" "$old_path" "$selected_project"
                rm -f "$UNDO_FILE"
                return 1
            fi
            
            # 3. Move project directory
            if ! _move_project_dir "$selected_project" "$to_project"; then
                _log_error "Failed to move project directory - rolling back"
                _move_src_dir "$new_path" "$old_path"
                _migrate_project "$new_path" "$old_path" "$selected_project"
                rm -f "$UNDO_FILE"
                return 1
            fi
            
            _log_success "Move completed successfully. Use 'cm undo' to revert if needed."
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
            
        "verify"|"v")
            # Verify project directory for path consistency
            local project_dir="$2"
            
            if [[ -z "$project_dir" ]]; then
                if [[ "$INTERACTIVE" == "true" ]]; then
                    project_dir=$(_select_project)
                    [[ $? -ne 0 ]] && return 1
                else
                    _log_error "Usage: cm verify <project_dir>"
                    return 1
                fi
            fi
            
            if [[ ! -d "$project_dir" ]]; then
                _log_error "Project directory not found: $project_dir"
                return 1
            fi
            
            _log_info "=== Project Verification: $(basename "$project_dir") ==="
            
            # Extract expected path from project directory name
            local encoded_name
            encoded_name=$(basename "$project_dir")
            if [[ "$encoded_name" == -* ]]; then
                # Decode the expected path
                local expected_path
                expected_path=$(echo "${encoded_name#-}" | sed 's|-|/|g')
                expected_path="/$expected_path"
                _log_info "Expected path from encoding: $expected_path"
                
                # Scan sessions for path mismatches
                local total_sessions=0 consistent_sessions=0 mismatched_sessions=0
                local unique_paths=() path_counts=()
                
                while IFS= read -r session_file; do
                    if [[ -f "$session_file" ]]; then
                        ((total_sessions++))
                        
                        # Extract all cwd paths from this session
                        local session_paths
                        session_paths=$(grep -oE "\"cwd\"[[:space:]]*:[[:space:]]*\"[^\"]+\"" "$session_file" 2>/dev/null | \
                                      sed -E 's|\"cwd\"[[:space:]]*:[[:space:]]*\"([^\"]+)\"|\1|g' | sort -u)
                        
                        local has_expected=false has_other=false
                        while IFS= read -r path; do
                            if [[ -n "$path" ]]; then
                                if [[ "$path" == "$expected_path" ]]; then
                                    has_expected=true
                                else
                                    has_other=true
                                    # Track unique unexpected paths
                                    local found=false
                                    for i in "${!unique_paths[@]}"; do
                                        if [[ "${unique_paths[$i]}" == "$path" ]]; then
                                            ((path_counts[$i]++))
                                            found=true
                                            break
                                        fi
                                    done
                                    if [[ "$found" == "false" ]]; then
                                        unique_paths+=("$path")
                                        path_counts+=(1)
                                    fi
                                fi
                            fi
                        done <<< "$session_paths"
                        
                        if [[ "$has_expected" == "true" && "$has_other" == "false" ]]; then
                            ((consistent_sessions++))
                        elif [[ "$has_other" == "true" ]]; then
                            ((mismatched_sessions++))
                        fi
                    fi
                done < <(find "$project_dir" -name "*.jsonl" -type f 2>/dev/null)
                
                # Report results
                _log_info "=== Verification Results ==="
                _log_info "Total sessions: $total_sessions"
                _log_info "Consistent: $consistent_sessions"
                _log_info "Mismatched: $mismatched_sessions"
                
                if [[ ${#unique_paths[@]} -gt 0 ]]; then
                    _log_warn "Unexpected paths found:"
                    for i in "${!unique_paths[@]}"; do
                        echo "  ${unique_paths[$i]} (${path_counts[$i]} occurrences)"
                    done
                fi
                
                if [[ "$mismatched_sessions" -eq 0 ]]; then
                    _log_success "✓ All sessions have consistent paths"
                else
                    _log_warn "⚠ $mismatched_sessions sessions have path inconsistencies"
                    _log_info "Consider running migration to fix inconsistencies"
                fi
            else
                _log_warn "Cannot decode expected path from project name: $encoded_name"
            fi
            ;;
            
        "organize"|"org")
            # Granular session organization commands
            local subcmd="$2"
            
            case "$subcmd" in
                "extract"|"e")
                    # Extract specific session UUID to target directory
                    local uuid="$3"
                    local source_dir="$4"
                    local target_dir="$5"
                    
                    # Handle --from and --to flags for better UX
                    if [[ "$uuid" == "--help" || -z "$uuid" ]]; then
                        _log_info "Usage: cm organize extract <uuid> --from <source> --to <target>"
                        _log_info "   or: cm organize extract <uuid> <source> <target>"
                        _log_info ""
                        _log_info "Extract a single session by UUID to another directory"
                        _log_info "Creates target if it doesn't exist, merges if it does"
                        return 0
                    fi
                    
                    # Parse --from and --to flags if present
                    local args=("$@")
                    for i in "${!args[@]}"; do
                        if [[ "${args[$i]}" == "--from" ]]; then
                            source_dir="${args[$((i+1))]}"
                        elif [[ "${args[$i]}" == "--to" ]]; then
                            target_dir="${args[$((i+1))]}"
                        fi
                    done
                    
                    if [[ -z "$source_dir" || -z "$target_dir" ]]; then
                        _log_error "Source and target directories required"
                        _log_info "Usage: cm organize extract <uuid> --from <source> --to <target>"
                        return 1
                    fi
                    
                    # Validate UUID format (basic check)
                    if [[ ! "$uuid" =~ ^[a-f0-9-]{36}$ ]]; then
                        _log_error "Invalid UUID format: $uuid"
                        _log_info "Expected format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                        return 1
                    fi
                    
                    # Resolve paths
                    source_dir=$(_resolve_absolute_path "$(pwd)" "$source_dir")
                    target_dir=$(_resolve_absolute_path "$(pwd)" "$target_dir")
                    
                    # Normalize paths
                    source_dir="${source_dir%/}"
                    target_dir="${target_dir%/}"
                    
                    # Find source project
                    local source_project
                    source_project=$(_suggest_project_dir_for "$source_dir")
                    
                    if [[ ! -d "$source_project" ]]; then
                        _log_error "Source project not found for: $source_dir"
                        _log_info "Expected project: $source_project"
                        return 1
                    fi
                    
                    # Find session file
                    local session_file="$source_project/${uuid}.jsonl"
                    if [[ ! -f "$session_file" ]]; then
                        _log_error "Session not found: $uuid"
                        _log_info "Looking in: $source_project"
                        
                        # Try to help find similar UUIDs
                        _log_info "Available sessions in project:"
                        local found_any=false
                        while IFS= read -r f; do
                            local base
                            base=$(basename "$f" .jsonl)
                            if [[ "$base" == *"${uuid:0:8}"* ]]; then
                                echo "  $base"
                                found_any=true
                            fi
                        done < <(find "$source_project" -name "*.jsonl" -type f | head -20)
                        
                        if [[ "$found_any" == "false" ]]; then
                            _log_info "  (showing first few)"
                            find "$source_project" -name "*.jsonl" -type f | head -5 | while read -r f; do
                                echo "  $(basename "$f" .jsonl)"
                            done
                        fi
                        return 1
                    fi
                    
                    # Check session content references source_dir
                    local ref_count
                    ref_count=$(grep -c "\"cwd\":\"$source_dir\"" "$session_file" 2>/dev/null || echo 0)
                    
                    if [[ "$ref_count" -eq 0 ]]; then
                        _log_warn "Session $uuid doesn't reference $source_dir"
                        
                        # Show what paths it does reference
                        local actual_paths
                        actual_paths=$(grep -o '"cwd":"[^"]*"' "$session_file" 2>/dev/null | \
                                      sed 's/"cwd":"\([^"]*\)"/\1/' | sort -u | head -3)
                        if [[ -n "$actual_paths" ]]; then
                            _log_info "Session references these paths instead:"
                            echo "$actual_paths" | while read -r p; do
                                echo "  $p"
                            done
                        fi
                        
                        if [[ "$INTERACTIVE" == "true" ]]; then
                            if ! _confirm "Extract anyway?"; then
                                _log_info "Extraction cancelled"
                                return 1
                            fi
                        else
                            _log_error "Session doesn't match source directory (use INTERACTIVE=true to override)"
                            return 1
                        fi
                    else
                        _log_info "Session has $ref_count references to $source_dir"
                    fi
                    
                    # Prepare target
                    local target_project
                    target_project=$(_suggest_project_dir_for "$target_dir")
                    
                    _log_info "=== Extract Operation Plan ==="
                    _log_info "Session:        $uuid"
                    _log_info "Source dir:     $source_dir"  
                    _log_info "Target dir:     $target_dir"
                    _log_info "Source project: $(basename "$source_project")"
                    _log_info "Target project: $(basename "$target_project")"
                    
                    # Check for conflicts
                    local target_session="$target_project/${uuid}.jsonl"
                    if [[ -f "$target_session" ]]; then
                        _log_warn "Session already exists in target"
                        
                        local source_size target_size
                        source_size=$(wc -c < "$session_file")
                        target_size=$(wc -c < "$target_session")
                        
                        _log_info "Source size: $source_size bytes"
                        _log_info "Target size: $target_size bytes"
                        
                        if [[ "$source_size" -eq "$target_size" ]]; then
                            _log_info "Files appear identical (same size)"
                        else
                            _log_warn "Files differ in size"
                        fi
                        
                        if [[ "$INTERACTIVE" == "true" ]]; then
                            if ! _confirm "Overwrite existing session in target?"; then
                                _log_info "Extraction cancelled"
                                return 1
                            fi
                        else
                            _log_error "Target session exists (use INTERACTIVE=true to overwrite)"
                            return 1
                        fi
                    fi
                    
                    # Interactive confirmation (before dry run for better UX)
                    if [[ "$INTERACTIVE" == "true" && "$DRY_RUN" != "true" ]]; then
                        if ! _confirm "Proceed with extraction?"; then
                            _log_info "Extraction cancelled"
                            return 1
                        fi
                    fi
                    
                    # Dry run check
                    if [[ "$DRY_RUN" == "true" ]]; then
                        _log_info "=== DRY RUN - No changes made ==="
                        _log_debug "Would create: $target_project (if needed)"
                        _log_debug "Would copy: $session_file -> $target_session"
                        _log_debug "Would update $ref_count path references"
                        _log_debug "Would remove: $session_file"
                        return 0
                    fi
                    
                    # === TRANSACTION START ===
                    _log_info "=== Executing Extraction ==="
                    
                    # Create target project if needed
                    if [[ ! -d "$target_project" ]]; then
                        _log_info "Creating target project directory..."
                        if ! mkdir -p "$target_project"; then
                            _log_error "Failed to create target project: $target_project"
                            return 1
                        fi
                    fi
                    
                    # Copy and transform session
                    _log_info "Copying session to target..."
                    if ! cp "$session_file" "${session_file}.extract-backup"; then
                        _log_error "Failed to backup session"
                        return 1
                    fi
                    
                    if ! cp "$session_file" "$target_session"; then
                        _log_error "Failed to copy session"
                        rm -f "${session_file}.extract-backup"
                        return 1
                    fi
                    
                    # Update paths in target
                    if [[ "$ref_count" -gt 0 ]]; then
                        _log_info "Updating $ref_count path references..."
                        local old_esc new_esc
                        old_esc=$(printf '%s' "$source_dir" | sed -e 's/[\/&]/\\&/g')
                        new_esc=$(printf '%s' "$target_dir" | sed -e 's/[\/&]/\\&/g')
                        
                        if [[ "$OSTYPE" == "darwin"* ]]; then
                            sed -i '' "s|\"cwd\":\"$old_esc\"|\"cwd\":\"$new_esc\"|g" "$target_session"
                        else
                            sed -i "s|\"cwd\":\"$old_esc\"|\"cwd\":\"$new_esc\"|g" "$target_session"
                        fi
                        
                        # Verify update
                        local updated_count
                        updated_count=$(grep -c "\"cwd\":\"$target_dir\"" "$target_session" 2>/dev/null || echo 0)
                        
                        if [[ "$updated_count" -ne "$ref_count" ]]; then
                            _log_warn "Path update mismatch: expected $ref_count, got $updated_count"
                        else
                            _log_success "Successfully updated all path references"
                        fi
                    fi
                    
                    # Remove from source
                    _log_info "Removing session from source..."
                    if ! rm "$session_file"; then
                        _log_error "Failed to remove source session"
                        _log_info "Manual cleanup required: $session_file"
                    else
                        rm -f "${session_file}.extract-backup"
                    fi
                    
                    # === TRANSACTION END ===
                    
                    _log_success "=== Extraction Complete ==="
                    _log_info "Session $uuid moved from:"
                    _log_info "  $source_dir"  
                    _log_info "to:"
                    _log_info "  $target_dir"
                    
                    # Report final state
                    local remaining_sessions
                    remaining_sessions=$(find "$source_project" -name "*.jsonl" -type f | wc -l)
                    _log_info "Source project has $remaining_sessions sessions remaining"
                    
                    if [[ "$remaining_sessions" -eq 0 ]]; then
                        _log_info "Source project is now empty - consider removing it"
                    fi
                    ;;
                    
                *)
                    _log_info "Usage: cm organize <subcommand>"
                    _log_info ""
                    _log_info "Subcommands:"
                    _log_info "  extract <uuid> --from <source> --to <target>"
                    _log_info "    Extract single session by UUID to another directory"
                    _log_info ""
                    _log_info "Aliases: extract -> e"
                    _log_info ""
                    _log_info "Future subcommands (not yet implemented):"
                    _log_info "  filter <criteria> --from <source> --to <target>"
                    _log_info "  split <source> --by <date|size|pattern>"
                    _log_info "  merge <source1> <source2> --to <target>"
                    ;;
            esac
            ;;
            
        "health"|"doctor")
            # System health check
            _log_info "=== Claude Manager Health Check ==="
            
            local health_issues=0
            
            # Check Claude directory
            if [[ -d "$CLAUDE_DIR" ]]; then
                _log_success "✓ Claude directory exists: $CLAUDE_DIR"
                
                if [[ -d "$CLAUDE_DIR/projects" ]]; then
                    local project_count
                    project_count=$(find "$CLAUDE_DIR/projects" -type d -mindepth 1 -maxdepth 1 | wc -l)
                    _log_success "✓ Projects directory exists with $project_count projects"
                else
                    _log_error "✗ Projects directory missing: $CLAUDE_DIR/projects"
                    ((health_issues++))
                fi
                
                if [[ -w "$CLAUDE_DIR" ]]; then
                    _log_success "✓ Claude directory is writable"
                else
                    _log_error "✗ Claude directory is not writable: $CLAUDE_DIR"
                    ((health_issues++))
                fi
            else
                _log_error "✗ Claude directory not found: $CLAUDE_DIR"
                ((health_issues++))
            fi
            
            # Check required tools
            local tools=("sed" "grep" "find" "mv" "cp")
            for tool in "${tools[@]}"; do
                if command -v "$tool" >/dev/null 2>&1; then
                    _log_success "✓ Required tool available: $tool"
                else
                    _log_error "✗ Required tool missing: $tool"
                    ((health_issues++))
                fi
            done
            
            # Check optional tools
            local optional_tools=("realpath" "pgrep")
            for tool in "${optional_tools[@]}"; do
                if command -v "$tool" >/dev/null 2>&1; then
                    _log_success "✓ Optional tool available: $tool"
                else
                    _log_warn "⚠ Optional tool missing: $tool (functionality may be limited)"
                fi
            done
            
            # Check platform-specific tools
            if [[ "$OSTYPE" == "darwin"* ]]; then
                if command -v gsed >/dev/null 2>&1; then
                    _log_success "✓ GNU sed available as gsed"
                fi
                if command -v gtimeout >/dev/null 2>&1; then
                    _log_success "✓ GNU timeout available as gtimeout"
                fi
            fi
            
            # Check permissions on common paths
            if [[ -f "$HOME/.bash_profile" ]] && [[ -w "$HOME/.bash_profile" ]]; then
                _log_success "✓ Shell profile writable: ~/.bash_profile"
            fi
            if [[ -f "$HOME/.zshrc" ]] && [[ -w "$HOME/.zshrc" ]]; then
                _log_success "✓ Shell profile writable: ~/.zshrc"
            fi
            
            # Check for active Claude processes
            local claude_procs
            claude_procs=$(pgrep -f "[Cc]laude" 2>/dev/null | wc -l) || claude_procs=0
            if [[ "$claude_procs" -gt 0 ]]; then
                _log_warn "⚠ $claude_procs Claude processes currently running"
                _log_info "  Consider closing Claude before major operations"
            else
                _log_success "✓ No active Claude processes detected"
            fi
            
            # Summary
            _log_info "=== Health Check Summary ==="
            if [[ "$health_issues" -eq 0 ]]; then
                _log_success "✅ System health: GOOD ($health_issues issues)"
                _log_info "Claude Manager is ready for use"
            elif [[ "$health_issues" -le 2 ]]; then
                _log_warn "⚠️ System health: WARNING ($health_issues issues)"
                _log_info "Minor issues detected - functionality may be limited"
            else
                _log_error "❌ System health: CRITICAL ($health_issues issues)"
                _log_info "Major issues detected - Claude Manager may not function properly"
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
            echo "Claude Manager - Usage:"
            echo ""
            echo "Commands:"
            echo "  migrate <old_path> <new_path> [project_dir]"
            echo "    Update session paths after moving/renaming source directory"
            echo ""
            echo "  move <old_path> <new_path>"
            echo "    Robust move with edge case handling, pre-flight checks, and rollback"
            echo ""
            echo "  organize <subcommand>"
            echo "    Granular session organization:"
            echo "    • extract <uuid> --from <source> --to <target>"
            echo "      Extract single session by UUID to another directory"
            echo ""
            echo "  full [new_path]"
            echo "    Interactive helper - run from source directory to move"
            echo ""
            echo "  list [project_dir]"
            echo "    List all projects or sessions within a specific project"
            echo ""
            echo "  verify <project_dir>"
            echo "    Check project for path consistency and report mismatches"
            echo ""
            echo "  health"
            echo "    System health check - validate tools, permissions, and setup"
            echo ""
            echo "  config"
            echo "    Display current configuration values"
            echo ""
            echo "  undo"
            echo "    Undo the last move operation"
            echo ""
            echo "Aliases: cm, cm-migrate, cm-move, cm-full, cm-list"
            echo ""
            echo "Configuration (Environment Variables):"
            echo "  CLAUDE_DIR - Claude directory (default: ~/.claude)"
            echo "  CLAUDE_BACKUP_STRATEGY - file|project (default: file)"
            echo "  CLAUDE_INTERACTIVE - true|false (default: true)"
            echo "  CLAUDE_DRY_RUN - true|false (default: false)"
            echo "  FORCE - true|false (override safety checks, default: false)"
            echo ""
            echo "Examples:"
            echo '  cm migrate "/Users/old/project" "/Users/new/project"'
            echo '  cm move "/old/src" "/new/src"  # New robust move'
            echo '  CLAUDE_DRY_RUN=true cm move "/old" "/new"  # Preview only'
            echo '  cm verify ~/.claude/projects/-Users-name-project'
            echo '  cm health  # Check system status'
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

# Execute main function if script is run directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    claude_manager "$@"
fi
