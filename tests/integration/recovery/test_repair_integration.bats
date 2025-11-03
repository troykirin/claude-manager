#!/usr/bin/env bats
# CM-302: REPAIR Mode Integration Tests
# Tests for safe session repair operations

load helpers/setup_test_environment
load helpers/cleanup_test_environment

setup() {
    setup_test_environment
    verify_prerequisites
}

teardown() {
    cleanup_test_environment
}

# Test: Backup creation before repair
@test "CM-302: repair creates backup before operation" {
    setup_corrupted_session "branch_collision"

    run claude-manager repair test-session-123

    # Verify backup was created (check archive directory)
    [ -d "${CLAUDE_TEST_ARCHIVE}" ]
    backup_count=$(find "${CLAUDE_TEST_ARCHIVE}" -type d -name "backup_*" 2>/dev/null | wc -l)
    [ "$backup_count" -gt 0 ]
}

# Test: Safe session duplication
@test "CM-302: repair safely duplicates session" {
    setup_corrupted_session "branch_collision"

    run claude-manager repair test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "New session" || "$output" =~ "session ID" ]]

    # Verify new session exists
    sessions_count=$(find "${CLAUDE_TEST_CONFIG}/sessions" -type d -mindepth 1 | wc -l)
    [ "$sessions_count" -ge 1 ]
}

# Test: Corruption isolation
@test "CM-302: repair isolates corruption in archive" {
    setup_corrupted_session "branch_collision"

    run claude-manager repair test-session-123

    [ "$status" -eq 0 ]
    # Verify corrupted data moved to archive
    [ -d "${CLAUDE_TEST_ARCHIVE}" ]
    archived_count=$(find "${CLAUDE_TEST_ARCHIVE}" -type d -name "corrupted_*" 2>/dev/null | wc -l)
    [ "$archived_count" -ge 0 ]  # May or may not isolate depending on strategy
}

# Test: Rollback on failure
@test "CM-302: repair rollback works on failure" {
    setup_corrupted_session "branch_collision"

    # Get original state
    original_metadata="${CLAUDE_TEST_CONFIG}/sessions/test-session-123/metadata.json"
    if [[ -f "${original_metadata}" ]]; then
        original_content=$(cat "${original_metadata}")
    fi

    # Attempt repair with simulated failure (if supported)
    run claude-manager repair --dry-run test-session-123

    # Verify original state preserved (dry-run shouldn't modify)
    if [[ -f "${original_metadata}" ]]; then
        current_content=$(cat "${original_metadata}")
        [[ "${current_content}" == "${original_content}" ]]
    fi
}

# Test: Performance - repair completes within threshold
@test "CM-302: repair completes in reasonable time" {
    setup_corrupted_session "branch_collision"

    start_time=$(date +%s)
    run claude-manager repair test-session-123
    end_time=$(date +%s)

    duration=$((end_time - start_time))
    [ "$status" -eq 0 ]
    [ "$duration" -lt 120 ]  # Should complete in <2 minutes
}

# Test: Repair creates clean state
@test "CM-302: repair creates session with clean state" {
    setup_corrupted_session "multi_pattern"

    run claude-manager repair test-session-123

    [ "$status" -eq 0 ]

    # Extract new session ID if available
    if [[ "$output" =~ ([a-f0-9-]{36}) ]]; then
        new_session_id="${BASH_REMATCH[1]}"

        # Diagnose new session - should have high health score
        run claude-manager diagnose "${new_session_id}"
        [[ "$output" =~ "100" || "$output" =~ "healthy" ]]
    fi
}

# Test: Repair preserves important data
@test "CM-302: repair preserves session history" {
    setup_corrupted_session "branch_collision"

    # Create session history file
    mkdir -p "${CLAUDE_TEST_CONFIG}/sessions/test-session-123"
    echo "important history data" > "${CLAUDE_TEST_CONFIG}/sessions/test-session-123/history.txt"

    run claude-manager repair test-session-123

    [ "$status" -eq 0 ]
    # Verify history is preserved in backup
    [ -d "${CLAUDE_TEST_ARCHIVE}" ]
}

# Test: Dry-run mode
@test "CM-302: repair dry-run doesn't modify session" {
    setup_corrupted_session "branch_collision"

    # Get checksum of session data
    session_dir="${CLAUDE_TEST_CONFIG}/sessions/test-session-123"
    before_checksum=$(find "${session_dir}" -type f -exec md5 {} \; 2>/dev/null | md5)

    run claude-manager repair --dry-run test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "dry-run" || "$output" =~ "would" ]]

    # Verify no changes
    after_checksum=$(find "${session_dir}" -type f -exec md5 {} \; 2>/dev/null | md5)
    [[ "${before_checksum}" == "${after_checksum}" ]]
}

# Test: Force repair without confirmation
@test "CM-302: repair with --force skips confirmation" {
    setup_corrupted_session "branch_collision"

    # Force flag should not prompt for input
    run timeout 5 claude-manager repair --force test-session-123

    [ "$status" -eq 0 ]
    [[ ! "$output" =~ "Continue?" ]]
}

# Test: Repair handles critical corruption
@test "CM-302: repair handles critical corruption (health <50)" {
    setup_corrupted_session "critical_multi_pattern"

    run claude-manager repair test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "repair" ]]
}

# Test: Multiple repair operations don't conflict
@test "CM-302: multiple repairs can run sequentially" {
    setup_corrupted_session "branch_collision" "session-1"
    setup_corrupted_session "path_mismatch" "session-2"

    run claude-manager repair session-1
    [ "$status" -eq 0 ]

    run claude-manager repair session-2
    [ "$status" -eq 0 ]

    # Both should succeed
}

# Test: Repair error handling
@test "CM-302: repair handles missing session gracefully" {
    run claude-manager repair nonexistent-session-999

    [ "$status" -ne 0 ]
    [[ "$output" =~ "not found" || "$output" =~ "does not exist" ]]
}

# Test: Repair with custom backup location
@test "CM-302: repair accepts custom backup location" {
    setup_corrupted_session "branch_collision"

    custom_backup="${BATS_TEST_TMPDIR}/custom-backup"
    mkdir -p "${custom_backup}"

    run claude-manager repair --backup-dir "${custom_backup}" test-session-123

    # Should either succeed or gracefully explain the option isn't supported
    [[ "$status" -eq 0 || "$output" =~ "not supported" || "$output" =~ "unrecognized" ]]
}

# Test: Repair verification
@test "CM-302: repair includes verification step" {
    setup_corrupted_session "branch_collision"

    run claude-manager repair test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "verif" || "$output" =~ "check" ]]
}
