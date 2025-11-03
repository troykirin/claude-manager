#!/usr/bin/env bats
# CM-302: REPAIR Mode Duplication Tests
# Tests for safe session duplication with corruption isolation

load '../test_helper'

# Test fixtures
setup() {
    # Create temporary test environment
    export TEST_CLAUDE_DIR="$(mktemp -d)"
    export CLAUDE_DIR="$TEST_CLAUDE_DIR"
    export INTERACTIVE="false"
    export DRY_RUN="false"

    # Create directory structure
    mkdir -p "$TEST_CLAUDE_DIR"/{projects,todos,statsig,shell-snapshots,.archive}

    # Create mock project directory
    export TEST_PROJECT_DIR="$TEST_CLAUDE_DIR/projects/-Users-test-project"
    mkdir -p "$TEST_PROJECT_DIR"

    # Generate test UUID
    if command -v uuidgen >/dev/null 2>&1; then
        TEST_SESSION_UUID=$(uuidgen | tr '[:upper:]' '[:lower:]')
    else
        TEST_SESSION_UUID="550e8400-e29b-41d4-a716-446655440000"
    fi

    # Create mock session file (valid JSONL)
    cat > "$TEST_PROJECT_DIR/${TEST_SESSION_UUID}.jsonl" << 'EOF'
{"sessionId":"550e8400-e29b-41d4-a716-446655440000","cwd":"/Users/test/project","timestamp":"2024-01-01T00:00:00Z"}
{"type":"message","role":"user","content":"test message 1"}
{"type":"message","role":"assistant","content":"test response 1"}
EOF

    # Create mock todo file
    cat > "$TEST_CLAUDE_DIR/todos/${TEST_SESSION_UUID}-agent-main.json" << 'EOF'
{
    "version": "1.0",
    "todos": [
        {"id": 1, "content": "Test todo", "status": "pending"}
    ]
}
EOF

    # Source the main script (for functions)
    source "$BATS_TEST_DIRNAME/../../claude-manager.sh"

    # Source repair module
    source "$BATS_TEST_DIRNAME/../../lib/repair.sh"
}

teardown() {
    # Clean up test environment
    if [[ -n "$TEST_CLAUDE_DIR" && -d "$TEST_CLAUDE_DIR" ]]; then
        rm -rf "$TEST_CLAUDE_DIR"
    fi
}

# ============================================================================
# Test: Backup Creation
# ============================================================================

@test "repair: backup creation for all 4 state systems" {
    local timestamp="20240101_000000"

    run _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"

    [ "$status" -eq 0 ]
    [[ "$output" =~ "Creating backup" ]]

    # Verify backup directory exists
    [ -d "$REPAIR_BACKUP_DIR" ]

    # Verify all 4 state systems backed up
    [ -d "$REPAIR_BACKUP_DIR/projects" ]
    [ -d "$REPAIR_BACKUP_DIR/todos" ]
    [ -d "$REPAIR_BACKUP_DIR/statsig" ]
    [ -d "$REPAIR_BACKUP_DIR/shell-snapshots" ]

    # Verify backup manifest created
    [ -f "$REPAIR_BACKUP_DIR/manifest.json" ]

    # Verify project file backed up
    local backup_project=$(find "$REPAIR_BACKUP_DIR/projects" -name "${TEST_SESSION_UUID}.jsonl")
    [ -f "$backup_project" ]
}

@test "repair: backup includes todo files" {
    local timestamp="20240101_000000"

    run _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"

    [ "$status" -eq 0 ]

    # Verify todo backup
    local backup_todo="$REPAIR_BACKUP_DIR/todos/${TEST_SESSION_UUID}-agent-main.json"
    [ -f "$backup_todo" ]
}

# ============================================================================
# Test: Safe Duplication
# ============================================================================

@test "repair: safe duplication generates new UUID" {
    run _repair_duplicate_safe "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]
    [[ "$output" =~ "Generated new UUID" ]]

    # Verify new UUID is different from original
    [ "$REPAIR_NEW_UUID" != "$TEST_SESSION_UUID" ]

    # Verify new UUID format
    [[ "$REPAIR_NEW_UUID" =~ ^[a-f0-9-]{36}$ ]]
}

@test "repair: safe duplication creates clean state" {
    run _repair_duplicate_safe "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]

    # Verify new session file created
    local new_session_file="$TEST_PROJECT_DIR/${REPAIR_NEW_UUID}.jsonl"
    [ -f "$new_session_file" ]

    # Verify JSONL integrity
    command -v jq >/dev/null 2>&1 && jq empty "$new_session_file"

    # Verify original file untouched
    [ -f "$TEST_PROJECT_DIR/${TEST_SESSION_UUID}.jsonl" ]
}

