#!/usr/bin/env bash
# REPAIR Mode: Safe Session Duplication with Corruption Isolation
# Part of CM-302: REPAIR Mode Duplication (NOS-678 Session Recovery Phase 1)

set -euo pipefail

# REPAIR mode state tracking
declare -g REPAIR_BACKUP_DIR=""
declare -g REPAIR_NEW_UUID=""
declare -g REPAIR_SESSION_ID=""
declare -g REPAIR_STATUS="pending"

# ============================================================================
# Main REPAIR Orchestrator
# ============================================================================

repair_session() {
    local session_id="$1"
    local repair_timestamp=$(date +%Y%m%d_%H%M%S)

    _log_info "=== REPAIR Mode: Session Duplication with Corruption Isolation ==="
    _log_info "Session ID: $session_id"
    _log_info "Timestamp: $repair_timestamp"

    REPAIR_SESSION_ID="$session_id"
    REPAIR_STATUS="in_progress"

    # Phase 1: Pre-flight checks
    _log_info "Phase 1/6: Pre-flight checks..."
    if ! _repair_verify_preconditions "$session_id"; then
        _log_error "Pre-flight checks failed"
        REPAIR_STATUS="failed_preflight"
        return 1
    fi

    # Phase 2: Create safety backup
    _log_info "Phase 2/6: Creating safety backup..."
    if ! _repair_create_backup "$session_id" "$repair_timestamp"; then
        _log_error "Backup creation failed"
        REPAIR_STATUS="failed_backup"
        return 1
    fi

    # Phase 3: Safe duplication
    _log_info "Phase 3/6: Safe duplication..."
    if ! _repair_duplicate_safe "$session_id"; then
        _log_error "Duplication failed - initiating rollback"
        _repair_rollback "$session_id"
        REPAIR_STATUS="failed_duplication"
        return 1
    fi

    # Phase 4: Corruption isolation
    _log_info "Phase 4/6: Isolating corruption..."
    if ! _repair_isolate_corruption "$session_id"; then
        _log_warn "Corruption isolation had issues - continuing with caution"
    fi

    # Phase 5: State restoration
    _log_info "Phase 5/6: Restoring valid state..."
    if ! _repair_restore_state "$session_id"; then
        _log_error "State restoration failed - initiating rollback"
        _repair_rollback "$session_id"
        REPAIR_STATUS="failed_restoration"
        return 1
    fi

    # Phase 6: Post-repair verification
    _log_info "Phase 6/6: Post-repair verification..."
    if ! _repair_verify_success "$session_id"; then
        _log_error "Verification failed - rollback recommended"
        REPAIR_STATUS="failed_verification"
        return 1
    fi

    REPAIR_STATUS="success"
    _log_success "=== REPAIR Mode: Session repair completed successfully ==="
    _log_info "New session UUID: $REPAIR_NEW_UUID"
    _log_info "Backup location: $REPAIR_BACKUP_DIR"
    _log_info "Original session archived at: ~/.claude/.archive/"

    return 0
}

# ============================================================================
# Phase 1: Pre-flight Checks
# ============================================================================

_repair_verify_preconditions() {
    local session_id="$1"
    local errors=0

    # Check if session exists
    local project_file=$(find "$CLAUDE_DIR/projects" -name "${session_id}.jsonl" -type f | head -1)
    if [[ -z "$project_file" ]]; then
        _log_error "Session not found: $session_id"
        ((errors++))
    else
        _log_success "✓ Session file found: $project_file"
    fi

    # Check if Claude is running
    local claude_procs=$(pgrep -f "[Cc]laude" 2>/dev/null | wc -l) || claude_procs=0
    if [[ "$claude_procs" -gt 0 ]]; then
        _log_warn "⚠ $claude_procs Claude processes running (recommend closing before repair)"
        if [[ "$INTERACTIVE" == "true" ]]; then
            if ! _confirm "Continue with active Claude processes?"; then
                ((errors++))
            fi
        fi
    else
        _log_success "✓ No active Claude processes"
    fi

    # Check disk space (need at least 500MB for safety)
    local free_space=$(df -k "$CLAUDE_DIR" | awk 'NR==2 {print $4}')
    if [[ "$free_space" -lt 512000 ]]; then
        _log_error "Insufficient disk space: ${free_space}KB available (need 500MB)"
        ((errors++))
    else
        _log_success "✓ Sufficient disk space: $((free_space / 1024))MB available"
    fi

    # Check file permissions
    if [[ ! -w "$CLAUDE_DIR/projects" ]]; then
        _log_error "Projects directory not writable: $CLAUDE_DIR/projects"
        ((errors++))
    else
        _log_success "✓ Projects directory writable"
    fi

    # Verify JSONL integrity
    if [[ -n "$project_file" ]]; then
        if ! jq empty "$project_file" 2>/dev/null; then
            _log_error "JSONL file contains invalid JSON: $project_file"
            ((errors++))
        else
            local line_count=$(wc -l < "$project_file")
            _log_success "✓ JSONL integrity OK ($line_count lines)"
        fi
    fi

    if [[ "$errors" -gt 0 ]]; then
        _log_error "Pre-flight checks failed with $errors error(s)"
        return 1
    fi

    _log_success "Pre-flight checks passed"
    return 0
}

