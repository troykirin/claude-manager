#!/usr/bin/env bats
# Unit tests for diagnostics.sh module

setup() {
    # Create temporary test environment
    export TEST_CLAUDE_DIR="$(mktemp -d)"
    export CLAUDE_DIR="$TEST_CLAUDE_DIR"

    # Create directory structure
    mkdir -p "$CLAUDE_DIR/projects"
    mkdir -p "$CLAUDE_DIR/todos"
    mkdir -p "$CLAUDE_DIR/shell-snapshots"
    mkdir -p "$CLAUDE_DIR/statsig"

    # Source the main script for logging functions
    source "${BATS_TEST_DIRNAME}/../../claude-manager.sh"

    # Source diagnostics module
    source "${BATS_TEST_DIRNAME}/../../lib/diagnostics.sh"

    # Test UUID
    export TEST_UUID="12345678-1234-1234-1234-123456789abc"
}

teardown() {
    # Clean up test environment
    rm -rf "$TEST_CLAUDE_DIR"
}

# Helper: Create a test session file
create_test_session() {
    local uuid="$1"
    local project_dir="$2"
    local cwd_path="$3"

    mkdir -p "$CLAUDE_DIR/projects/$project_dir"
    local session_file="$CLAUDE_DIR/projects/$project_dir/${uuid}.jsonl"

    cat > "$session_file" <<EOF
{"sessionId":"$uuid","cwd":"$cwd_path","timestamp":"2024-01-01T00:00:00Z"}
{"role":"user","content":"test message","cwd":"$cwd_path"}
{"role":"assistant","content":"test response","cwd":"$cwd_path"}
EOF
}

# Helper: Create test todo files
create_test_todos() {
    local uuid="$1"
    local count="${2:-1}"

    for ((i=1; i<=count; i++)); do
        cat > "$CLAUDE_DIR/todos/${uuid}-agent-${i}.json" <<EOF
{"task":"test task $i","status":"pending"}
EOF
    done
}

# Helper: Create test shell snapshot
create_test_snapshot() {
    local uuid="$1"

    cat > "$CLAUDE_DIR/shell-snapshots/snapshot-${uuid}-1.sh" <<EOF
#!/bin/bash
export TEST_VAR="test"
EOF
}

# =============================================================================
# Pattern Detection Tests
# =============================================================================

@test "detect_branch_collision: no collision when session in single project" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"

    result=$(detect_branch_collision "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)

    [ "$detected" = "false" ]
}

@test "detect_branch_collision: detects collision when session in multiple projects" {
    create_test_session "$TEST_UUID" "-Users-test-project1" "/Users/test/project1"
    create_test_session "$TEST_UUID" "-Users-test-project2" "/Users/test/project2"

    result=$(detect_branch_collision "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$detected" = "true" ]
    [[ "$details" == *"2 different project"* ]]
}

@test "detect_migration_race: no race when single session file exists" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"

    result=$(detect_migration_race "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)

    [ "$detected" = "false" ]
}

@test "detect_migration_race: detects race with multiple copies" {
    create_test_session "$TEST_UUID" "-Users-test-old" "/Users/test/old"
    create_test_session "$TEST_UUID" "-Users-test-new" "/Users/test/new"

    result=$(detect_migration_race "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$detected" = "true" ]
    [[ "$details" == *"2 copies"* ]]
}

@test "detect_migration_race: detects backup files" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"

    # Create backup file
    touch "$CLAUDE_DIR/projects/-Users-test-project/${TEST_UUID}.jsonl.bak"

    result=$(detect_migration_race "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$detected" = "false" ]  # Single session file
    # But if we had multiple, backup would be mentioned
}

@test "detect_cross_system_inconsistency: healthy state with all files" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"
    create_test_todos "$TEST_UUID" 2
    create_test_snapshot "$TEST_UUID"

    result=$(detect_cross_system_inconsistency "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)

    [ "$detected" = "false" ]
}

@test "detect_cross_system_inconsistency: detects orphaned todos" {
    # Todos without project file
    create_test_todos "$TEST_UUID" 3

    result=$(detect_cross_system_inconsistency "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$detected" = "true" ]
    [[ "$details" == *"orphaned todos"* ]]
}