@test "repair: safe duplication preserves JSONL content" {
    run _repair_duplicate_safe "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]

    # Compare line counts
    local orig_lines=$(wc -l < "$TEST_PROJECT_DIR/${TEST_SESSION_UUID}.jsonl")
    local new_lines=$(wc -l < "$TEST_PROJECT_DIR/${REPAIR_NEW_UUID}.jsonl")

    [ "$orig_lines" -eq "$new_lines" ]
}

# ============================================================================
# Test: Corruption Isolation
# ============================================================================

@test "repair: corruption isolation moves to .archive/" {
    # First create backup and duplicate
    local timestamp="20240101_000000"
    _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"
    _repair_duplicate_safe "$TEST_SESSION_UUID"

    run _repair_isolate_corruption "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]
    [[ "$output" =~ "Archiving original" ]]

    # Verify archive created
    local archives=$(find "$TEST_CLAUDE_DIR/.archive" -type d -name "sessions-*")
    [ -n "$archives" ]

    # Verify original moved to archive
    local archived_session=$(find "$TEST_CLAUDE_DIR/.archive" -name "${TEST_SESSION_UUID}.jsonl")
    [ -f "$archived_session" ]
}

@test "repair: corruption isolation creates archive manifest" {
    local timestamp="20240101_000000"
    _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"
    _repair_duplicate_safe "$TEST_SESSION_UUID"

    run _repair_isolate_corruption "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]

    # Find archive directory
    local archive_dir=$(find "$TEST_CLAUDE_DIR/.archive" -type d -name "sessions-*" | head -1)

    # Verify manifest exists
    [ -f "$archive_dir/ARCHIVE_REASON.txt" ]

    # Verify manifest contains key info
    grep -q "Session ID: $TEST_SESSION_UUID" "$archive_dir/ARCHIVE_REASON.txt"
    grep -q "REPAIR mode" "$archive_dir/ARCHIVE_REASON.txt"
}

# ============================================================================
# Test: State Restoration
# ============================================================================

@test "repair: state restoration validates JSONL" {
    local timestamp="20240101_000000"
    _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"
    _repair_duplicate_safe "$TEST_SESSION_UUID"

    run _repair_restore_state "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]
    [[ "$output" =~ "Restoring valid state" ]]

    # Verify new session exists and is valid
    local new_session_file="$TEST_PROJECT_DIR/${REPAIR_NEW_UUID}.jsonl"
    [ -f "$new_session_file" ]

    # Verify JSONL integrity
    if command -v jq >/dev/null 2>&1; then
        jq empty "$new_session_file"
    fi
}

# ============================================================================
# Test: Rollback Mechanism
# ============================================================================

@test "repair: rollback restores from backup" {
    local timestamp="20240101_000000"

    # Create backup
    _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"

    # Simulate failure by creating new session then removing original
    _repair_duplicate_safe "$TEST_SESSION_UUID"
    rm "$TEST_PROJECT_DIR/${TEST_SESSION_UUID}.jsonl"

    # Execute rollback
    run _repair_rollback "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]
    [[ "$output" =~ "ROLLBACK COMPLETED" ]]

    # Verify original session restored
    [ -f "$TEST_PROJECT_DIR/${TEST_SESSION_UUID}.jsonl" ]
}

@test "repair: rollback removes failed duplicate" {
    local timestamp="20240101_000000"

    _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"
    _repair_duplicate_safe "$TEST_SESSION_UUID"

    # Verify new session exists
    [ -f "$TEST_PROJECT_DIR/${REPAIR_NEW_UUID}.jsonl" ]

    # Execute rollback
    run _repair_rollback "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]

    # Verify new session removed
    [ ! -f "$TEST_PROJECT_DIR/${REPAIR_NEW_UUID}.jsonl" ]
}

@test "repair: rollback handles missing backup gracefully" {
    REPAIR_BACKUP_DIR="/nonexistent/backup/dir"

    run _repair_rollback "$TEST_SESSION_UUID"

    [ "$status" -eq 1 ]
    [[ "$output" =~ "Backup directory not found" ]]
}

# ============================================================================
# Test: End-to-End REPAIR Workflow
# ============================================================================

