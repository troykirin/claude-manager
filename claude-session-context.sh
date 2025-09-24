#!/usr/bin/env bash

# Claude Context-Sensitive Session Management
# Extension to claude-manager for intelligent session continuation

# Don't use set -e in sourced scripts - it will exit the shell!

# Configuration
CLAUDE_DIR="${CLAUDE_DIR:-$HOME/.claude}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
_log_info() { echo -e "${BLUE}[INFO] $1${NC}" >&2; }
_log_warn() { echo -e "${YELLOW}[WARN] $1${NC}" >&2; }
_log_error() { echo -e "${RED}[ERROR] $1${NC}" >&2; }
_log_success() { echo -e "${GREEN}[SUCCESS] $1${NC}" >&2; }

# Context-sensitive session selection
# Returns the best session ID for the current directory
cs_find_best_session() {
    local cwd="$1"
    [[ -z "$cwd" ]] && cwd="$(pwd)"
    
    # Find all sessions and score them
    local best_session=""
    local best_score=0
    local session_count=0
    
    # Debug mode
    local debug="${CS_DEBUG:-false}"
    
    [[ "$debug" == "true" ]] && _log_info "Searching for sessions matching: $cwd"
    
    # Search all project directories
    if [[ -d "$CLAUDE_DIR/projects" ]]; then
        while IFS= read -r project_dir; do
            [[ ! -d "$project_dir" ]] && continue
            
            # Look for session files in this project
            while IFS= read -r session_file; do
                [[ ! -f "$session_file" ]] && continue
                
                session_count=$((session_count + 1))
                
                # Extract session ID
                local session_id
                session_id=$(basename "$session_file" .jsonl)
                
                # Score this session based on multiple factors
                local score=0
                local session_cwd=""
                local last_modified=""
                
                # Extract working directory from session
                session_cwd=$(grep -m 1 '"cwd":' "$session_file" 2>/dev/null | \
                    sed 's/.*"cwd":"\([^"]*\)".*/\1/' || echo "")
                
                # Get last modified time (for recency scoring)
                if [[ "$(uname)" == "Darwin" ]]; then
                    last_modified=$(stat -f %m "$session_file" 2>/dev/null || echo "0")
                else
                    last_modified=$(stat -c %Y "$session_file" 2>/dev/null || echo "0")
                fi
                
                # Calculate scores
                local path_score=0
                local recency_score=0
                local depth_score=0
                
                # 1. Path matching score (most important)
                if [[ "$session_cwd" == "$cwd" ]]; then
                    path_score=100  # Exact match
                elif [[ "$cwd" == "$session_cwd"* ]]; then
                    path_score=80   # Session is parent of current dir
                elif [[ "$session_cwd" == "$cwd"* ]]; then
                    path_score=60   # Session is child of current dir
                elif [[ "$(dirname "$session_cwd")" == "$(dirname "$cwd")" ]]; then
                    path_score=40   # Same parent directory (sibling)
                fi
                
                # 2. Recency score (0-50 points based on age)
                local now
                now=$(date +%s)
                local age=$((now - last_modified))
                if [[ $age -lt 3600 ]]; then
                    recency_score=50  # Less than 1 hour old
                elif [[ $age -lt 86400 ]]; then
                    recency_score=40  # Less than 1 day old
                elif [[ $age -lt 604800 ]]; then
                    recency_score=20  # Less than 1 week old
                else
                    recency_score=5   # Older
                fi
                
                # 3. Session depth score (number of messages)
                local message_count
                message_count=$(grep -c '"role":' "$session_file" 2>/dev/null || echo "0")
                if [[ $message_count -gt 20 ]]; then
                    depth_score=30
                elif [[ $message_count -gt 10 ]]; then
                    depth_score=20
                elif [[ $message_count -gt 5 ]]; then
                    depth_score=10
                else
                    depth_score=5
                fi
                
                # Total score with weights
                score=$((path_score * 2 + recency_score + depth_score))
                
                if [[ "$debug" == "true" ]]; then
                    _log_info "Session: $session_id"
                    echo "  Path: $session_cwd (score: $path_score)"
                    echo "  Age: $((age / 3600)) hours (score: $recency_score)"
                    echo "  Messages: $message_count (score: $depth_score)"
                    echo "  Total Score: $score"
                fi
                
                # Update best session if this scores higher
                if [[ $score -gt $best_score ]]; then
                    best_score=$score
                    best_session="$session_id"
                fi
                
            done < <(find "$project_dir" -name "*.jsonl" -type f 2>/dev/null)
        done < <(find "$CLAUDE_DIR/projects" -type d -mindepth 1 -maxdepth 1 2>/dev/null)
    fi
    
    if [[ "$debug" == "true" ]]; then
        _log_info "Evaluated $session_count sessions"
        if [[ -n "$best_session" ]]; then
            _log_success "Best match: $best_session (score: $best_score)"
        fi
    fi
    
    echo "$best_session"
}

