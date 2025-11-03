#!/usr/bin/env bash
# Governance Integration Layer for Session Recovery
# Provides bash wrappers for TypeScript audit trail

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-${(%):-%x}}")" && pwd)"
RECOVERY_CLI="${RECOVERY_CLI:-${SCRIPT_DIR}/../federation-integration/dist/recovery/cli.js}"
RECOVERY_AUDIT_CONFIG="${RECOVERY_AUDIT_CONFIG:-}"
LOKI_URL="${LOKI_URL:-http://localhost:3100}"
LINEAR_TEAM_ID="${LINEAR_TEAM_ID:-}"
ENABLE_LINEAR_ISSUES="${ENABLE_LINEAR_ISSUES:-false}"
SEVERITY_THRESHOLD="${SEVERITY_THRESHOLD:-50}"

# Export environment for CLI
export RECOVERY_AUDIT_CONFIG
export LOKI_URL
export LINEAR_TEAM_ID
export ENABLE_LINEAR_ISSUES
export SEVERITY_THRESHOLD

# Check if CLI is available
check_cli() {
    if [[ ! -f "$RECOVERY_CLI" ]]; then
        echo "ERROR: Recovery CLI not found at $RECOVERY_CLI" >&2
        echo "Run: cd federation-integration && bun build src/recovery/cli.ts --outdir ./dist/recovery --target node" >&2
        return 1
    fi
}

# Log recovery event
# Usage: log_recovery_event <operation> <session_id> <outcome> [health_score] [corruption_patterns] [backup_location] [duration_ms] [error_message]
log_recovery_event() {
    local operation="$1"
    local session_id="$2"
    local outcome="$3"
    local health_score="${4:-}"
    local corruption_patterns="${5:-}"
    local backup_location="${6:-}"
    local duration_ms="${7:-}"
    local error_message="${8:-}"

    check_cli || return 1

    local args=(
        "--operation" "$operation"
        "--session" "$session_id"
        "--outcome" "$outcome"
    )

    [[ -n "$health_score" ]] && args+=("--health-score" "$health_score")
    [[ -n "$corruption_patterns" ]] && args+=("--corruption-patterns" "$corruption_patterns")
    [[ -n "$backup_location" ]] && args+=("--backup-location" "$backup_location")
    [[ -n "$duration_ms" ]] && args+=("--duration" "$duration_ms")
    [[ -n "$error_message" ]] && args+=("--error" "$error_message")

    node "$RECOVERY_CLI" log "${args[@]}"
}

# Convenience wrappers for specific operations

# Log diagnostic event
# Usage: log_diagnostic_event <session_id> <outcome> <health_score> [corruption_patterns]
log_diagnostic_event() {
    local session_id="$1"
    local outcome="$2"
    local health_score="$3"
    local corruption_patterns="${4:-}"

    log_recovery_event "diagnose" "$session_id" "$outcome" "$health_score" "$corruption_patterns"
}

# Log repair event
# Usage: log_repair_event <session_id> <outcome> <health_score> [backup_location] [duration_ms]
log_repair_event() {
    local session_id="$1"
    local outcome="$2"
    local health_score="$3"
    local backup_location="${4:-}"
    local duration_ms="${5:-}"

    log_recovery_event "repair" "$session_id" "$outcome" "$health_score" "" "$backup_location" "$duration_ms"
}

# Log verify event
# Usage: log_verify_event <session_id> <outcome> <health_score>
log_verify_event() {
    local session_id="$1"
    local outcome="$2"
    local health_score="$3"

    log_recovery_event "verify" "$session_id" "$outcome" "$health_score"
}

# Log rollback event
# Usage: log_rollback_event <session_id> <outcome> <backup_location> [duration_ms] [error_message]
log_rollback_event() {
    local session_id="$1"
    local outcome="$2"
    local backup_location="$3"
    local duration_ms="${4:-}"
    local error_message="${5:-}"

    log_recovery_event "rollback" "$session_id" "$outcome" "" "" "$backup_location" "$duration_ms" "$error_message"
}

# Generate audit report
# Usage: generate_audit_report <session_id> [format]
generate_audit_report() {
    local session_id="$1"
    local format="${2:-text}"

    check_cli || return 1

    node "$RECOVERY_CLI" report --session "$session_id" --format "$format"
}

# Query recovery history
# Usage: query_recovery_history [--session <id>] [--operation <type>] [--outcome <result>] [--format <type>]
query_recovery_history() {
    check_cli || return 1

    node "$RECOVERY_CLI" query "$@"
}

# Get recovery metrics
# Usage: get_recovery_metrics [format]
get_recovery_metrics() {
    local format="${1:-text}"

    check_cli || return 1

    node "$RECOVERY_CLI" metrics --format "$format"
}

# Recovery operation wrapper with automatic logging
# Usage: with_recovery_logging <operation> <session_id> <command...>
with_recovery_logging() {
    local operation="$1"
    local session_id="$2"
    shift 2

    local start_time
    start_time=$(date +%s%3N)

    local outcome="success"
    local error_message=""

    if "$@"; then
        outcome="success"
    else
        outcome="failure"
        error_message="Command failed: $*"
    fi

    local end_time
    end_time=$(date +%s%3N)
    local duration=$((end_time - start_time))

    log_recovery_event "$operation" "$session_id" "$outcome" "" "" "" "$duration" "$error_message"
}

# Example usage in recovery scripts:
#
# # Diagnose session
# log_diagnostic_event "$session_id" "success" "75" "missing-blocks,corrupted-metadata"
#
# # Repair with logging
# with_recovery_logging "repair" "$session_id" repair_session_command "$session_id"
#
# # Generate report
# generate_audit_report "$session_id"
