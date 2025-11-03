#!/usr/bin/env bats
# Infrastructure Verification Tests
# Ensures test helpers and fixtures work correctly

load helpers/setup_test_environment
load helpers/cleanup_test_environment

setup() {
    setup_test_environment
}

teardown() {
    cleanup_test_environment
}

@test "Infrastructure: test environment setup works" {
    [ -d "${CLAUDE_TEST_HOME}" ]
    [ -d "${CLAUDE_TEST_CONFIG}" ]
    [ -f "${CLAUDE_TEST_CONFIG}/config.json" ]
}

@test "Infrastructure: corrupted session generation works" {
    setup_corrupted_session "branch_collision"

    session_dir="${CLAUDE_TEST_CONFIG}/sessions/test-session-123"
    [ -d "${session_dir}" ]
    [ -f "${session_dir}/metadata.json" ]

    # Verify metadata is valid JSON
    run jq . "${session_dir}/metadata.json"
    [ "$status" -eq 0 ]
}

@test "Infrastructure: healthy session generation works" {
    setup_healthy_session "test-healthy"

    session_dir="${CLAUDE_TEST_CONFIG}/sessions/test-healthy"
    [ -d "${session_dir}" ]
    [ -f "${session_dir}/metadata.json" ]

    # Verify health score is 100
    run jq -r '.health_score' "${session_dir}/metadata.json"
    [ "$output" = "100" ]
}

@test "Infrastructure: prerequisites verification works" {
    run verify_prerequisites
    [ "$status" -eq 0 ]
}

@test "Infrastructure: cleanup works" {
    setup_corrupted_session "branch_collision"

    # Verify session exists
    [ -d "${CLAUDE_TEST_CONFIG}/sessions/test-session-123" ]

    # Clean up
    cleanup_session "test-session-123"

    # Verify session removed
    [ ! -d "${CLAUDE_TEST_CONFIG}/sessions/test-session-123" ]
}

@test "Infrastructure: fixtures directory exists" {
    fixtures_dir="${BATS_TEST_DIRNAME}/fixtures/corrupted_sessions"
    [ -d "${fixtures_dir}/branch_collision" ]
    [ -d "${fixtures_dir}/cross_system_inconsistency" ]
    [ -d "${fixtures_dir}/path_mismatch" ]
}

@test "Infrastructure: fixture files are valid JSON" {
    fixtures_dir="${BATS_TEST_DIRNAME}/fixtures/corrupted_sessions"

    for fixture in branch_collision cross_system_inconsistency path_mismatch; do
        metadata_file="${fixtures_dir}/${fixture}/metadata.json"
        [ -f "${metadata_file}" ]

        run jq . "${metadata_file}"
        [ "$status" -eq 0 ]
    done
}

@test "Infrastructure: all corruption types generate correctly" {
    for type in branch_collision cross_system_inconsistency path_mismatch multi_pattern critical_multi_pattern; do
        session_id="test-${type}"
        setup_corrupted_session "${type}" "${session_id}"

        session_dir="${CLAUDE_TEST_CONFIG}/sessions/${session_id}"
        [ -d "${session_dir}" ]
        [ -f "${session_dir}/metadata.json" ]
    done
}