# Main cs command - Context-sensitive continue
cs() {
    local cwd="$(pwd)"
    local force_new=false
    local list_only=false
    local debug=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --new|-n)
                force_new=true
                shift
                ;;
            --list|-l)
                list_only=true
                shift
                ;;
            --debug|-d)
                debug=true
                export CS_DEBUG=true
                shift
                ;;
            --help|-h)
                cat <<EOF
Claude Context-Sensitive Session Management

Usage: cs [OPTIONS]

Options:
  --new, -n     Force new session instead of continuing
  --list, -l    List sessions for current directory
  --debug, -d   Show detailed scoring information
  --help, -h    Show this help message

Environment:
  CS_DEBUG      Enable debug output (true/false)
  CS_THRESHOLD  Minimum score to auto-continue (default: 100)

Examples:
  cs            # Continue best matching session or create new
  cs --new      # Force new session in current directory
  cs --list     # Show all sessions for current directory
  cs --debug    # Show scoring details for session selection
EOF
                return 0
                ;;
            *)
                _log_error "Unknown option: $1"
                return 1
                ;;
        esac
    done
    
    # List mode - show all sessions for current directory
    if [[ "$list_only" == "true" ]]; then
        _log_info "Sessions for: $cwd"
        echo ""
        
        local found_any=false
        if [[ -d "$CLAUDE_DIR/projects" ]]; then
            while IFS= read -r session_file; do
                [[ ! -f "$session_file" ]] && continue
                
                local session_cwd
                session_cwd=$(grep -m 1 '"cwd":' "$session_file" 2>/dev/null | \
                    sed 's/.*"cwd":"\([^"]*\)".*/\1/' || echo "")
                
                if [[ "$session_cwd" == "$cwd" ]]; then
                    found_any=true
                    local session_id
                    session_id=$(basename "$session_file" .jsonl)
                    
                    local message_count
                    message_count=$(grep -c '"role":' "$session_file" 2>/dev/null || echo "0")
                    
                    local last_modified_display=""
                    if [[ "$(uname)" == "Darwin" ]]; then
                        last_modified_display=$(stat -f "%Sm" -t "%Y-%m-%d %H:%M" "$session_file" 2>/dev/null || echo "unknown")
                    else
                        last_modified_display=$(stat -c "%y" "$session_file" 2>/dev/null | cut -d' ' -f1-2 || echo "unknown")
                    fi
                    
                    echo -e "${CYAN}$session_id${NC}"
                    echo "  Messages: $message_count"
                    echo "  Modified: $last_modified_display"
                    echo ""
                fi
            done < <(find "$CLAUDE_DIR/projects" -name "*.jsonl" -type f 2>/dev/null)
        fi
        
        if [[ "$found_any" == "false" ]]; then
            _log_warn "No sessions found for current directory"
        fi
        
        return 0
    fi
    
    # Force new session mode
    if [[ "$force_new" == "true" ]]; then
        _log_info "Starting new session in: $cwd"
        claude
        return $?
    fi
    
    # Find best matching session
    local best_session
    best_session=$(cs_find_best_session "$cwd")
    
    # Threshold for auto-continue (default 100 = exact path match)
    local threshold="${CS_THRESHOLD:-100}"
    
    if [[ -n "$best_session" ]]; then
        _log_success "Continuing session: $best_session"
        _log_info "Directory: $cwd"
        echo ""
        
        # Use claude --continue with specific session ID
        claude --continue -r "$best_session"
    else
        _log_warn "No matching session found for: $cwd"
        _log_info "Starting new session..."
        echo ""
        
        # Start new session
        claude
    fi
}

# Additional utility commands
cs-status() {
    local cwd="$(pwd)"
    _log_info "Current directory: $cwd"
    
    local best_session
    best_session=$(cs_find_best_session "$cwd")
    
    if [[ -n "$best_session" ]]; then
        _log_success "Active session: $best_session"
    else
        _log_warn "No session bound to this directory"
    fi
}

cs-cleanup() {
    _log_info "Cleaning up stale sessions..."
    
    # Find sessions older than 30 days
    local cleaned=0
    local cutoff_time=$(($(date +%s) - 2592000))  # 30 days ago
    
    if [[ -d "$CLAUDE_DIR/projects" ]]; then
        while IFS= read -r session_file; do
            [[ ! -f "$session_file" ]] && continue
            
            local last_modified
            if [[ "$(uname)" == "Darwin" ]]; then
                last_modified=$(stat -f %m "$session_file" 2>/dev/null || echo "0")
            else
                last_modified=$(stat -c %Y "$session_file" 2>/dev/null || echo "0")
            fi
            
            if [[ $last_modified -lt $cutoff_time ]]; then
                local session_id
                session_id=$(basename "$session_file" .jsonl)
                _log_warn "Archiving stale session: $session_id"
                
                # Move to archive directory instead of deleting
                local archive_dir="$CLAUDE_DIR/archived_sessions"
                mkdir -p "$archive_dir"
                mv "$session_file" "$archive_dir/" 2>/dev/null && cleaned=$((cleaned + 1))
            fi
        done < <(find "$CLAUDE_DIR/projects" -name "*.jsonl" -type f 2>/dev/null)
    fi
    
    _log_success "Archived $cleaned stale sessions"
}

# Functions are automatically available when sourced in zsh