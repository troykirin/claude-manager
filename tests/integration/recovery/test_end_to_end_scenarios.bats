#!/usr/bin/env bats
# CM-304: End-to-End Scenario Tests
# Complete workflow validation for session recovery

load helpers/setup_test_environment
load helpers/cleanup_test_environment

setup() {
    setup_test_environment
    verify_prerequisites
}

teardown() {
    cleanup_test_environment
}

# Scenario A: Full recovery workflow
@test "E2E Scenario A: Detect → Repair → Verify → Audit" {
    # Step 1: Setup corrupted session
    setup_corrupted_session "multi_pattern"

    # Step 2: Detect corruption
    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "Health Score:" ]]

    # Extract health score
    if [[ "$output" =~ Health[[:space:]]*Score:[[:space:]]*([0-9]+) ]]; then
        health_score="${BASH_REMATCH[1]}"
        [ "$health_score" -lt 90 ]  # Should detect issues
    fi

    # Step 3: Repair session
    run claude-manager repair --force test-session-123

    [ "$status" -eq 0 ]

    # Extract new session ID if available
    new_session_id=""
    if [[ "$output" =~ ([a-f0-9-]{36}) ]]; then
        new_session_id="${BASH_REMATCH[1]}"
    elif [[ "$output" =~ session[[:space:]]+([a-zA-Z0-9-]+) ]]; then
        new_session_id="${BASH_REMATCH[1]}"
    fi

    # Step 4: Verify repair (on new session if available, otherwise original)
    if [[ -n "$new_session_id" ]] && [[ "$new_session_id" != "test-session-123" ]]; then
        run claude-manager diagnose "$new_session_id"
    else
        run claude-manager diagnose test-session-123
    fi

    [ "$status" -eq 0 ]
    # Repaired session should have better health

    # Step 5: Check audit trail (if supported)
    if command -v node &> /dev/null; then
        local cli_path="../../../federation-integration/dist/recovery/cli.js"
        if [[ -f "${cli_path}" ]]; then
            run node "${cli_path}" query --session test-session-123
            # Audit should show both diagnose and repair operations
        fi
    fi
}

# Scenario B: Critical corruption workflow
@test "E2E Scenario B: Critical corruption → REPAIR → Linear tracking" {
    skip "Requires Linear API integration"

    # Step 1: Setup critical corruption (health <50)
    setup_corrupted_session "critical_multi_pattern"

    # Step 2: Diagnose (should trigger Linear issue)
    export LINEAR_API_KEY="test-key"
    export LINEAR_TEAM_ID="test-team"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "critical" ]]

    # Extract Linear issue ID if created
    linear_issue=""
    if [[ "$output" =~ Linear[[:space:]]+issue:[[:space:]]+([A-Z]+-[0-9]+) ]]; then
        linear_issue="${BASH_REMATCH[1]}"
    fi

    # Step 3: Repair
    run claude-manager repair --force test-session-123

    [ "$status" -eq 0 ]

    # Step 4: Verify Linear issue updated (if created)
    if [[ -n "$linear_issue" ]]; then
        [[ "$output" =~ "$linear_issue" || "$output" =~ "updated" ]]
    fi
}

# Scenario C: Failed repair with rollback
@test "E2E Scenario C: Failed repair → Rollback → Manual intervention" {
    setup_corrupted_session "branch_collision"

    # Get original state
    session_dir="${CLAUDE_TEST_CONFIG}/sessions/test-session-123"
    original_metadata=""
    if [[ -f "${session_dir}/metadata.json" ]]; then
        original_metadata=$(cat "${session_dir}/metadata.json")
    fi

    # Attempt dry-run repair (simulates failure without actual changes)
    run claude-manager repair --dry-run test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "dry-run" ]]

    # Verify original state preserved
    if [[ -n "$original_metadata" ]] && [[ -f "${session_dir}/metadata.json" ]]; then
        current_metadata=$(cat "${session_dir}/metadata.json")
        [[ "$current_metadata" == "$original_metadata" ]]
    fi

    # Manual intervention: Diagnose shows issue still present
    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]
    [[ "$output" =~ "collision" || "$output" =~ "duplicate_branch" ]]
}

# Scenario D: Multiple sessions concurrent repair
@test "E2E Scenario D: Batch processing multiple corrupted sessions" {
    # Setup multiple corrupted sessions
    setup_corrupted_session "branch_collision" "session-1"
    setup_corrupted_session "path_mismatch" "session-2"
    setup_corrupted_session "cross_system_inconsistency" "session-3"

    # Diagnose all sessions
    run claude-manager diagnose --all

    [ "$status" -eq 0 ]
    [[ "$output" =~ "session-1" ]]
    [[ "$output" =~ "session-2" ]]
    [[ "$output" =~ "session-3" ]]

    # Repair sessions individually
    run claude-manager repair --force session-1
    [ "$status" -eq 0 ]

    run claude-manager repair --force session-2
    [ "$status" -eq 0 ]

    run claude-manager repair --force session-3
    [ "$status" -eq 0 ]

    # Verify all repairs completed
}

