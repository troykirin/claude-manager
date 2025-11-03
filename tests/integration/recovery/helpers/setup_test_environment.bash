#!/usr/bin/env bash
# Integration Test Environment Setup
# Part of CM-304 Session Recovery Integration Testing

# Note: Don't use 'set -u' as BATS variables may not be set during load
set -eo pipefail

# Test environment configuration (use defaults if BATS vars not set)
export CLAUDE_TEST_HOME="${BATS_TEST_TMPDIR:-/tmp}/claude-home"
export CLAUDE_TEST_CONFIG="${CLAUDE_TEST_HOME}/.claude"
export CLAUDE_TEST_ARCHIVE="${CLAUDE_TEST_CONFIG}/.archive"

# Setup base test environment
setup_test_environment() {
    # Create test directory structure
    mkdir -p "${CLAUDE_TEST_CONFIG}/"{sessions,cache,.archive}

    # Copy test fixtures if available
    local fixtures_dir="${BATS_TEST_DIRNAME}/../fixtures"
    if [[ -d "${fixtures_dir}" ]]; then
        cp -r "${fixtures_dir}"/* "${CLAUDE_TEST_CONFIG}/"
    fi

    # Set up minimal config
    cat > "${CLAUDE_TEST_CONFIG}/config.json" <<EOF
{
  "version": "1.0",
  "test_mode": true,
  "paths": {
    "sessions": "${CLAUDE_TEST_CONFIG}/sessions",
    "archive": "${CLAUDE_TEST_ARCHIVE}"
  }
}
EOF
}

# Setup a corrupted session for testing
setup_corrupted_session() {
    local corruption_type="${1:-branch_collision}"
    local session_id="${2:-test-session-123}"

    local fixtures_dir="${BATS_TEST_DIRNAME}/../fixtures/corrupted_sessions/${corruption_type}"
    local session_dir="${CLAUDE_TEST_CONFIG}/sessions/${session_id}"

    mkdir -p "${session_dir}"

    # Copy corruption fixture
    if [[ -d "${fixtures_dir}" ]]; then
        cp -r "${fixtures_dir}"/* "${session_dir}/"
    else
        # Generate synthetic corruption
        generate_corrupted_session "${corruption_type}" "${session_id}"
    fi
}

# Generate synthetic corrupted session data
generate_corrupted_session() {
    local corruption_type="$1"
    local session_id="$2"
    local session_dir="${CLAUDE_TEST_CONFIG}/sessions/${session_id}"

    mkdir -p "${session_dir}"

    case "${corruption_type}" in
        branch_collision)
            # Create branch collision scenario
            cat > "${session_dir}/metadata.json" <<EOF
{
  "session_id": "${session_id}",
  "current_branch": "feature/test",
  "branches": ["main", "feature/test", "feature/test"],
  "corruption_patterns": ["duplicate_branch"]
}
EOF
            ;;

        cross_system_inconsistency)
            # Create cross-system inconsistency
            cat > "${session_dir}/metadata.json" <<EOF
{
  "session_id": "${session_id}",
  "git_branch": "main",
  "claude_context_branch": "feature/dev",
  "corruption_patterns": ["branch_mismatch"]
}
EOF
            ;;

        path_mismatch)
            # Create path mismatch scenario
            cat > "${session_dir}/metadata.json" <<EOF
{
  "session_id": "${session_id}",
  "working_directory": "/nonexistent/path",
  "corruption_patterns": ["invalid_path"]
}
EOF
            ;;

        multi_pattern)
            # Create multiple corruption patterns
            cat > "${session_dir}/metadata.json" <<EOF
{
  "session_id": "${session_id}",
  "current_branch": "feature/test",
  "branches": ["main", "feature/test", "feature/test"],
  "working_directory": "/invalid/path",
  "corruption_patterns": ["duplicate_branch", "invalid_path", "metadata_incomplete"]
}
EOF
            ;;

        critical_multi_pattern)
            # Create critical corruption (health <50)
            cat > "${session_dir}/metadata.json" <<EOF
{
  "session_id": "${session_id}",
  "corruption_patterns": [
    "duplicate_branch",
    "invalid_path",
    "metadata_incomplete",
    "session_state_mismatch",
    "corrupted_history"
  ],
  "health_score": 30
}
EOF
            ;;

        *)
            echo "Unknown corruption type: ${corruption_type}" >&2
            return 1
            ;;
    esac
}

# Setup healthy session for comparison
setup_healthy_session() {
    local session_id="${1:-test-session-healthy}"
    local session_dir="${CLAUDE_TEST_CONFIG}/sessions/${session_id}"

    mkdir -p "${session_dir}"

    cat > "${session_dir}/metadata.json" <<EOF
{
  "session_id": "${session_id}",
  "current_branch": "main",
  "working_directory": "$(pwd)",
  "state": "active",
  "health_score": 100
}
EOF
}

# Verify test prerequisites
verify_prerequisites() {
    # Check for required tools
    local required_tools=("jq" "curl")

    for tool in "${required_tools[@]}"; do
        if ! command -v "${tool}" &> /dev/null; then
            echo "Required tool not found: ${tool}" >&2
            return 1
        fi
    done

    return 0
}

# Export functions for BATS
export -f setup_test_environment
export -f setup_corrupted_session
export -f generate_corrupted_session
export -f setup_healthy_session
export -f verify_prerequisites