@test "repair: end-to-end workflow completes successfully" {
    run repair_session "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]
    [[ "$output" =~ "REPAIR Mode: Session repair completed successfully" ]]

    # Verify new session created
    [ -n "$REPAIR_NEW_UUID" ]
    [ -f "$TEST_PROJECT_DIR/${REPAIR_NEW_UUID}.jsonl" ]

    # Verify original archived
    local archived=$(find "$TEST_CLAUDE_DIR/.archive" -name "${TEST_SESSION_UUID}.jsonl")
    [ -f "$archived" ]

    # Verify backup preserved
    [ -d "$REPAIR_BACKUP_DIR" ]
}

@test "repair: workflow preserves data integrity" {
    run repair_session "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]

    # Compare content
    local new_session_file="$TEST_PROJECT_DIR/${REPAIR_NEW_UUID}.jsonl"
    local backup_session=$(find "$REPAIR_BACKUP_DIR/projects" -name "${TEST_SESSION_UUID}.jsonl")

    # Verify line counts match
    local orig_lines=$(wc -l < "$backup_session")
    local new_lines=$(wc -l < "$new_session_file")

    [ "$orig_lines" -eq "$new_lines" ]
}

@test "repair: workflow creates complete backup" {
    run repair_session "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]

    # Verify backup manifest
    [ -f "$REPAIR_BACKUP_DIR/manifest.json" ]

    # Verify manifest contains repair status
    grep -q '"repair_status"' "$REPAIR_BACKUP_DIR/manifest.json"
}

# ============================================================================
# Test: Pre-flight Checks
# ============================================================================

@test "repair: pre-flight detects missing session" {
    local fake_uuid="00000000-0000-0000-0000-000000000000"

    run _repair_verify_preconditions "$fake_uuid"

    [ "$status" -eq 1 ]
    [[ "$output" =~ "Session not found" ]]
}

@test "repair: pre-flight validates disk space" {
    run _repair_verify_preconditions "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]
    [[ "$output" =~ "disk space" ]]
}

@test "repair: pre-flight checks JSONL integrity" {
    # Create invalid JSONL
    echo "invalid json{" > "$TEST_PROJECT_DIR/${TEST_SESSION_UUID}.jsonl"

    run _repair_verify_preconditions "$TEST_SESSION_UUID"

    [ "$status" -eq 1 ]
    [[ "$output" =~ "invalid JSON" ]]
}

# ============================================================================
# Test: Post-Repair Verification
# ============================================================================

@test "repair: verification detects successful repair" {
    local timestamp="20240101_000000"
    _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"
    _repair_duplicate_safe "$TEST_SESSION_UUID"
    _repair_isolate_corruption "$TEST_SESSION_UUID"
    _repair_restore_state "$TEST_SESSION_UUID"

    run _repair_verify_success "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]
    [[ "$output" =~ "verification passed" ]]
}

@test "repair: verification calculates health score" {
    local timestamp="20240101_000000"
    _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"
    _repair_duplicate_safe "$TEST_SESSION_UUID"
    _repair_isolate_corruption "$TEST_SESSION_UUID"
    _repair_restore_state "$TEST_SESSION_UUID"

    run _repair_verify_success "$TEST_SESSION_UUID"

    [ "$status" -eq 0 ]
    [[ "$output" =~ "health score" ]]
}

# ============================================================================
# Test: Error Handling
# ============================================================================

@test "repair: handles duplicate failure gracefully" {
    # Make project directory read-only
    chmod -w "$TEST_PROJECT_DIR"

    run repair_session "$TEST_SESSION_UUID"

    [ "$status" -eq 1 ]

    # Restore permissions for teardown
    chmod +w "$TEST_PROJECT_DIR"
}

@test "repair: rollback on verification failure" {
    # Create backup
    local timestamp="20240101_000000"
    _repair_create_backup "$TEST_SESSION_UUID" "$timestamp"
    _repair_duplicate_safe "$TEST_SESSION_UUID"

    # Corrupt new session
    echo "corrupt" > "$TEST_PROJECT_DIR/${REPAIR_NEW_UUID}.jsonl"

    run _repair_verify_success "$TEST_SESSION_UUID"

    [ "$status" -eq 1 ]
    [[ "$output" =~ "invalid JSON" ]]
}

# ============================================================================
# Test: Performance
# ============================================================================

@test "repair: completes in under 2 minutes" {
    # Note: This is a smoke test for performance
    # Real timing would need larger datasets

    local start_time=$(date +%s)

    run repair_session "$TEST_SESSION_UUID"

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    [ "$status" -eq 0 ]

    # Should complete very quickly with test data
    [ "$duration" -lt 120 ]
}