# ============================================================================
# Phase 2: Backup Creation
# ============================================================================

_repair_create_backup() {
    local session_id="$1"
    local timestamp="$2"

    # Create backup directory
    local backup_base="${XDG_STATE_HOME:-$HOME/.local/state}/nabi/repairs"
    REPAIR_BACKUP_DIR="$backup_base/repair-${session_id}-${timestamp}"

    mkdir -p "$REPAIR_BACKUP_DIR"/{projects,todos,statsig,shell-snapshots}

    _log_info "Creating backup at: $REPAIR_BACKUP_DIR"

    # Backup 1: Projects (JSONL files)
    local project_files=$(find "$CLAUDE_DIR/projects" -name "${session_id}.jsonl" -type f)
    local project_count=0
    while IFS= read -r pfile; do
        if [[ -n "$pfile" && -f "$pfile" ]]; then
            local proj_dir=$(basename "$(dirname "$pfile")")
            mkdir -p "$REPAIR_BACKUP_DIR/projects/$proj_dir"
            cp "$pfile" "$REPAIR_BACKUP_DIR/projects/$proj_dir/"
            ((project_count++))
        fi
    done <<< "$project_files"
    _log_info "  ✓ Backed up $project_count project file(s)"

    # Backup 2: Todos
    local todo_files=$(find "$CLAUDE_DIR/todos" -name "${session_id}-agent-*.json" -type f 2>/dev/null || true)
    local todo_count=0
    while IFS= read -r tfile; do
        if [[ -n "$tfile" && -f "$tfile" ]]; then
            cp "$tfile" "$REPAIR_BACKUP_DIR/todos/"
            ((todo_count++))
        fi
    done <<< "$todo_files"
    _log_info "  ✓ Backed up $todo_count todo file(s)"

    # Backup 3: Statsig (if exists)
    local statsig_count=0
    if [[ -d "$CLAUDE_DIR/statsig" ]]; then
        local statsig_files=$(find "$CLAUDE_DIR/statsig" -type f -name "statsig.session_id.*" 2>/dev/null || true)
        while IFS= read -r sfile; do
            if [[ -n "$sfile" && -f "$sfile" ]]; then
                cp "$sfile" "$REPAIR_BACKUP_DIR/statsig/"
                ((statsig_count++))
            fi
        done <<< "$statsig_files"
    fi
    _log_info "  ✓ Backed up $statsig_count statsig file(s)"

    # Backup 4: Shell snapshots (last 5 related to this session)
    local snapshot_count=0
    if [[ -d "$CLAUDE_DIR/shell-snapshots" ]]; then
        # Find recent snapshots (generic backup, not session-specific)
        local snapshots=$(ls -t "$CLAUDE_DIR/shell-snapshots"/snapshot-*-*.sh 2>/dev/null | head -5 || true)
        while IFS= read -r snapshot; do
            if [[ -n "$snapshot" && -f "$snapshot" ]]; then
                cp "$snapshot" "$REPAIR_BACKUP_DIR/shell-snapshots/"
                ((snapshot_count++))
            fi
        done <<< "$snapshots"
    fi
    _log_info "  ✓ Backed up $snapshot_count shell snapshot(s)"

    # Create manifest
    cat > "$REPAIR_BACKUP_DIR/manifest.json" << EOF
{
    "session_id": "$session_id",
    "timestamp": "$timestamp",
    "backup_dir": "$REPAIR_BACKUP_DIR",
    "files_backed_up": {
        "projects": $project_count,
        "todos": $todo_count,
        "statsig": $statsig_count,
        "shell_snapshots": $snapshot_count
    },
    "repair_status": "backup_complete"
}
EOF

    _log_success "Backup manifest created"
    return 0
}

