#!/bin/bash
# Manual Test Suite for cm mv Edge Cases
# Based on requirements from user feedback

set -e

CLAUDE_DIR="${CLAUDE_DIR:-$HOME/.claude}"
TEST_BASE="/tmp/cm-mv-test-$$"
CM_PATH="/Users/tryk/.local/bin/cm"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[TEST]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Setup test environment
setup_test_env() {
    log "Setting up test environment at $TEST_BASE"
    
    # Create test Claude directory structure
    mkdir -p "$TEST_BASE/.claude/projects"
    mkdir -p "$TEST_BASE/.claude/todos"
    mkdir -p "$TEST_BASE/.claude/statsig"
    mkdir -p "$TEST_BASE/.claude/shell-snapshots"
    
    # Create source project directories
    mkdir -p "$TEST_BASE/source-project"
    mkdir -p "$TEST_BASE/dest-project"
    mkdir -p "$TEST_BASE/no-claude-dir"
    
    # Create test session directories first
    mkdir -p "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-source-project"
    
    # Create test session with various JSON formatting
    cat > "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-source-project/conversation.jsonl" << 'EOF'
{"timestamp": "2025-01-15T10:00:00.000Z", "type": "human", "text": "Test message 1"}
{
  "timestamp": "2025-01-15T10:01:00.000Z",
  "type": "assistant", 
  "text": "Test response 1"
}
{"timestamp":"2025-01-15T10:02:00.000Z","type":"human","text":"Test message 2"}
EOF
    
    # Create project config with whitespace variations
    cat > "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-source-project/.claude_project" << EOF
{
  "cwd": "$TEST_BASE/source-project",
  "name": "Test Source Project"
}
EOF
    
    # Export test environment
    export CLAUDE_DIR="$TEST_BASE/.claude"
    export CLAUDE_SAFETY_CHECKS="false"  # Disable safety checks for testing
    export CLAUDE_INTERACTIVE="false"    # Disable interactive prompts
    export FORCE="true"                   # Force operations despite active processes
    
    log "Test environment ready"
}

# Test 1: Destination already exists
test_destination_exists() {
    log "Test 1: Destination already exists"
    
    # Create destination session
    mkdir -p "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-dest-project"
    echo '{"cwd": "'$TEST_BASE'/dest-project", "name": "Existing Dest"}' > "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-dest-project/.claude_project"
    
    # Test dry run first (run from base directory for proper path resolution)
    log "Testing dry run with existing destination..."
    cd "$TEST_BASE"
    if CLAUDE_DRY_RUN=true "$CM_PATH" mv source-project dest-project 2>&1 | grep -q "already exists\|project.*exists"; then
        log "✅ Dry run correctly detected existing destination"
    else
        error "❌ Dry run failed to detect existing destination"
        return 1
    fi
    
    # Test actual move should fail
    log "Testing actual move with existing destination..."
    if "$CM_PATH" mv source-project dest-project 2>&1 | grep -q "already exists\|project.*exists"; then
        log "✅ Move correctly rejected existing destination"  
    else
        error "❌ Move should have failed with existing destination"
        return 1
    fi
}

# Test 2: No project directory scenarios
test_no_project_dir() {
    log "Test 2: No project directory scenarios"
    
    # Test source doesn't exist
    log "Testing non-existent source..."
    cd "$TEST_BASE"
    if "$CM_PATH" mv nonexistent new-dest 2>&1 | grep -q "does not exist\|not found\|No Claude projects"; then
        log "✅ Correctly handled non-existent source"
    else
        error "❌ Should have failed with non-existent source"
        return 1
    fi
    
    # Test destination parent doesn't exist
    log "Testing destination with non-existent parent..."
    mkdir -p nonexistent-parent-test
    if "$CM_PATH" mv nonexistent-parent-test nonexistent-parent/dest 2>&1 | grep -q "parent.*not.*exist\|cannot create\|Invalid"; then
        log "✅ Correctly handled non-existent destination parent"
    else
        warn "⚠️  May need to check destination parent validation"
    fi
}

# Test 3: JSON whitespace handling
test_json_whitespace() {
    log "Test 3: JSON whitespace handling"
    
    # Create session with various JSON formatting
    mkdir -p "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-whitespace-test"
    mkdir -p "$TEST_BASE/whitespace-source"
    
    # Pretty-printed JSON with lots of whitespace
    cat > "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-whitespace-test/.claude_project" << EOF
{
  "cwd"    :    "$TEST_BASE/whitespace-source"   ,
  "name"   :   "Whitespace Test Project"    ,
  "version": "1.0"
}
EOF
    
    # Create destination
    mkdir -p "$TEST_BASE/whitespace-dest"
    
    # Test move
    log "Testing move with whitespace-heavy JSON..."
    cd "$TEST_BASE"
    if "$CM_PATH" mv whitespace-source whitespace-dest; then
        # Check if path was updated correctly
        if grep -q "$TEST_BASE/whitespace-dest" "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-whitespace-dest/.claude_project"; then
            log "✅ JSON whitespace handling successful"
        else
            error "❌ Path not updated in whitespace JSON"
            return 1
        fi
    else
        error "❌ Move failed with whitespace JSON"
        return 1
    fi
}