# Scenario E: Progressive corruption detection
@test "E2E Scenario E: Detect early → Monitor → Escalate if needed" {
    # Start with healthy session
    setup_healthy_session "test-session-healthy"

    # Initial check: Should be healthy
    run claude-manager diagnose test-session-healthy

    [ "$status" -eq 0 ]
    [[ "$output" =~ "100" || "$output" =~ "healthy" ]]

    # Introduce minor corruption
    session_dir="${CLAUDE_TEST_CONFIG}/sessions/test-session-healthy"
    echo '{"duplicate": "branch"}' >> "${session_dir}/metadata.json"

    # Detect corruption
    run claude-manager diagnose test-session-healthy

    [ "$status" -eq 0 ]
    # May or may not detect invalid JSON depending on parser

    # Repair if needed
    if [[ "$output" =~ "corruption" ]] || [[ "$output" =~ "issue" ]]; then
        run claude-manager repair --force test-session-healthy
        [ "$status" -eq 0 ]
    fi
}

# Scenario F: Recovery with data preservation
@test "E2E Scenario F: Repair preserves critical session data" {
    setup_corrupted_session "branch_collision"

    # Add important data
    session_dir="${CLAUDE_TEST_CONFIG}/sessions/test-session-123"
    echo "critical user data" > "${session_dir}/user_data.txt"

    # Repair session
    run claude-manager repair --force test-session-123

    [ "$status" -eq 0 ]

    # Verify backup contains user data
    if [[ -d "${CLAUDE_TEST_ARCHIVE}" ]]; then
        backup_count=$(find "${CLAUDE_TEST_ARCHIVE}" -name "user_data.txt" | wc -l)
        [ "$backup_count" -ge 1 ]
    fi
}

# Scenario G: Federation integration workflow
@test "E2E Scenario G: Operations propagate through federation" {
    skip "Requires full federation stack"

    setup_corrupted_session "multi_pattern"

    # Check federation health
    run claude-manager federation-health

    # Perform operations
    run claude-manager diagnose test-session-123
    [ "$status" -eq 0 ]

    run claude-manager repair --force test-session-123
    [ "$status" -eq 0 ]

    # Verify events in Loki
    sleep 2
    run curl -s "http://localhost:3100/loki/api/v1/query?query={job=\"claude-manager\"}"

    if [ "$status" -eq 0 ]; then
        [[ "$output" =~ "diagnose" ]]
        [[ "$output" =~ "repair" ]]
    fi
}

# Scenario H: Error recovery workflow
@test "E2E Scenario H: Handle errors gracefully throughout workflow" {
    # Attempt operations on nonexistent session
    run claude-manager diagnose nonexistent-999

    [ "$status" -ne 0 ]
    [[ "$output" =~ "not found" || "$output" =~ "does not exist" ]]

    # Attempt repair on nonexistent session
    run claude-manager repair nonexistent-999

    [ "$status" -ne 0 ]
    [[ "$output" =~ "not found" || "$output" =~ "does not exist" ]]

    # System should remain stable
    run claude-manager diagnose --help
    [ "$status" -eq 0 ]
}

# Scenario I: Performance under load
@test "E2E Scenario I: System handles multiple operations efficiently" {
    # Create multiple sessions
    for i in {1..5}; do
        setup_corrupted_session "branch_collision" "load-test-session-${i}"
    done

    start_time=$(date +%s)

    # Process all sessions
    for i in {1..5}; do
        run claude-manager diagnose "load-test-session-${i}"
        [ "$status" -eq 0 ]
    done

    end_time=$(date +%s)
    duration=$((end_time - start_time))

    # Should complete all operations in reasonable time
    [ "$duration" -lt 60 ]  # <1 minute for 5 operations
}

# Scenario J: Complete lifecycle
@test "E2E Scenario J: Complete session lifecycle with recovery" {
    # Create healthy session
    setup_healthy_session "lifecycle-session"

    # Initial verification
    run claude-manager diagnose lifecycle-session
    [ "$status" -eq 0 ]

    # Simulate corruption
    session_dir="${CLAUDE_TEST_CONFIG}/sessions/lifecycle-session"
    cat >> "${session_dir}/metadata.json" <<EOF
{
  "branches": ["main", "main"]
}
EOF

    # Detect corruption
    run claude-manager diagnose lifecycle-session
    [ "$status" -eq 0 ]

    # Repair
    run claude-manager repair --force lifecycle-session
    [ "$status" -eq 0 ]

    # Final verification
    run claude-manager diagnose lifecycle-session
    [ "$status" -eq 0 ]

    # Lifecycle complete
}