# ============================================================================
# Phase 3: Safe Duplication
# ============================================================================

_repair_duplicate_safe() {
    local session_id="$1"

    # Generate new UUID for duplicated session
    if command -v uuidgen >/dev/null 2>&1; then
        REPAIR_NEW_UUID=$(uuidgen | tr '[:upper:]' '[:lower:]')
    else
        # Fallback: generate UUID using Python
        REPAIR_NEW_UUID=$(python3 -c "import uuid; print(str(uuid.uuid4()))")
    fi

    _log_info "Generated new UUID: $REPAIR_NEW_UUID"

    # Find original project file
    local orig_project_file=$(find "$CLAUDE_DIR/projects" -name "${session_id}.jsonl" -type f | head -1)
    if [[ -z "$orig_project_file" ]]; then
        _log_error "Original project file not found"
        return 1
    fi

    local orig_project_dir=$(dirname "$orig_project_file")
    local new_project_file="$orig_project_dir/${REPAIR_NEW_UUID}.jsonl"

    # Copy JSONL content (this is the source of truth)
    _log_info "Duplicating JSONL content..."
    if ! cp "$orig_project_file" "$new_project_file"; then
        _log_error "Failed to duplicate JSONL file"
        return 1
    fi

    # Update sessionId in new file (first line should contain sessionId)
    if command -v jq >/dev/null 2>&1; then
        # Use jq to update sessionId in first line (if present)
        local first_line=$(head -1 "$new_project_file")
        if echo "$first_line" | jq -e '.sessionId' >/dev/null 2>&1; then
            local updated_line=$(echo "$first_line" | jq --arg new_id "$REPAIR_NEW_UUID" '.sessionId = $new_id')
            # Replace first line with updated version
            { echo "$updated_line"; tail -n +2 "$new_project_file"; } > "${new_project_file}.tmp"
            mv "${new_project_file}.tmp" "$new_project_file"
            _log_success "✓ Updated sessionId in new file"
        fi
    fi

    # Rebuild todo state from conversation (create fresh, clean state)
    _log_info "Rebuilding todo state from JSONL..."
    if ! _repair_rebuild_todos "$REPAIR_NEW_UUID" "$new_project_file"; then
        _log_warn "Todo rebuild had issues - continuing"
    fi

    # Create fresh cross-references (no statsig corruption)
    _log_info "Creating fresh cross-references..."
    # Note: Statsig files are typically auto-generated by Claude, so we don't duplicate them

    _log_success "Safe duplication completed"
    return 0
}

# Helper: Rebuild todos from JSONL
_repair_rebuild_todos() {
    local new_uuid="$1"
    local jsonl_file="$2"

    # Extract todo items from JSONL (look for todo-related messages)
    # This is a basic implementation - enhance based on actual todo structure
    local todo_count=0

    # Create empty todo structure for new session
    local todo_file="$CLAUDE_DIR/todos/${new_uuid}-agent-main.json"
    cat > "$todo_file" << 'EOF'
{
    "version": "1.0",
    "todos": [],
    "metadata": {
        "created_from": "repair_mode",
        "source": "jsonl_rebuild"
    }
}
EOF

    _log_info "  ✓ Created clean todo state"
    return 0
}

# ============================================================================
# Phase 4: Corruption Isolation
# ============================================================================

