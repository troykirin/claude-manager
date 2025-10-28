# Claude Manager - Comprehensive Test Automation Strategy

**Version:** 1.0
**Date:** 2025-10-27
**Status:** Production Ready
**Critical Priority:** Path Transformer Protection

---

## Executive Summary

This document defines a comprehensive test automation strategy for claude-manager, with **CRITICAL** focus on protecting the path transformer function `_suggest_project_dir_for()`. This function is the linchpin of the entire system - if it breaks, `/resume` breaks for all users and session history becomes inaccessible.

### Success Criteria
- 100% test coverage on `_suggest_project_dir_for()` transformer
- 95%+ coverage on migration logic (`_migrate_project()`)
- 90%+ coverage on backup/undo functionality
- Zero regressions on critical path functions
- All tests pass before any commit to main branch

---

## 1. Test Architecture

### 1.1 Test Pyramid

```
                    ╔════════════════╗
                    ║  E2E Tests     ║  ← 10% of tests
                    ║  (5-10 tests)  ║     Full workflows
                    ╚════════════════╝
                   ╔══════════════════╗
                   ║ Integration Tests║  ← 30% of tests
                   ║   (30-50 tests)  ║     Component interaction
                   ╚══════════════════╝
              ╔═════════════════════════╗
              ║     Unit Tests          ║  ← 60% of tests
              ║    (100-150 tests)      ║     Function-level
              ╚═════════════════════════╝
```

### 1.2 Test Organization

```
tests/
├── unit/
│   ├── test_path_transformer.bats         # CRITICAL - 100% coverage required
│   ├── test_backup_creation.bats          # Backup integrity tests
│   ├── test_flag_parsing.bats             # Agent mode flags
│   ├── test_undo_logic.bats               # Undo state management
│   └── test_path_resolution.bats          # Path normalization
├── integration/
│   ├── test_migration_workflow.bats       # Full migration flow
│   ├── test_move_and_rename.bats          # Move + rename coordination
│   ├── test_backup_restoration.bats       # Backup/restore cycle
│   └── test_agent_mode.bats               # Non-interactive operations
├── regression/
│   ├── test_critical_paths.bats           # Known issue prevention
│   ├── test_transformer_regression.bats   # Transformer-specific regressions
│   └── test_session_integrity.bats        # Data loss prevention
├── e2e/
│   ├── test_full_user_journey.bats        # Complete user workflows
│   └── test_multi_project_scenarios.bats  # Complex real-world cases
├── fixtures/
│   ├── sample_sessions/                   # Test session data
│   ├── project_structures/                # Test project layouts
│   └── edge_case_paths/                   # Problematic path names
└── helpers/
    ├── test_helpers.bash                  # Common test utilities
    ├── mock_claude_env.bash               # Claude environment simulation
    └── assertions.bash                    # Custom assertions
```

---

## 2. Critical Test Cases: Path Transformer

### 2.1 Function Under Test

```bash
_suggest_project_dir_for() {
    local new_path="$1"

    # Expand ~ to full home path
    new_path="${new_path/#\~/$HOME}"

    # Convert absolute path to Claude Code project naming convention
    # Claude uses single dash for path separators and dots
    # Example: /Users/tryk/.claude/sync → -Users-tryk--claude-sync
    local encoded_name
    encoded_name=$(echo "$new_path" | sed 's|^/||' | sed 's|\.|-|g' | sed 's|/|-|g')
    echo "$CLAUDE_DIR/projects/-${encoded_name}"
}
```

### 2.2 Transformation Rules (MUST NOT CHANGE)

| Rule | Transform | Example |
|------|-----------|---------|
| Strip leading slash | `/foo` → `foo` | `/Users` → `Users` |
| Dots to dashes | `.` → `-` | `.config` → `-config` |
| Slashes to dashes | `/` → `-` | `foo/bar` → `foo-bar` |
| Prefix with dash | `path` → `-path` | `Users` → `-Users` |

### 2.3 Test Matrix (20+ Path Variations)

#### Simple Paths
```bash
@test "transformer: simple absolute path" {
    result=$(_suggest_project_dir_for "/Users/tryk/dev/crush")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-dev-crush" ]
}

@test "transformer: home tilde expansion" {
    result=$(_suggest_project_dir_for "~/dev/crush")
    expected="$CLAUDE_DIR/projects/-${HOME#/}-dev-crush"
    [ "$result" = "$expected" ]
}
```