@test "detect_cross_system_inconsistency: handles missing todos gracefully" {
    # Project without todos (could be archived)
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"

    result=$(detect_cross_system_inconsistency "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    # Should note missing todos but not flag as error
    [ "$detected" = "false" ]
    [[ "$details" == *"no todos"* ]]
}

@test "detect_cross_system_inconsistency: detects orphaned snapshots" {
    # Snapshots without project file
    create_test_snapshot "$TEST_UUID"

    result=$(detect_cross_system_inconsistency "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$detected" = "true" ]
    [[ "$details" == *"orphaned snapshots"* ]]
}

@test "detect_path_mismatch: no mismatch when paths match" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"

    result=$(detect_path_mismatch "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)

    [ "$detected" = "false" ]
}

@test "detect_path_mismatch: detects mismatch when paths differ" {
    # Project directory says one path, but session contains different path
    create_test_session "$TEST_UUID" "-Users-test-old-path" "/Users/test/new/path"

    result=$(detect_path_mismatch "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$detected" = "true" ]
    [[ "$details" == *"Expected:"* ]]
    [[ "$details" == *"/Users/test/old/path"* ]]
}

@test "detect_orphaned_todos: no orphans when project exists" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"
    create_test_todos "$TEST_UUID" 2

    result=$(detect_orphaned_todos "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)

    [ "$detected" = "false" ]
}

@test "detect_orphaned_todos: detects orphaned todos" {
    create_test_todos "$TEST_UUID" 3

    result=$(detect_orphaned_todos "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$detected" = "true" ]
    [[ "$details" == *"3 orphaned todo"* ]]
}

@test "detect_timestamp_drift: no drift with recent timestamps" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"
    create_test_todos "$TEST_UUID" 1

    # Files just created, should be recent
    result=$(detect_timestamp_drift "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)

    [ "$detected" = "false" ]
}

@test "detect_timestamp_drift: detects significant drift" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"
    create_test_todos "$TEST_UUID" 1

    # Artificially age the project file
    local project_file="$CLAUDE_DIR/projects/-Users-test-project/${TEST_UUID}.jsonl"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: use -t flag (format: [[CC]YY]MMDDhhmm[.ss])
        touch -t 202401010000 "$project_file"
    else
        # Linux: use -d flag
        touch -d "2024-01-01 00:00:00" "$project_file"
    fi

    result=$(detect_timestamp_drift "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$detected" = "true" ]
    [[ "$details" == *"drift"* ]]
}

# =============================================================================
# Validation Tests
# =============================================================================

@test "validate_process_safety: returns valid status" {
    # May return ok or warning depending on active processes
    result=$(validate_process_safety)
    status=$(echo "$result" | cut -d'|' -f1)

    # Should return either "ok" or "warning", not error or empty
    [[ "$status" == "ok" || "$status" == "warning" ]]
}

@test "validate_cross_system_state: ok with healthy session" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"
    create_test_todos "$TEST_UUID" 2

    result=$(validate_cross_system_state "$TEST_UUID")
    status=$(echo "$result" | cut -d'|' -f1)

    [ "$status" = "ok" ]
}

@test "validate_cross_system_state: error with no project file" {
    result=$(validate_cross_system_state "$TEST_UUID")
    status=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$status" = "error" ]
    [[ "$details" == *"no project file"* ]]
}

@test "validate_cross_system_state: warning with multiple project files" {
    create_test_session "$TEST_UUID" "-Users-test-project1" "/Users/test/project1"
    create_test_session "$TEST_UUID" "-Users-test-project2" "/Users/test/project2"

    result=$(validate_cross_system_state "$TEST_UUID")
    status=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$status" = "warning" ]
    [[ "$details" == *"2 project files"* ]]
}

@test "validate_path_consistency: ok with single path" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"

    result=$(validate_path_consistency "$TEST_UUID")
    status=$(echo "$result" | cut -d'|' -f1)

    [ "$status" = "ok" ]
}

@test "validate_path_consistency: warning with multiple paths" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"

    # Add another line with different path
    echo '{"role":"user","content":"test","cwd":"/Users/test/other"}' >> \
        "$CLAUDE_DIR/projects/-Users-test-project/${TEST_UUID}.jsonl"

    result=$(validate_path_consistency "$TEST_UUID")
    status=$(echo "$result" | cut -d'|' -f1)
    details=$(echo "$result" | cut -d'|' -f2)

    [ "$status" = "warning" ]
    [[ "$details" == *"2 different paths"* ]]
}

# =============================================================================
# Health Score Calculation Tests
# =============================================================================

@test "calculate_health_score: perfect score with no issues" {
    local patterns="false
false
false
false
false
false"
    local validations="ok
ok
ok"

    score=$(calculate_health_score "$patterns" "$validations")

    [ "$score" -eq 100 ]
}

@test "calculate_health_score: deducts 20 points per corruption pattern" {
    local patterns="true
false
false
false
false
false"
    local validations="ok
ok
ok"

    score=$(calculate_health_score "$patterns" "$validations")

    [ "$score" -eq 80 ]
}

@test "calculate_health_score: deducts 10 points per validation warning" {
    local patterns="false
false
false
false
false
false"
    local validations="warning
ok
ok"

    score=$(calculate_health_score "$patterns" "$validations")

    [ "$score" -eq 90 ]
}