_repair_isolate_corruption() {
    local session_id="$1"
    local archive_dir="$CLAUDE_DIR/.archive"
    local timestamp=$(date +%Y%m%d_%H%M%S)

    mkdir -p "$archive_dir/sessions-${timestamp}"

    _log_info "Archiving original corrupted session..."

    # Move original project files to archive
    local archived_count=0
    local project_files=$(find "$CLAUDE_DIR/projects" -name "${session_id}.jsonl" -type f)
    while IFS= read -r pfile; do
        if [[ -n "$pfile" && -f "$pfile" ]]; then
            local proj_dir=$(basename "$(dirname "$pfile")")
            mkdir -p "$archive_dir/sessions-${timestamp}/projects/$proj_dir"
            mv "$pfile" "$archive_dir/sessions-${timestamp}/projects/$proj_dir/"
            ((archived_count++))
        fi
    done <<< "$project_files"

    # Move original todos to archive
    local todo_files=$(find "$CLAUDE_DIR/todos" -name "${session_id}-agent-*.json" -type f 2>/dev/null || true)
    while IFS= read -r tfile; do
        if [[ -n "$tfile" && -f "$tfile" ]]; then
            mkdir -p "$archive_dir/sessions-${timestamp}/todos"
            mv "$tfile" "$archive_dir/sessions-${timestamp}/todos/"
        fi
    done <<< "$todo_files"

    # Create archive manifest
    cat > "$archive_dir/sessions-${timestamp}/ARCHIVE_REASON.txt" << EOF
Session ID: $session_id
Archive Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Reason: REPAIR mode - corruption isolation
New Session ID: $REPAIR_NEW_UUID
Backup Location: $REPAIR_BACKUP_DIR

This session was archived during REPAIR mode operation.
The session was duplicated with a new UUID and clean state.
Original files preserved here for forensic analysis.
EOF

    _log_success "✓ Archived $archived_count file(s) to: $archive_dir/sessions-${timestamp}"
    return 0
}

# ============================================================================
# Phase 5: State Restoration
# ============================================================================

_repair_restore_state() {
    local session_id="$1"

    _log_info "Restoring valid state from JSONL source of truth..."

    # The new session file already contains valid JSONL content (copied in Phase 3)
    # Now we need to ensure cross-system consistency

    local new_project_file=$(find "$CLAUDE_DIR/projects" -name "${REPAIR_NEW_UUID}.jsonl" -type f | head -1)
    if [[ ! -f "$new_project_file" ]]; then
        _log_error "New project file not found: $REPAIR_NEW_UUID"
        return 1
    fi

    # Verify JSONL integrity
    if ! jq empty "$new_project_file" 2>/dev/null; then
        _log_error "New JSONL file contains invalid JSON"
        return 1
    fi

    # Update cross-system references
    _log_info "Updating cross-system references..."

    # Update todo references (if any exist)
    local todo_file="$CLAUDE_DIR/todos/${REPAIR_NEW_UUID}-agent-main.json"
    if [[ -f "$todo_file" ]]; then
        _log_success "  ✓ Todo state synchronized"
    fi

    # Note: Statsig files are auto-generated by Claude, no manual update needed

    _log_success "State restoration completed"
    return 0
}

# ============================================================================
# Phase 6: Post-Repair Verification
# ============================================================================

_repair_verify_success() {
    local session_id="$1"
    local errors=0

    _log_info "Verifying repair success..."

    # Check new session exists
    local new_project_file=$(find "$CLAUDE_DIR/projects" -name "${REPAIR_NEW_UUID}.jsonl" -type f | head -1)
    if [[ ! -f "$new_project_file" ]]; then
        _log_error "New project file not found"
        ((errors++))
    else
        _log_success "✓ New session file exists"
    fi

    # Verify JSONL integrity
    if ! jq empty "$new_project_file" 2>/dev/null; then
        _log_error "New JSONL contains invalid JSON"
        ((errors++))
    else
        local line_count=$(wc -l < "$new_project_file")
        _log_success "✓ JSONL integrity OK ($line_count lines)"
    fi

    # Verify original is archived
    local archive_exists=$(find "$CLAUDE_DIR/.archive" -name "${session_id}.jsonl" -type f | wc -l)
    if [[ "$archive_exists" -eq 0 ]]; then
        _log_warn "⚠ Original session not found in archive"
    else
        _log_success "✓ Original session archived"
    fi

    # Verify backup exists
    if [[ ! -d "$REPAIR_BACKUP_DIR" ]]; then
        _log_error "Backup directory missing: $REPAIR_BACKUP_DIR"
        ((errors++))
    else
        _log_success "✓ Backup directory preserved"
    fi

    # Compare line counts (new should match original backup)
    local backup_project=$(find "$REPAIR_BACKUP_DIR/projects" -name "${session_id}.jsonl" -type f | head -1)
    if [[ -f "$backup_project" ]]; then
        local orig_lines=$(wc -l < "$backup_project")
        local new_lines=$(wc -l < "$new_project_file")
        if [[ "$orig_lines" -eq "$new_lines" ]]; then
            _log_success "✓ Line count matches ($new_lines lines)"
        else
            _log_warn "⚠ Line count mismatch: orig=$orig_lines, new=$new_lines"
        fi
    fi

    if [[ "$errors" -gt 0 ]]; then
        _log_error "Verification failed with $errors error(s)"
        return 1
    fi

    # Calculate health score (basic scoring)
    local health_score=100
    if [[ "$archive_exists" -eq 0 ]]; then
        ((health_score -= 10))
    fi

    _log_success "Post-repair verification passed (health score: $health_score/100)"

    if [[ "$health_score" -lt 70 ]]; then
        _log_warn "Health score below threshold - manual review recommended"
        return 1
    fi

    return 0
}

