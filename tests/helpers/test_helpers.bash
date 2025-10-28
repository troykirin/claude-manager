#!/usr/bin/env bash

# test_helpers.bash
# Common test utilities for claude-manager test suite

# Create a mock Claude project with sessions
# Usage: create_mock_project <project_name> <cwd_path> [session_count]
create_mock_project() {
    local project_name="$1"
    local cwd_path="$2"
    local session_count="${3:-1}"

    local project_dir="$CLAUDE_DIR/projects/$project_name"
    mkdir -p "$project_dir"

    for i in $(seq 1 "$session_count"); do
        cat > "$project_dir/session-$i.jsonl" <<EOF
{"cwd":"$cwd_path","thread_id":"test-$i","timestamp":"2025-01-15T10:00:00Z"}
{"role":"user","content":"Test message $i","timestamp":"2025-01-15T10:00:01Z"}
{"role":"assistant","content":"Test response $i","timestamp":"2025-01-15T10:00:02Z"}
EOF
    done

    # Create .claude_project file
    cat > "$project_dir/.claude_project" <<EOF
{
  "cwd": "$cwd_path",
  "name": "Test Project $project_name"
}
EOF
}

# Create a mock session with various JSON formatting
# Usage: create_session_with_formatting <file_path> <cwd_path>
create_session_with_formatting() {
    local file_path="$1"
    local cwd_path="$2"

    mkdir -p "$(dirname "$file_path")"

    cat > "$file_path" <<EOF
{"cwd":"$cwd_path","thread_id":"test-1"}
{"cwd" : "$cwd_path","thread_id":"test-2"}
{"cwd"    :    "$cwd_path"   ,"thread_id":"test-3"}
{  "cwd":"$cwd_path"  ,"thread_id":"test-4"}
EOF
}

# Count cwd occurrences in a file
# Usage: count_cwd_in_file <file> <path>
count_cwd_in_file() {
    local file="$1"
    local path="$2"
    grep -c "\"cwd\":\"$path\"" "$file" 2>/dev/null || echo 0
}

# Count cwd occurrences with whitespace tolerance
# Usage: count_cwd_tolerant <file> <path>
count_cwd_tolerant() {
    local file="$1"
    local path="$2"
    grep -cE "\"cwd\"[[:space:]]*:[[:space:]]*\"$path\"" "$file" 2>/dev/null || echo 0
}

# Assert file contains path
# Usage: assert_file_contains_path <file> <path>
assert_file_contains_path() {
    local file="$1"
    local path="$2"
    if ! grep -q "\"cwd\":\"$path\"" "$file" 2>/dev/null; then
        echo "ASSERTION FAILED: File $file does not contain path: $path" >&2
        echo "File contents:" >&2
        cat "$file" >&2
        return 1
    fi
}

# Assert file does NOT contain path
# Usage: assert_file_not_contains_path <file> <path>
assert_file_not_contains_path() {
    local file="$1"
    local path="$2"
    if grep -q "\"cwd\":\"$path\"" "$file" 2>/dev/null; then
        echo "ASSERTION FAILED: File $file unexpectedly contains path: $path" >&2
        echo "File contents:" >&2
        cat "$file" >&2
        return 1
    fi
}

# Verify backup file exists and is valid
# Usage: assert_backup_valid <backup_path>
assert_backup_valid() {
    local backup_path="$1"

    if [ ! -f "$backup_path" ]; then
        echo "ASSERTION FAILED: Backup not found: $backup_path" >&2
        return 1
    fi

    # Verify it's a valid tar.gz
    if ! tar -tzf "$backup_path" >/dev/null 2>&1; then
        echo "ASSERTION FAILED: Backup tar corrupted: $backup_path" >&2
        return 1
    fi
}