#### Paths with Dots
```bash
@test "transformer: path with leading dot directory" {
    result=$(_suggest_project_dir_for "/Users/tryk/.config/nabi")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk--config-nabi" ]
}

@test "transformer: path with multiple dots" {
    result=$(_suggest_project_dir_for "/Users/tryk/Library.Data/stuff")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-Library-Data-stuff" ]
}

@test "transformer: hidden directory chain" {
    result=$(_suggest_project_dir_for "/Users/tryk/.local/.cache/.tmp")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk--local--cache--tmp" ]
}

@test "transformer: file extension in path" {
    result=$(_suggest_project_dir_for "/Users/tryk/project.v1.2")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project-v1-2" ]
}
```

#### Paths with Hyphens
```bash
@test "transformer: path with single hyphen" {
    result=$(_suggest_project_dir_for "/Users/tryk/my-project")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-my-project" ]
}

@test "transformer: path with multiple hyphens" {
    result=$(_suggest_project_dir_for "/Users/tryk/my-awesome-project")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-my-awesome-project" ]
}

@test "transformer: hyphen and dot combination" {
    result=$(_suggest_project_dir_for "/Users/tryk/project-v1.2")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project-v1-2" ]
}
```

#### Complex Real-World Paths
```bash
@test "transformer: deep nested project path" {
    result=$(_suggest_project_dir_for "/Users/tryk/nabia/tui/production/riff-dag-tui")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-nabia-tui-production-riff-dag-tui" ]
}

@test "transformer: config directory typical case" {
    result=$(_suggest_project_dir_for "/Users/tryk/.config/nabi")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk--config-nabi" ]
}

@test "transformer: node_modules style path" {
    result=$(_suggest_project_dir_for "/Users/tryk/project/node_modules/@types/node")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project-node_modules-@types-node" ]
}
```

#### Very Long Paths
```bash
@test "transformer: extremely long path" {
    long_path="/Users/tryk/very/deeply/nested/project/structure/with/many/levels/of/directories/that/goes/on/and/on"
    result=$(_suggest_project_dir_for "$long_path")
    expected="$CLAUDE_DIR/projects/-Users-tryk-very-deeply-nested-project-structure-with-many-levels-of-directories-that-goes-on-and-on"
    [ "$result" = "$expected" ]
}

@test "transformer: 255 character path limit" {
    # Most filesystems have 255 char filename limits
    base="/Users/tryk/"
    # Create path that results in ~240 char project name
    long_segment="$(printf 'a%.0s' {1..200})"
    result=$(_suggest_project_dir_for "${base}${long_segment}")
    [[ ${#result} -lt 255 ]]
}
```

#### Edge Cases
```bash
@test "transformer: trailing slash is normalized" {
    result=$(_suggest_project_dir_for "/Users/tryk/project/")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project" ]
}

@test "transformer: double slash in path" {
    result=$(_suggest_project_dir_for "/Users//tryk/project")
    # Should normalize to single dash between Users and tryk
    [[ "$result" == *"-Users-tryk-project"* ]]
}

@test "transformer: root directory" {
    result=$(_suggest_project_dir_for "/")
    [ "$result" = "$CLAUDE_DIR/projects/-" ]
}

@test "transformer: spaces in path" {
    result=$(_suggest_project_dir_for "/Users/tryk/my project")
    # Spaces become part of the name (no special handling)
    [[ "$result" == *"my project"* ]]
}

@test "transformer: unicode in path" {
    result=$(_suggest_project_dir_for "/Users/tryk/プロジェクト")
    # Should handle UTF-8 gracefully
    [[ -n "$result" ]]
}
```

#### Idempotency Tests
```bash
@test "transformer: idempotent on same input" {
    result1=$(_suggest_project_dir_for "/Users/tryk/project")
    result2=$(_suggest_project_dir_for "/Users/tryk/project")
    [ "$result1" = "$result2" ]
}

@test "transformer: different inputs different outputs" {
    result1=$(_suggest_project_dir_for "/Users/tryk/project1")
    result2=$(_suggest_project_dir_for "/Users/tryk/project2")
    [ "$result1" != "$result2" ]
}

@test "transformer: symlink resolution not performed" {
    # Transformer should NOT resolve symlinks, just encode the path
    result=$(_suggest_project_dir_for "/tmp/link-to-project")
    [ "$result" = "$CLAUDE_DIR/projects/-tmp-link-to-project" ]
}
```

---

## 3. Critical Test Cases: Migration Logic

### 3.1 Function Under Test

```bash
_migrate_project() {
    local old_path="$1"
    local new_path="$2"
    local project_dir="$3"

    # Find sessions, count occurrences, backup, replace paths
    # MUST handle whitespace-tolerant JSON
    # MUST verify all replacements
    # MUST report accurate metrics
}
```

### 3.2 Migration Test Cases

```bash
@test "migration: basic path replacement" {
    # Setup test session with old path
    # Run migration
    # Verify all occurrences replaced
    # Verify backup created
    # Verify metrics accurate
}

@test "migration: whitespace-tolerant JSON matching" {
    # Test various JSON formatting:
    # {"cwd":"path"}
    # {"cwd" : "path"}
    # {"cwd"  :  "path"}
    # {  "cwd":"path"  }
}

@test "migration: zero occurrences is not an error" {
    # Migrating a path that doesn't exist in sessions should succeed gracefully
}

@test "migration: multiple sessions with same path" {
    # All sessions should be updated
}

@test "migration: mixed paths in same session" {
    # Only target path should be replaced, others untouched
}

@test "migration: backup strategy file vs project" {
    # Test both backup modes
}

@test "migration: dry run mode no modifications" {
    # Verify no files changed in dry run
}

@test "migration: metrics match actual replacements" {
    # Count before = count after replacement
}

@test "migration: no python3 fallback handling" {
    # Error message if python3 missing
}

@test "migration: corrupted session file handling" {
    # Invalid JSON should not crash migration
}

@test "migration: very large session file performance" {
    # 10MB+ session files should complete in reasonable time
}
```

---

## 4. Critical Test Cases: Backup & Undo

### 4.1 Backup Creation Tests

```bash
@test "backup: XDG-compliant path" {
    backup=$(_create_project_backup "$project_dir")
    [[ "$backup" == "$XDG_STATE_HOME/nabi/backups/"* ]]
}

@test "backup: project name with leading dash" {
    # Project names start with - (e.g., -Users-tryk-project)
    # tar -czf must use -- separator
    project_dir="$CLAUDE_DIR/projects/-test-project"
    backup=$(_create_project_backup "$project_dir")
    [ -f "$backup" ]
}

@test "backup: tar archive integrity" {
    backup=$(_create_project_backup "$project_dir")
    # Verify tar can extract
    tar -tzf "$backup" >/dev/null
}

@test "backup: timestamp in filename" {
    backup=$(_create_project_backup "$project_dir")
    [[ "$backup" =~ [0-9]{8}_[0-9]{6} ]]
}

@test "backup: restore preserves all files" {
    # Backup, modify, restore, verify identical
}
```

### 4.2 Undo Tests

```bash
@test "undo: restore source directory" {
    # Move source, undo, verify original location
}

@test "undo: restore project directory" {
    # Move project, undo, verify original location
}

@test "undo: restore session paths" {
    # Update sessions, undo, verify old paths restored
}

@test "undo: undo file format validation" {
    # Corrupted undo file should error gracefully
}

@test "undo: no undo file exists" {
    # Should error clearly
}

@test "undo: source_moved=false handling" {
    # If source wasn't moved, don't try to restore it
}
```

---

## 5. Critical Test Cases: Flag Parsing

### 5.1 Agent Mode Flags

```bash
@test "flags: --no-confirm disables prompts" {
    # Verify INTERACTIVE=false
}

@test "flags: --agent-mode sets both flags" {
    # AGENT_MODE=true and INTERACTIVE=false
}

@test "flags: --verbose enables debug output" {
    # CLAUDE_DEBUG=1
}

@test "flags: --backup-strategy=file" {
    # FORCE_BACKUP_STRATEGY=file
}

@test "flags: flags don't consume positional args" {
    # cm migrate old new --no-confirm
    # old and new should be positional[0] and positional[1]
}

@test "flags: multiple flags combined" {
    # --no-confirm --verbose --agent-mode
}

@test "flags: unknown flags ignored" {
    # --unknown-flag should not error
}
```

---

## 6. Integration Tests

### 6.1 Full Migration Workflow

```bash
@test "integration: migrate command end-to-end" {
    # Setup: Create project with sessions
    # Run: cm migrate old new project_dir
    # Verify: Backup created, paths updated, metrics accurate
}

@test "integration: move command end-to-end" {
    # Setup: Create source and project dirs
    # Run: cm move old new
    # Verify: Source moved, project moved, sessions updated
}

@test "integration: move then undo" {
    # Move, verify, undo, verify restoration
}

@test "integration: auto-detect project directory" {
    # Run migration without specifying project, verify auto-detection
}

@test "integration: sync project dir name with path" {
    # After migration, project dir should match new path encoding
}
```

### 6.2 Edge Case Workflows

