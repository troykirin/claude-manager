#!/usr/bin/env bash
# Test helper for claude-manager tests

# Setup BATS test environment
setup_bats() {
    # Set test mode
    export CLAUDE_TEST_MODE=1
    export CLAUDE_DEBUG=0
}

# Common test utilities
assert_file_exists() {
    local file="$1"
    [[ -f "$file" ]] || {
        echo "Expected file not found: $file"
        return 1
    }
}

assert_dir_exists() {
    local dir="$1"
    [[ -d "$dir" ]] || {
        echo "Expected directory not found: $dir"
        return 1
    }
}

# Mock functions for testing
mock_uuid() {
    echo "550e8400-e29b-41d4-a716-446655440000"
}
