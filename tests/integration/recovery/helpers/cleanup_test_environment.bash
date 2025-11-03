#!/usr/bin/env bash
# Integration Test Environment Cleanup
# Part of CM-304 Session Recovery Integration Testing

# Note: Don't use 'set -u' as BATS variables may not be set during load
set -eo pipefail

# Clean up test environment after tests
cleanup_test_environment() {
    # Remove test directories if they exist
    if [[ -n "${CLAUDE_TEST_HOME:-}" ]] && [[ -d "${CLAUDE_TEST_HOME}" ]]; then
        rm -rf "${CLAUDE_TEST_HOME}"
    fi

    # Clean up any test artifacts
    if [[ -n "${BATS_TEST_TMPDIR:-}" ]] && [[ -d "${BATS_TEST_TMPDIR}" ]]; then
        find "${BATS_TEST_TMPDIR}" -type f -name "*.test.log" -delete 2>/dev/null || true
    fi
}

# Clean up specific session data
cleanup_session() {
    local session_id="$1"
    local session_dir="${CLAUDE_TEST_CONFIG}/sessions/${session_id}"

    if [[ -d "${session_dir}" ]]; then
        rm -rf "${session_dir}"
    fi
}

# Clean up archive directory
cleanup_archive() {
    if [[ -d "${CLAUDE_TEST_ARCHIVE}" ]]; then
        rm -rf "${CLAUDE_TEST_ARCHIVE}"/*
    fi
}

# Verify cleanup was successful
verify_cleanup() {
    local issues=0

    if [[ -d "${CLAUDE_TEST_CONFIG}/sessions" ]]; then
        local remaining_sessions
        remaining_sessions=$(find "${CLAUDE_TEST_CONFIG}/sessions" -type d -mindepth 1 | wc -l)
        if [[ "${remaining_sessions}" -gt 0 ]]; then
            echo "Warning: ${remaining_sessions} test sessions not cleaned up" >&2
            ((issues++))
        fi
    fi

    return "${issues}"
}

# Export functions for BATS
export -f cleanup_test_environment
export -f cleanup_session
export -f cleanup_archive
export -f verify_cleanup