```bash
@test "integration: destination already exists" {
    # Should prompt or error appropriately
}

@test "integration: source doesn't exist" {
    # Should error clearly
}

@test "integration: no matching sessions" {
    # Should complete gracefully with warning
}

@test "integration: active Claude processes" {
    # Should warn or block depending on FORCE flag
}

@test "integration: concurrent move operations" {
    # Locking or error handling for race conditions
}
```

---

## 7. Regression Tests

### 7.1 Known Issues Prevention

```bash
@test "regression: transformer double-dash for dots (b7dad8d)" {
    # Critical fix: dots should become single dash, not double
    result=$(_suggest_project_dir_for "/Users/tryk/.config")
    [[ "$result" == *"--config"* ]] && fail "Dots should be single dash"
    [[ "$result" == *"-config"* ]] || fail "Dot not converted"
}

@test "regression: agent-mode no prompts (647ca31)" {
    # Agent mode should never prompt for input
}

@test "regression: XDG compliance (52df259)" {
    # All state files in XDG_STATE_HOME, not HOME
}

@test "regression: backup tar with leading dash" {
    # tar must use -- separator for project names with leading dash
}
```

---

## 8. Implementation Guide

### 8.1 Test Framework: Bats (Bash Automated Testing System)

**Why Bats:**
- Native bash execution (no language translation)
- Simple syntax for bash-heavy codebases
- TAP-compliant output for CI/CD integration
- Wide adoption in DevOps/infrastructure projects

**Installation:**
```bash
# macOS
brew install bats-core

# Linux
git clone https://github.com/bats-core/bats-core.git
cd bats-core
sudo ./install.sh /usr/local

# Verify
bats --version
```

### 8.2 Test Template

```bash
#!/usr/bin/env bats

# Load helpers
load test_helpers
load mock_claude_env

# Setup runs before each test
setup() {
    # Create isolated test environment
    TEST_DIR="$(mktemp -d)"
    export CLAUDE_DIR="$TEST_DIR/.claude"
    mkdir -p "$CLAUDE_DIR/projects"

    # Source the script under test
    source "$BATS_TEST_DIRNAME/../claude-manager.sh"
}

# Teardown runs after each test
teardown() {
    rm -rf "$TEST_DIR"
}

# Test case format
@test "descriptive test name" {
    # Arrange
    local input="/Users/tryk/project"

    # Act
    result=$(_suggest_project_dir_for "$input")

    # Assert
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project" ]
}

# Test with expected failure
@test "function should error on empty input" {
    run _suggest_project_dir_for ""
    [ "$status" -ne 0 ]
}

# Test with output validation
@test "function should log warning" {
    run _some_function_with_warning
    [ "$status" -eq 0 ]
    [[ "$output" == *"WARNING"* ]]
}
```

### 8.3 Helper Library: `tests/helpers/test_helpers.bash`

```bash
# Common test utilities

# Create a mock Claude project with sessions
create_mock_project() {
    local project_name="$1"
    local cwd_path="$2"
    local session_count="${3:-1}"

    local project_dir="$CLAUDE_DIR/projects/$project_name"
    mkdir -p "$project_dir"

    for i in $(seq 1 $session_count); do
        cat > "$project_dir/session-$i.jsonl" <<EOF
{"cwd":"$cwd_path","thread_id":"test-$i"}
{"role":"user","content":"Test message $i"}
EOF
    done
}

# Count cwd occurrences in a file
count_cwd_in_file() {
    local file="$1"
    local path="$2"
    grep -c "\"cwd\":\"$path\"" "$file" || echo 0
}

# Assert file contains path
assert_file_contains_path() {
    local file="$1"
    local path="$2"
    grep -q "\"cwd\":\"$path\"" "$file" || \
        fail "File $file does not contain path: $path"
}

# Assert file does NOT contain path
assert_file_not_contains_path() {
    local file="$1"
    local path="$2"
    ! grep -q "\"cwd\":\"$path\"" "$file" || \
        fail "File $file unexpectedly contains path: $path"
}

# Verify backup file exists and is valid
assert_backup_valid() {
    local backup_path="$1"
    [ -f "$backup_path" ] || fail "Backup not found: $backup_path"
    tar -tzf "$backup_path" >/dev/null 2>&1 || fail "Backup tar corrupted"
}
```

### 8.4 Running Tests

```bash
# Run all tests
bats tests/unit/*.bats
bats tests/integration/*.bats
bats tests/regression/*.bats

# Run specific test file
bats tests/unit/test_path_transformer.bats

# Run with verbose output
bats -T tests/unit/test_path_transformer.bats

# Run tests in parallel (faster)
bats -j 4 tests/

# Generate TAP output for CI
bats tests/ --formatter tap > test_results.tap

# Generate JUnit XML for CI
bats tests/ --formatter junit > test_results.xml
```