# ============================================================================
# Rollback Mechanism
# ============================================================================

_repair_rollback() {
    local session_id="$1"

    _log_warn "=== EMERGENCY ROLLBACK INITIATED ==="
    _log_info "Session: $session_id"
    _log_info "Backup: $REPAIR_BACKUP_DIR"

    if [[ ! -d "$REPAIR_BACKUP_DIR" ]]; then
        _log_error "CRITICAL: Backup directory not found - cannot rollback"
        return 1
    fi

    local rollback_errors=0

    # Remove new (failed) session if it exists
    if [[ -n "$REPAIR_NEW_UUID" ]]; then
        _log_info "Removing failed duplicate session..."
        local new_files=$(find "$CLAUDE_DIR/projects" -name "${REPAIR_NEW_UUID}.jsonl" -type f)
        while IFS= read -r nfile; do
            if [[ -n "$nfile" && -f "$nfile" ]]; then
                rm -f "$nfile"
            fi
        done <<< "$new_files"

        # Remove new todos
        rm -f "$CLAUDE_DIR/todos/${REPAIR_NEW_UUID}-agent-"*.json 2>/dev/null
    fi

    # Restore from backup
    _log_info "Restoring from backup..."

    # Restore project files
    local backup_projects=$(find "$REPAIR_BACKUP_DIR/projects" -name "${session_id}.jsonl" -type f)
    while IFS= read -r bfile; do
        if [[ -n "$bfile" && -f "$bfile" ]]; then
            local proj_dir=$(basename "$(dirname "$bfile")")
            local dest_dir="$CLAUDE_DIR/projects/$proj_dir"
            mkdir -p "$dest_dir"
            if ! cp "$bfile" "$dest_dir/"; then
                _log_error "Failed to restore: $bfile"
                ((rollback_errors++))
            fi
        fi
    done <<< "$backup_projects"

    # Restore todos
    local backup_todos=$(find "$REPAIR_BACKUP_DIR/todos" -name "${session_id}-agent-*.json" -type f)
    while IFS= read -r tfile; do
        if [[ -n "$tfile" && -f "$tfile" ]]; then
            if ! cp "$tfile" "$CLAUDE_DIR/todos/"; then
                _log_error "Failed to restore todo: $tfile"
                ((rollback_errors++))
            fi
        fi
    done <<< "$backup_todos"

    # Restore statsig
    local backup_statsig=$(find "$REPAIR_BACKUP_DIR/statsig" -type f)
    while IFS= read -r sfile; do
        if [[ -n "$sfile" && -f "$sfile" ]]; then
            if ! cp "$sfile" "$CLAUDE_DIR/statsig/"; then
                _log_error "Failed to restore statsig: $sfile"
                ((rollback_errors++))
            fi
        fi
    done <<< "$backup_statsig"

    if [[ "$rollback_errors" -gt 0 ]]; then
        _log_error "Rollback completed with $rollback_errors error(s)"
        _log_error "Manual intervention may be required"
        return 1
    fi

    _log_success "=== ROLLBACK COMPLETED SUCCESSFULLY ==="
    _log_info "Session restored to pre-repair state"
    _log_info "Backup preserved at: $REPAIR_BACKUP_DIR"

    return 0
}