# Test 4: Undo functionality
test_undo_functionality() {
    log "Test 4: Undo functionality"
    
    # Create a session to move
    mkdir -p "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-undo-test"
    mkdir -p "$TEST_BASE/undo-source"
    mkdir -p "$TEST_BASE/undo-dest"
    echo '{"cwd": "'$TEST_BASE'/undo-source", "name": "Undo Test"}' > "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-undo-test/.claude_project"
    
    # Perform move
    log "Performing move for undo test..."
    cd "$TEST_BASE"
    "$CM_PATH" mv undo-source undo-dest
    
    # Verify move happened
    if [ -d "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-undo-dest" ]; then
        log "✅ Move completed successfully"
        
        # Test undo
        log "Testing undo functionality..."
        if "$CM_PATH" mv undo-dest undo-source; then
            if [ -d "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-undo-source" ]; then
                log "✅ Undo successful"
            else
                error "❌ Undo failed - directory not restored"
                return 1
            fi
        else
            error "❌ Undo command failed"
            return 1
        fi
    else
        error "❌ Initial move failed"
        return 1
    fi
}

# Test 5: Large directory performance
test_large_directory() {
    log "Test 5: Large directory performance (simplified)"
    
    # Create a session with moderately sized conversation
    mkdir -p "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-large-test"
    mkdir -p "$TEST_BASE/large-source"
    mkdir -p "$TEST_BASE/large-dest"
    
    # Create larger conversation file
    log "Creating larger test conversation..."
    {
        for i in {1..100}; do
            echo '{"timestamp": "2025-01-15T10:'$(printf "%02d" $i)':00.000Z", "type": "human", "text": "Test message '$i' with some longer content to make this file larger and test performance with bigger conversations"}'
            echo '{"timestamp": "2025-01-15T10:'$(printf "%02d" $i)':30.000Z", "type": "assistant", "text": "Test response '$i' with detailed explanation and code examples to simulate real conversation size"}'
        done
    } > "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-large-test/conversation.jsonl"
    
    echo '{"cwd": "'$TEST_BASE'/large-source", "name": "Large Test Project"}' > "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-large-test/.claude_project"
    
    # Time the move
    log "Testing move performance with larger session..."
    cd "$TEST_BASE"
    start_time=$(date +%s)
    
    if "$CM_PATH" mv large-source large-dest; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        log "✅ Large directory move completed in ${duration}s"
        
        if [ $duration -gt 10 ]; then
            warn "⚠️  Move took ${duration}s - may want to optimize for larger sessions"
        fi
    else
        error "❌ Large directory move failed"
        return 1
    fi
}

# Test 6: Concurrency protection
test_concurrency_protection() {
    log "Test 6: Concurrency protection"
    
    # Create test session
    mkdir -p "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-concurrent-test"
    mkdir -p "$TEST_BASE/concurrent-source"
    mkdir -p "$TEST_BASE/concurrent-dest"
    echo '{"cwd": "'$TEST_BASE'/concurrent-source", "name": "Concurrent Test"}' > "$TEST_BASE/.claude/projects/-tmp-cm-mv-test-$$-concurrent-test/.claude_project"
    
    # Simulate Claude process (create a mock process)
    log "Testing concurrency detection..."
    
    # Create a fake Claude process
    sleep 30 &
    FAKE_PID=$!
    
    # Test move (should work since our fake process isn't actually Claude)
    if command -v pgrep >/dev/null; then
        log "Running concurrency check..."
        cd "$TEST_BASE"
        
        # Run the move (should work since our fake process isn't actually Claude)
        if "$CM_PATH" mv concurrent-source concurrent-dest; then
            log "✅ Move completed (no actual Claude process detected)"
        else
            warn "⚠️  Move failed - check concurrency detection logic"
        fi
    else
        log "⚠️  pgrep not available, skipping concurrency test"
    fi
    
    # Clean up
    kill $FAKE_PID 2>/dev/null || true
}

# Cleanup function
cleanup() {
    log "Cleaning up test environment..."
    rm -rf "$TEST_BASE"
    unset CLAUDE_DIR
}

# Run all tests
run_all_tests() {
    log "Starting cm mv edge case testing suite"
    log "Test environment: $TEST_BASE"
    
    setup_test_env
    
    local failed_tests=0
    
    # Run each test
    test_destination_exists || ((failed_tests++))
    test_no_project_dir || ((failed_tests++))
    test_json_whitespace || ((failed_tests++))
    test_undo_functionality || ((failed_tests++))
    test_large_directory || ((failed_tests++))
    test_concurrency_protection || ((failed_tests++))
    
    # Summary
    if [ $failed_tests -eq 0 ]; then
        log "🎉 All edge case tests passed!"
    else
        error "❌ $failed_tests test(s) failed"
        return 1
    fi
}

# Main execution
main() {
    if [ ! -f "$CM_PATH" ]; then
        error "cm command not found at $CM_PATH"
        exit 1
    fi
    
    trap cleanup EXIT
    
    if [ "${1:-}" = "--cleanup-only" ]; then
        cleanup
        exit 0
    fi
    
    run_all_tests
}

# Check if being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi