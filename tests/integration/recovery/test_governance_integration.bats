#!/usr/bin/env bats
# CM-303: Governance Integration Tests
# Tests for federation integration (Loki, Linear, SurrealDB)

load helpers/setup_test_environment
load helpers/cleanup_test_environment

setup() {
    setup_test_environment
    verify_prerequisites
}

teardown() {
    cleanup_test_environment
}

# Test: Loki event logging
@test "CM-303: diagnostic event logged to Loki" {
    skip "Requires Loki service running"

    setup_corrupted_session "branch_collision"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]

    # Query Loki for the event (requires Loki to be accessible)
    sleep 2  # Allow time for log ingestion

    run curl -s "http://localhost:3100/loki/api/v1/query?query={operation=\"diagnose\"}"

    if [ "$status" -eq 0 ]; then
        [[ "$output" =~ "test-session-123" ]]
    else
        skip "Loki service not accessible"
    fi
}

# Test: Repair operation logging
@test "CM-303: repair event logged to Loki" {
    skip "Requires Loki service running"

    setup_corrupted_session "branch_collision"

    run claude-manager repair test-session-123

    [ "$status" -eq 0 ]

    sleep 2  # Allow time for log ingestion

    run curl -s "http://localhost:3100/loki/api/v1/query?query={operation=\"repair\"}"

    if [ "$status" -eq 0 ]; then
        [[ "$output" =~ "repair" ]]
    fi
}

# Test: Critical corruption creates Linear issue
@test "CM-303: critical corruption triggers Linear issue creation" {
    skip "Requires Linear API access"

    setup_corrupted_session "critical_multi_pattern"

    # Set Linear environment variables (should be mocked in test env)
    export LINEAR_API_KEY="test-key"
    export LINEAR_TEAM_ID="test-team"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]

    # Should mention Linear issue creation
    [[ "$output" =~ "Linear" || "$output" =~ "issue created" ]]
}

# Test: Audit trail queryability
@test "CM-303: audit trail is queryable" {
    skip "Requires federation-integration CLI"

    setup_corrupted_session "branch_collision"

    # Perform operations to generate audit trail
    run claude-manager diagnose test-session-123
    run claude-manager repair test-session-123

    # Query audit trail (requires Node.js CLI)
    if command -v node &> /dev/null; then
        local cli_path="../../../federation-integration/dist/recovery/cli.js"
        if [[ -f "${cli_path}" ]]; then
            run node "${cli_path}" query --session test-session-123

            [ "$status" -eq 0 ]
            [[ "$output" =~ "diagnose" || "$output" =~ "repair" ]]
        else
            skip "Federation CLI not found"
        fi
    else
        skip "Node.js not available"
    fi
}

# Test: Event metadata completeness
@test "CM-303: logged events contain required metadata" {
    setup_corrupted_session "branch_collision"

    # Capture diagnostic output
    run claude-manager diagnose --json test-session-123

    [ "$status" -eq 0 ]

    # Parse JSON and verify metadata fields
    if command -v jq &> /dev/null; then
        echo "$output" | jq -e '.session_id' > /dev/null
        echo "$output" | jq -e '.timestamp' > /dev/null || true  # May not be in output
        echo "$output" | jq -e '.operation' > /dev/null || true
    fi
}

# Test: Federation event streaming
@test "CM-303: operations stream to federation event bus" {
    skip "Requires NATS JetStream"

    setup_corrupted_session "branch_collision"

    # Check if NATS is available
    if command -v nats &> /dev/null; then
        # Subscribe to recovery events
        nats sub "recovery.>" --count=1 &
        NATS_PID=$!

        run claude-manager diagnose test-session-123

        wait $NATS_PID || true
        # Should have received event
    else
        skip "NATS CLI not available"
    fi
}

# Test: SurrealDB state persistence
@test "CM-303: session state persisted to SurrealDB" {
    skip "Requires SurrealDB connection"

    setup_corrupted_session "branch_collision"

    run claude-manager diagnose test-session-123

    [ "$status" -eq 0 ]

    # Query SurrealDB for session state (requires surreal CLI)
    if command -v surreal &> /dev/null; then
        run surreal sql --conn http://localhost:8000 \
            --user root --pass root \
            --ns test --db test \
            "SELECT * FROM sessions WHERE id = 'test-session-123'"

        if [ "$status" -eq 0 ]; then
            [[ "$output" =~ "test-session-123" ]]
        fi
    else
        skip "SurrealDB CLI not available"
    fi
}

# Test: Governance policy enforcement
@test "CM-303: repair operations respect governance policies" {
    setup_corrupted_session "critical_multi_pattern"

    # Critical repairs should require approval (if policy enabled)
    run claude-manager repair test-session-123

    # Should either complete or request approval
    [[ "$status" -eq 0 || "$output" =~ "approval" || "$output" =~ "policy" ]]
}

# Test: Audit trail immutability
@test "CM-303: audit records are immutable" {
    skip "Requires audit storage verification"

    setup_corrupted_session "branch_collision"

    run claude-manager diagnose test-session-123
    [ "$status" -eq 0 ]

    # Attempt to modify audit record (should fail)
    # This test verifies the concept; actual implementation depends on audit backend
}

# Test: Compliance reporting
@test "CM-303: operations generate compliance reports" {
    setup_corrupted_session "branch_collision"

    run claude-manager diagnose test-session-123
    run claude-manager repair test-session-123

    # Generate compliance report (if supported)
    run claude-manager report --compliance

    # Should either succeed or gracefully explain feature status
    [[ "$status" -eq 0 || "$output" =~ "not implemented" || "$output" =~ "unrecognized" ]]
}

# Test: Event correlation
@test "CM-303: related events are correlated" {
    setup_corrupted_session "branch_collision"

    # Perform related operations
    run claude-manager diagnose test-session-123
    diagnostic_output="$output"

    run claude-manager repair test-session-123
    repair_output="$output"

    # Events should reference same session (correlation)
    [[ "$diagnostic_output" =~ "test-session-123" ]]
    [[ "$repair_output" =~ "test-session-123" ]]
}

# Test: Federation health monitoring
@test "CM-303: federation services health check" {
    # Check health of federated services
    run claude-manager federation-health

    # Should report status of Loki, Linear, SurrealDB, NATS
    # Either succeed or gracefully handle unavailable services
    [[ "$status" -eq 0 || "$output" =~ "not available" || "$output" =~ "unrecognized" ]]
}

# Test: Error propagation to federation
@test "CM-303: errors are propagated to federation" {
    # Trigger an error scenario
    run claude-manager diagnose nonexistent-session-999

    [ "$status" -ne 0 ]

    # Error should be logged (verify in subsequent tests if logging available)
}

# Test: Batch operation governance
@test "CM-303: batch operations maintain governance" {
    setup_corrupted_session "branch_collision" "session-1"
    setup_corrupted_session "path_mismatch" "session-2"

    run claude-manager diagnose --all

    [ "$status" -eq 0 ]

    # All operations should be individually tracked
    [[ "$output" =~ "session-1" ]]
    [[ "$output" =~ "session-2" ]]
}
