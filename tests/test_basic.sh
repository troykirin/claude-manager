#!/usr/bin/env bash

# Basic test suite for claude-manager

set -e

TEST_DIR="/tmp/claude-manager-test-$$"
CLAUDE_DIR="$TEST_DIR/.claude"

setup() {
    echo "Setting up test environment..."
    mkdir -p "$CLAUDE_DIR/projects"
    mkdir -p "$TEST_DIR/old-project"
    mkdir -p "$TEST_DIR/new-project"

    # Create test session file
    cat > "$CLAUDE_DIR/projects/-test-project/test-session.jsonl" << 'EOF'
{"cwd": "/tmp/old-project", "type": "test"}
{"cwd": "/tmp/old-project", "type": "test"}
EOF
}

teardown() {
    echo "Cleaning up test environment..."
    rm -rf "$TEST_DIR"
}

test_migrate() {
    echo "Test: Basic migration"

    export CLAUDE_DIR="$CLAUDE_DIR"
    export CLAUDE_INTERACTIVE="false"
    export CLAUDE_DRY_RUN="false"

    # Source the script
    source ../src/claude-manager.sh

    # Run migration
    _migrate_project "/tmp/old-project" "/tmp/new-project" "$CLAUDE_DIR/projects/-test-project"

    # Verify
    if grep -q "/tmp/new-project" "$CLAUDE_DIR/projects/-test-project/test-session.jsonl"; then
        echo "✓ Migration successful"
        return 0
    else
        echo "✗ Migration failed"
        return 1
    fi
}

test_backup() {
    echo "Test: Backup creation"

    export CLAUDE_BACKUP_STRATEGY="file"

    # Create test file
    touch "$TEST_DIR/test-file.jsonl"

    source ../src/claude-manager.sh
    _backup_file "$TEST_DIR/test-file.jsonl"

    if [[ -f "$TEST_DIR/test-file.jsonl.bak" ]]; then
        echo "✓ Backup created"
        return 0
    else
        echo "✗ Backup failed"
        return 1
    fi
}

# Run tests
echo "=== Claude Manager Test Suite ==="
echo ""

setup
trap teardown EXIT

# Run individual tests
test_migrate || exit 1
test_backup || exit 1

echo ""
echo "=== All Tests Passed ==="