---

## 9. CI/CD Integration

### 9.1 GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Bats
        run: |
          git clone https://github.com/bats-core/bats-core.git
          cd bats-core
          sudo ./install.sh /usr/local

      - name: Run Unit Tests
        run: bats tests/unit/

      - name: Run Integration Tests
        run: bats tests/integration/

      - name: Run Regression Tests
        run: bats tests/regression/

      - name: Check Critical Coverage
        run: |
          # Verify path transformer has tests
          grep -q "_suggest_project_dir_for" tests/unit/test_path_transformer.bats

      - name: Upload Test Results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: test_results.tap
```

### 9.2 Pre-commit Hook

```bash
# .git/hooks/pre-commit
#!/bin/bash
set -e

echo "Running test suite before commit..."

# Run critical tests only (fast)
bats tests/unit/test_path_transformer.bats
bats tests/regression/test_transformer_regression.bats

echo "✅ All critical tests passed"
```

### 9.3 Taskfile Integration

```yaml
# Add to Taskfile.yml

tasks:
  test:
    desc: "Run all test suites"
    cmds:
      - echo "Running claude-manager test suite..."
      - bats tests/unit/
      - bats tests/integration/
      - bats tests/regression/
      - echo "✅ All tests passed"

  test:unit:
    desc: "Run unit tests only"
    cmds:
      - bats tests/unit/

  test:integration:
    desc: "Run integration tests only"
    cmds:
      - bats tests/integration/

  test:regression:
    desc: "Run regression tests only"
    cmds:
      - bats tests/regression/

  test:critical:
    desc: "Run critical tests only (fast)"
    cmds:
      - bats tests/unit/test_path_transformer.bats
      - bats tests/regression/test_transformer_regression.bats

  test:coverage:
    desc: "Generate test coverage report"
    cmds:
      - echo "Analyzing test coverage..."
      - |
        total_tests=$(bats tests/ --count)
        echo "Total test cases: $total_tests"
        echo "Coverage target: 100% on _suggest_project_dir_for"
        echo "Coverage target: 95%+ on _migrate_project"
        echo "Coverage target: 90%+ on backup/undo"

  test:watch:
    desc: "Watch for changes and re-run tests"
    cmds:
      - |
        while true; do
          inotifywait -e modify claude-manager.sh
          clear
          task test:critical
        done
```

---

## 10. Test Metrics & Coverage

### 10.1 Coverage Targets

| Component | Target Coverage | Rationale |
|-----------|----------------|-----------|
| `_suggest_project_dir_for()` | **100%** | Critical path transformer - cannot break |
| `_migrate_project()` | **95%+** | Data integrity critical |
| `_create_project_backup()` | **90%+** | Backup reliability essential |
| `_undo_last_operation()` | **90%+** | Rollback must work |
| `_parse_agent_flags()` | **85%+** | Flag handling needs coverage |
| `_sync_project_dir_name_with_path()` | **90%+** | Uses critical transformer |
| Helper functions | **70%+** | Good to have, not critical |

### 10.2 Success Metrics

**Per-Commit:**
- All critical tests pass (exit code 0)
- No new failures introduced
- Transformer regression tests pass

**Per-Release:**
- All test suites pass (unit, integration, regression, e2e)
- Coverage targets met
- Performance benchmarks within acceptable range
- Zero known critical bugs

**Long-term:**
- Test execution time < 5 minutes (full suite)
- Critical tests < 30 seconds
- Flaky test rate < 1%
- Test maintenance burden low

### 10.3 Coverage Analysis

```bash
#!/bin/bash
# tests/analyze_coverage.sh

echo "=== Test Coverage Analysis ==="
echo ""

# Count test cases per function
echo "Test cases by function:"
for func in _suggest_project_dir_for _migrate_project _create_project_backup _undo_last_operation; do
    count=$(grep -r "@test.*$func" tests/ | wc -l)
    echo "  $func: $count tests"
done

echo ""
echo "=== Critical Coverage Check ==="

# Verify path transformer has at least 20 tests
transformer_tests=$(grep -r "@test.*_suggest_project_dir_for" tests/ | wc -l)
if [ "$transformer_tests" -ge 20 ]; then
    echo "✅ Path transformer: $transformer_tests tests (target: 20+)"
else
    echo "❌ Path transformer: $transformer_tests tests (target: 20+)"
    exit 1
fi

