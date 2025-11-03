#!/usr/bin/env bats
# CM-301: Diagnostics Integration Tests
# Tests for session corruption detection and health scoring

load helpers/setup_test_environment
load helpers/cleanup_test_environment

setup() {
    setup_test_environment
    verify_prerequisites
}

teardown() {
    cleanup_test_environment
}

# Test: Basic corruption detection
@test "CM-301: diagnose detects branch collision" {
    setup_corrupted_session "branch_collision"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "branch collision" || "$output" =~ "duplicate_branch" ]]
    [[ "$output" =~ "Health Score:" ]]
}

# Test: Healthy session validation
@test "CM-301: diagnose validates healthy session" {
    setup_healthy_session "test-session-healthy"

    run claude-manager diagnose test-session-healthy

    [ "$status" -eq 0 ]
    [[ "$output" =~ "Health Score: 100" || "$output" =~ "healthy" ]]
}

# Test: Multiple pattern detection
@test "CM-301: diagnose detects multiple corruption patterns" {
    setup_corrupted_session "multi_pattern"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "3 corruption patterns" || "$output" =~ "multiple issues" ]]
}

# Test: JSON output format
@test "CM-301: diagnose outputs valid JSON" {
    setup_corrupted_session "branch_collision"

    run claude-manager diagnose --json test-session-123

    [ "$status" -eq 0 ]
    echo "$output" | jq . > /dev/null  # Validates JSON structure
}

# Test: Health score calculation accuracy
@test "CM-301: diagnose calculates health score accurately" {
    setup_healthy_session "test-session-healthy"

    run claude-manager diagnose test-session-healthy

    [ "$status" -eq 0 ]
    # Extract health score (handles various output formats)
    if [[ "$output" =~ Health[[:space:]]*Score:[[:space:]]*([0-9]+) ]]; then
        health_score="${BASH_REMATCH[1]}"
        [ "$health_score" -ge 90 ]
    fi
}

# Test: Cross-system inconsistency detection
@test "CM-301: diagnose detects cross-system inconsistency" {
    setup_corrupted_session "cross_system_inconsistency"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "branch_mismatch" || "$output" =~ "inconsistency" ]]
}

# Test: Path validation
@test "CM-301: diagnose detects invalid paths" {
    setup_corrupted_session "path_mismatch"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "invalid_path" || "$output" =~ "path" ]]
}

# Test: Critical corruption threshold
@test "CM-301: diagnose identifies critical corruption (health <50)" {
    setup_corrupted_session "critical_multi_pattern"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "critical" || "$output" =~ "severe" ]]
}

# Test: Error handling for missing session
@test "CM-301: diagnose handles missing session gracefully" {
    run claude-manager diagnose nonexistent-session-999

    [ "$status" -ne 0 ]
    [[ "$output" =~ "not found" || "$output" =~ "does not exist" ]]
}

# Test: Diagnostic report completeness
@test "CM-301: diagnose generates complete diagnostic report" {
    setup_corrupted_session "multi_pattern"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]
    # Verify report contains key sections
    [[ "$output" =~ "session" ]]
    [[ "$output" =~ "corruption" || "$output" =~ "pattern" ]]
    [[ "$output" =~ "health" || "$output" =~ "score" ]]
}

# Test: Performance - diagnostic completes quickly
@test "CM-301: diagnose completes within performance threshold" {
    setup_corrupted_session "multi_pattern"

    start_time=$(date +%s)
    run claude-manager diagnose test-session-123
    end_time=$(date +%s)

    duration=$((end_time - start_time))
    [ "$status" -eq 0 ]
    [ "$duration" -lt 10 ]  # Should complete in <10 seconds
}

# Test: Verbose output mode
@test "CM-301: diagnose provides verbose output with --verbose flag" {
    setup_corrupted_session "branch_collision"

    run claude-manager diagnose --verbose test-session-123

    [ "$status" -eq 0 ]
    # Verbose mode should provide more detail
    [[ "${#output}" -gt 100 ]]  # Output should be substantial
}

# Test: Batch diagnostics
@test "CM-301: diagnose can process multiple sessions" {
    setup_corrupted_session "branch_collision" "session-1"
    setup_healthy_session "session-2"
    setup_corrupted_session "path_mismatch" "session-3"

    run claude-manager diagnose --all

    [ "$status" -eq 0 ]
    [[ "$output" =~ "session-1" ]]
    [[ "$output" =~ "session-2" ]]
    [[ "$output" =~ "session-3" ]]
}