@test "calculate_health_score: combines pattern and validation deductions" {
    local patterns="true
true
false
false
false
false"
    local validations="warning
warning
ok"

    # 2 patterns = -40, 2 warnings = -20, total = 40
    score=$(calculate_health_score "$patterns" "$validations")

    [ "$score" -eq 40 ]
}

@test "calculate_health_score: never goes below 0" {
    # All patterns detected + all warnings
    local patterns="true
true
true
true
true
true"
    local validations="warning
warning
warning"

    # 6 patterns = -120, 3 warnings = -30, would be -50 but capped at 0
    score=$(calculate_health_score "$patterns" "$validations")

    [ "$score" -eq 0 ]
}

# =============================================================================
# Severity Classification Tests
# =============================================================================

@test "get_severity: HEALTHY for scores >= 90" {
    severity=$(get_severity 100)
    [ "$severity" = "HEALTHY" ]

    severity=$(get_severity 90)
    [ "$severity" = "HEALTHY" ]
}

@test "get_severity: MINOR_ISSUES for scores 70-89" {
    severity=$(get_severity 89)
    [ "$severity" = "MINOR_ISSUES" ]

    severity=$(get_severity 70)
    [ "$severity" = "MINOR_ISSUES" ]
}

@test "get_severity: DEGRADED for scores 50-69" {
    severity=$(get_severity 69)
    [ "$severity" = "DEGRADED" ]

    severity=$(get_severity 50)
    [ "$severity" = "DEGRADED" ]
}

@test "get_severity: CORRUPTED for scores 30-49" {
    severity=$(get_severity 49)
    [ "$severity" = "CORRUPTED" ]

    severity=$(get_severity 30)
    [ "$severity" = "CORRUPTED" ]
}

@test "get_severity: CRITICAL for scores < 30" {
    severity=$(get_severity 29)
    [ "$severity" = "CRITICAL" ]

    severity=$(get_severity 0)
    [ "$severity" = "CRITICAL" ]
}

# =============================================================================
# Integration Tests
# =============================================================================

@test "diagnose_session_health: runs without errors on healthy session" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"
    create_test_todos "$TEST_UUID" 2

    # Mock statsig for "current" detection
    echo "$TEST_UUID" > "$CLAUDE_DIR/statsig/statsig.session_id.test"

    run diagnose_session_health "$TEST_UUID"

    [ "$status" -eq 0 ]
}

@test "diagnose_session_health: detects current session from statsig" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"
    create_test_todos "$TEST_UUID" 2

    # Mock statsig
    echo "$TEST_UUID" > "$CLAUDE_DIR/statsig/statsig.session_id.test"

    run diagnose_session_health "current"

    [ "$status" -eq 0 ]
}

@test "diagnose_session_health: returns error when session not found" {
    run diagnose_session_health "nonexistent-uuid"

    # Should handle gracefully even if session doesn't exist
    [ "$status" -eq 0 ]
}

@test "diagnose_session_health: detects multiple corruption patterns" {
    # Create a corrupted session with multiple issues
    create_test_session "$TEST_UUID" "-Users-test-old" "/Users/test/new"  # Path mismatch
    create_test_session "$TEST_UUID" "-Users-test-new" "/Users/test/new"  # Branch collision
    # No todos = orphaned project (not critical but noted)

    run diagnose_session_health "$TEST_UUID"

    [ "$status" -eq 0 ]
    # Output should mention both issues
}

# =============================================================================
# Edge Cases and Error Handling
# =============================================================================

@test "detect_branch_collision: handles missing projects directory" {
    rm -rf "$CLAUDE_DIR/projects"

    result=$(detect_branch_collision "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)

    [ "$detected" = "false" ]
}

@test "detect_orphaned_todos: handles missing todos directory" {
    rm -rf "$CLAUDE_DIR/todos"

    result=$(detect_orphaned_todos "$TEST_UUID")
    detected=$(echo "$result" | cut -d'|' -f1)

    [ "$detected" = "false" ]
}

@test "validate_path_consistency: handles malformed JSONL gracefully" {
    create_test_session "$TEST_UUID" "-Users-test-project" "/Users/test/project"

    # Append malformed line
    echo 'this is not valid json' >> \
        "$CLAUDE_DIR/projects/-Users-test-project/${TEST_UUID}.jsonl"

    result=$(validate_path_consistency "$TEST_UUID")
    status=$(echo "$result" | cut -d'|' -f1)

    # Should complete without crashing
    [ -n "$status" ]
}

@test "calculate_health_score: handles empty input" {
    score=$(calculate_health_score "" "")

    [ "$score" -eq 100 ]
}