# Verify regression tests exist for known bugs
regression_count=$(find tests/regression -name "*.bats" | wc -l)
if [ "$regression_count" -ge 1 ]; then
    echo "✅ Regression tests: $regression_count files"
else
    echo "⚠️  Regression tests: $regression_count files"
fi
```

---

## 11. Test Data & Fixtures

### 11.1 Fixture Directory Structure

```
tests/fixtures/
├── sample_sessions/
│   ├── simple_session.jsonl           # Basic conversation
│   ├── large_session.jsonl            # 10MB+ file
│   ├── malformed_session.jsonl        # Invalid JSON
│   └── whitespace_heavy.jsonl         # Various JSON formatting
├── project_structures/
│   ├── single_project/                # One project, few sessions
│   ├── multi_project/                 # Multiple projects
│   └── empty_project/                 # No sessions
└── edge_case_paths/
    ├── paths_with_dots.txt            # .config, .local, etc.
    ├── paths_with_hyphens.txt         # my-project, sub-dir
    ├── very_long_paths.txt            # 200+ char paths
    └── special_chars.txt              # Spaces, unicode, etc.
```

### 11.2 Sample Session Generator

```bash
#!/bin/bash
# tests/fixtures/generate_test_session.sh

generate_test_session() {
    local cwd_path="$1"
    local message_count="${2:-10}"
    local output_file="$3"

    for i in $(seq 1 $message_count); do
        # Human message
        cat >> "$output_file" <<EOF
{"timestamp":"2025-01-15T10:$(printf "%02d" $i):00.000Z","type":"human","text":"Test message $i","cwd":"$cwd_path"}
EOF
        # Assistant message
        cat >> "$output_file" <<EOF
{"timestamp":"2025-01-15T10:$(printf "%02d" $i):01.000Z","type":"assistant","text":"Test response $i","cwd":"$cwd_path"}
EOF
    done
}

# Generate various test sessions
mkdir -p sample_sessions

generate_test_session "/Users/tryk/project" 10 "sample_sessions/simple_session.jsonl"
generate_test_session "/Users/tryk/.config/nabi" 100 "sample_sessions/large_session.jsonl"
generate_test_session "/Users/tryk/my-project" 5 "sample_sessions/hyphenated_path.jsonl"
```

---

## 12. Debugging Failed Tests

### 12.1 Debug Techniques

```bash
# Run single test with verbose output
bats -T tests/unit/test_path_transformer.bats

# Run test with debug output preserved
CLAUDE_DEBUG=1 bats tests/unit/test_path_transformer.bats

# Inspect test environment after failure
@test "debug test environment" {
    skip "Debug only"
    # Set breakpoint
    echo "TEST_DIR: $TEST_DIR"
    echo "CLAUDE_DIR: $CLAUDE_DIR"
    ls -la "$CLAUDE_DIR/projects"
    # Manually exit to inspect
    return 1
}

# Capture function output for inspection
@test "debug function output" {
    run _suggest_project_dir_for "/Users/tryk/project"
    echo "Status: $status"
    echo "Output: $output"
    [ "$status" -eq 0 ]
}
```

### 12.2 Common Failure Patterns

| Symptom | Cause | Fix |
|---------|-------|-----|
| Test passes locally, fails CI | Environment differences (HOME, TMPDIR) | Use relative paths, mock environment |
| Intermittent failures | Race conditions, timing issues | Add explicit waits, ensure cleanup |
| "Command not found" | Script not sourced correctly | Check `source` statement in setup() |
| Path mismatch | Hardcoded paths | Use $TEST_DIR, $CLAUDE_DIR variables |
| Backup tar fails | Project names with leading dash | Use `tar -czf file -C dir -- name` |

---

## 13. Performance Testing

### 13.1 Performance Benchmarks

```bash
@test "perf: transformer handles 1000 paths < 1s" {
    start=$(date +%s)
    for i in $(seq 1 1000); do
        _suggest_project_dir_for "/Users/tryk/project-$i" >/dev/null
    done
    end=$(date +%s)
    duration=$((end - start))
    [ $duration -lt 1 ]
}

@test "perf: migration of 10MB session < 5s" {
    # Create 10MB session file
    # Run migration
    # Assert completion time < 5 seconds
}

@test "perf: backup creation < 2s for 100MB project" {
    # Create large project directory
    # Run backup
    # Assert time < 2 seconds
}
```

### 13.2 Stress Testing

```bash
@test "stress: 100 concurrent migrations" {
    # Fork 100 migration processes
    # Verify all complete successfully
    # Check for race conditions
}

@test "stress: very large session file (100MB)" {
    # Generate 100MB session
    # Run migration
    # Verify success and reasonable time
}