# Create a large session file for performance testing
# Usage: create_large_session <file_path> <cwd_path> <message_count>
create_large_session() {
    local file_path="$1"
    local cwd_path="$2"
    local message_count="${3:-1000}"

    mkdir -p "$(dirname "$file_path")"

    for i in $(seq 1 "$message_count"); do
        cat >> "$file_path" <<EOF
{"timestamp":"2025-01-15T$(printf "%02d" $((i % 24))):$(printf "%02d" $((i % 60))):00.000Z","type":"human","text":"Test message $i with some content to make it realistic","cwd":"$cwd_path"}
{"timestamp":"2025-01-15T$(printf "%02d" $((i % 24))):$(printf "%02d" $((i % 60))):01.000Z","type":"assistant","text":"Test response $i with detailed explanation","cwd":"$cwd_path"}
EOF
    done
}

# Mock environment variable setup
# Usage: mock_env <var_name> <value>
mock_env() {
    local var_name="$1"
    local value="$2"
    export "$var_name"="$value"
}

# Restore environment variable
# Usage: restore_env <var_name>
restore_env() {
    local var_name="$1"
    unset "$var_name"
}

# Measure execution time
# Usage: time_command <command>
time_command() {
    local start_time
    local end_time
    local duration

    start_time=$(date +%s)
    "$@"
    end_time=$(date +%s)
    duration=$((end_time - start_time))

    echo "$duration"
}

# Assert duration is less than threshold
# Usage: assert_duration_lt <command> <threshold_seconds>
assert_duration_lt() {
    local threshold="$1"
    shift

    local duration
    duration=$(time_command "$@")

    if [ "$duration" -ge "$threshold" ]; then
        echo "ASSERTION FAILED: Command took ${duration}s (threshold: ${threshold}s)" >&2
        return 1
    fi
}

# Create path with special characters for edge case testing
# Usage: create_special_path <base_dir> <path_type>
create_special_path() {
    local base_dir="$1"
    local path_type="$2"

    case "$path_type" in
        "dots")
            echo "$base_dir/.config/.local/.cache"
            ;;
        "hyphens")
            echo "$base_dir/my-awesome-project"
            ;;
        "mixed")
            echo "$base_dir/project-v1.2.3"
            ;;
        "spaces")
            echo "$base_dir/my project"
            ;;
        "underscores")
            echo "$base_dir/my_project"
            ;;
        "long")
            echo "$base_dir/very/deeply/nested/structure/with/many/levels"
            ;;
        *)
            echo "$base_dir/default"
            ;;
    esac
}

# Verify session file integrity
# Usage: verify_session_integrity <session_file>
verify_session_integrity() {
    local session_file="$1"

    # Check file exists
    [ -f "$session_file" ] || return 1

    # Check each line is valid JSON (basic check)
    while IFS= read -r line; do
        if [ -n "$line" ]; then
            # Basic JSON validation - should start with { and end with }
            [[ "$line" =~ ^\{.*\}$ ]] || return 1
        fi
    done < "$session_file"

    return 0
}

# Generate test paths for matrix testing
# Usage: generate_test_paths
generate_test_paths() {
    cat <<EOF
/Users/tryk/dev/crush
/Users/tryk/.config/nabi
/Users/tryk/Library.Data/stuff
/Users/tryk/my-project
/Users/tryk/nabia/tui/production/riff-dag-tui
/tmp/test-project-123
~/dev/project
EOF
}

# Compare two files ignoring whitespace in JSON
# Usage: json_files_equal <file1> <file2>
json_files_equal() {
    local file1="$1"
    local file2="$2"

    # Simple comparison - could be enhanced with jq if available
    if command -v python3 >/dev/null 2>&1; then
        python3 <<EOF
import json
with open("$file1") as f1, open("$file2") as f2:
    for line1, line2 in zip(f1, f2):
        if line1.strip() and line2.strip():
            j1 = json.loads(line1)
            j2 = json.loads(line2)
            if j1 != j2:
                exit(1)
EOF
        return $?
    else
        # Fallback to simple diff
        diff -w "$file1" "$file2" >/dev/null 2>&1
    fi
}