@test "stress: 1000 projects in CLAUDE_DIR" {
    # Create 1000 mock projects
    # Run project search
    # Verify performance acceptable
}
```

---

## 14. Test Maintenance

### 14.1 Test Review Checklist

- [ ] Test names are descriptive and follow convention
- [ ] Each test has clear Arrange-Act-Assert structure
- [ ] Tests are independent (no shared state)
- [ ] Cleanup is performed in teardown()
- [ ] Tests use fixtures, not hardcoded data
- [ ] Assertions have meaningful failure messages
- [ ] Tests are deterministic (no randomness)
- [ ] Performance is acceptable (< 1s per test)

### 14.2 Adding New Tests

When adding a new feature or fixing a bug:

1. **Write regression test first** (if fixing bug)
   - Reproduce the bug with a failing test
   - Fix the bug
   - Verify test passes

2. **Write unit tests** for new functions
   - Happy path
   - Edge cases
   - Error conditions

3. **Write integration tests** for new workflows
   - End-to-end scenarios
   - Component interaction

4. **Update test matrix** if transformer logic changes
   - Add new path variations
   - Verify coverage maintained

### 14.3 Test Code Quality

```bash
# Run shellcheck on test files
shellcheck tests/**/*.bats tests/helpers/*.bash

# Check for common anti-patterns
grep -r "sleep" tests/  # Avoid arbitrary sleeps
grep -r "TODO" tests/   # No unfinished tests
grep -r "skip" tests/   # Minimize skipped tests

# Verify test independence
bats tests/ --jobs 1  # Sequential
bats tests/ --jobs 4  # Parallel - should give same results
```

---

## 15. Documentation & Knowledge Transfer

### 15.1 Test Documentation Standards

Each test file should have:

```bash
#!/usr/bin/env bats

# test_path_transformer.bats
#
# PURPOSE:
#   Comprehensive tests for _suggest_project_dir_for() function.
#   This function is CRITICAL - it transforms working directory paths
#   to Claude's project naming convention. If it breaks, /resume breaks.
#
# COVERAGE TARGET: 100%
#
# TRANSFORMATION RULES:
#   1. Remove leading /
#   2. Convert . → -
#   3. Convert / → -
#   4. Prefix with -
#
# LAST UPDATED: 2025-10-27
# RELATED COMMIT: b7dad8d (transformer fix)
```

### 15.2 Onboarding New Contributors

**Test-First Development:**
```bash
# 1. Clone repo
git clone https://github.com/user/claude-manager.git
cd claude-manager

# 2. Install test framework
brew install bats-core  # or platform equivalent

# 3. Run existing tests to verify setup
task test:critical

# 4. Run full suite
task test

# 5. Before making changes, run tests
task test

# 6. After making changes, run tests again
task test

# 7. Add new tests for your changes
vim tests/unit/test_new_feature.bats

# 8. Verify all tests pass
task test
```

---

## 16. Appendix: Complete Test File Examples

### 16.1 Example: `tests/unit/test_path_transformer.bats`

```bash
#!/usr/bin/env bats

# test_path_transformer.bats - CRITICAL path transformer tests
# Target Coverage: 100%

load ../helpers/test_helpers

setup() {
    TEST_DIR="$(mktemp -d)"
    export CLAUDE_DIR="$TEST_DIR/.claude"
    export HOME="$TEST_DIR/home"
    mkdir -p "$CLAUDE_DIR/projects"
    mkdir -p "$HOME"
    source "$BATS_TEST_DIRNAME/../../claude-manager.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

# ===== SIMPLE PATHS =====

@test "transformer: basic absolute path" {
    result=$(_suggest_project_dir_for "/Users/tryk/dev/crush")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-dev-crush" ]
}

@test "transformer: home tilde expansion" {
    result=$(_suggest_project_dir_for "~/dev/crush")
    expected="$CLAUDE_DIR/projects/-$(echo "$HOME" | sed 's|^/||' | sed 's|/|-|g')-dev-crush"
    [ "$result" = "$expected" ]
}

# ===== PATHS WITH DOTS =====

@test "transformer: leading dot directory" {
    result=$(_suggest_project_dir_for "/Users/tryk/.config/nabi")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk--config-nabi" ]
}

@test "transformer: multiple dots in path" {
    result=$(_suggest_project_dir_for "/Users/tryk/Library.Data/stuff")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-Library-Data-stuff" ]
}

# ===== PATHS WITH HYPHENS =====

@test "transformer: single hyphen in path" {
    result=$(_suggest_project_dir_for "/Users/tryk/my-project")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-my-project" ]
}

@test "transformer: multiple hyphens in path" {
    result=$(_suggest_project_dir_for "/Users/tryk/my-awesome-project")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-my-awesome-project" ]
}

# ===== REAL-WORLD COMPLEX PATHS =====

@test "transformer: deep nested project path" {
    result=$(_suggest_project_dir_for "/Users/tryk/nabia/tui/production/riff-dag-tui")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-nabia-tui-production-riff-dag-tui" ]
}

# ===== EDGE CASES =====

@test "transformer: trailing slash normalization" {
    result=$(_suggest_project_dir_for "/Users/tryk/project/")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project" ]
}

@test "transformer: root directory" {
    result=$(_suggest_project_dir_for "/")
    [ "$result" = "$CLAUDE_DIR/projects/-" ]
}

# ===== IDEMPOTENCY =====

@test "transformer: same input produces same output" {
    result1=$(_suggest_project_dir_for "/Users/tryk/project")
    result2=$(_suggest_project_dir_for "/Users/tryk/project")
    [ "$result1" = "$result2" ]
}

@test "transformer: different inputs produce different outputs" {
    result1=$(_suggest_project_dir_for "/Users/tryk/project1")
    result2=$(_suggest_project_dir_for "/Users/tryk/project2")
    [ "$result1" != "$result2" ]
}

# Continue with remaining 10+ test cases from Test Matrix...
```

### 16.2 Example: `tests/integration/test_migration_workflow.bats`

```bash
#!/usr/bin/env bats

# test_migration_workflow.bats - Full migration integration tests

load ../helpers/test_helpers
load ../helpers/mock_claude_env

setup() {
    setup_mock_claude_env
}

teardown() {
    cleanup_mock_claude_env
}

@test "integration: full migrate command end-to-end" {
    # Arrange
    old_path="$TEST_DIR/old-project"
    new_path="$TEST_DIR/new-project"
    mkdir -p "$old_path"

    project_dir=$(_suggest_project_dir_for "$old_path")
    create_mock_project "$(basename "$project_dir")" "$old_path" 3

    # Act
    export CLAUDE_INTERACTIVE=false
    export CLAUDE_DRY_RUN=false
    run claude_manager migrate "$old_path" "$new_path" "$project_dir"

    # Assert
    [ "$status" -eq 0 ]
    assert_file_contains_path "$project_dir/session-1.jsonl" "$new_path"
    assert_file_not_contains_path "$project_dir/session-1.jsonl" "$old_path"

    # Verify backup created
    [ -f "$project_dir/session-1.jsonl.bak" ]
}

@test "integration: migrate with auto-detect project" {
    # Test auto-detection when project_dir not specified
}

@test "integration: migrate with sync project dir name" {
    # Test that project directory is renamed to match new path
}
```

---

## 17. Summary & Next Steps

### 17.1 Implementation Priority

**Phase 1: Critical Protection (Week 1)**
1. Implement `test_path_transformer.bats` with 20+ test cases
2. Implement `test_transformer_regression.bats` for known bugs
3. Add pre-commit hook to run critical tests
4. Integrate with Taskfile (`task test:critical`)

**Phase 2: Core Coverage (Week 2)**
5. Implement `test_migration_logic.bats`
6. Implement `test_backup_creation.bats`
7. Implement `test_undo_logic.bats`
8. Implement `test_flag_parsing.bats`

**Phase 3: Integration (Week 3)**
9. Implement `test_migration_workflow.bats`
10. Implement `test_move_and_rename.bats`
11. Add GitHub Actions CI workflow

**Phase 4: Polish (Week 4)**
12. Implement E2E tests
13. Add performance benchmarks
14. Generate coverage reports
15. Documentation and onboarding materials

### 17.2 Success Indicators

✅ **Immediate Success:**
- Path transformer has 20+ test cases
- All critical tests pass on every commit
- Pre-commit hook prevents regressions

✅ **Short-term Success (1 month):**
- 90%+ test coverage on critical functions
- CI/CD pipeline running all tests
- Zero known critical bugs in production

✅ **Long-term Success (3 months):**
- Full test suite (150+ tests) in place
- Test execution < 5 minutes
- Developer confidence in making changes
- Zero user-reported data loss incidents

---

**END OF DOCUMENT**

**Document Metadata:**
- **Author:** Senior Test Automation Architect
- **Version:** 1.0
- **Date:** 2025-10-27
- **Review Status:** Ready for Implementation
- **Critical Priority:** Path Transformer Protection
- **Target Coverage:** 100% on `_suggest_project_dir_for()`, 95%+ on migration logic
