# Test Suite Quick Start Guide

## Prerequisites

### Install Bats Testing Framework

**macOS:**
```bash
brew install bats-core
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install bats
```

**From Source:**
```bash
git clone https://github.com/bats-core/bats-core.git
cd bats-core
sudo ./install.sh /usr/local
```

**Verify Installation:**
```bash
bats --version
# Expected: Bats 1.x.x
```

## Running Tests

### Run All Tests
```bash
# From project root
bats tests/

# With parallel execution (faster)
bats -j 4 tests/
```

### Run Critical Tests Only (Fast - under 30s)
```bash
bats tests/unit/test_path_transformer.bats
bats tests/regression/test_transformer_regression.bats
```

### Run Specific Test Suite
```bash
# Unit tests only
bats tests/unit/

# Integration tests only
bats tests/integration/

# Regression tests only
bats tests/regression/
```

### Run Single Test File
```bash
bats tests/unit/test_path_transformer.bats
```

### Run with Verbose Output
```bash
# Show test names and timing
bats -T tests/unit/test_path_transformer.bats

# Show all output (including echo statements)
bats --tap tests/unit/test_path_transformer.bats
```

## Using Taskfile (Recommended)

### Add to Taskfile.yml
```yaml
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

  test:critical:
    desc: "Run critical tests only (fast)"
    cmds:
      - bats tests/unit/test_path_transformer.bats
      - bats tests/regression/test_transformer_regression.bats

  test:watch:
    desc: "Watch for changes and re-run tests"
    cmds:
      - |
        echo "Watching for changes to claude-manager.sh..."
        while true; do
          if command -v inotifywait >/dev/null; then
            inotifywait -e modify claude-manager.sh
          else
            # Fallback: poll every 2 seconds
            sleep 2
          fi
          clear
          echo "Change detected, running tests..."
          task test:critical
        done
```

### Run via Task
```bash
# Run all tests
task test

# Run unit tests
task test:unit

# Run critical tests only
task test:critical

# Watch mode (re-run on changes)
task test:watch
```

## Test Development Workflow

### 1. Create New Test File
```bash
# Create new test file
touch tests/unit/test_new_feature.bats

# Add header
cat > tests/unit/test_new_feature.bats <<'EOF'
#!/usr/bin/env bats

# test_new_feature.bats - Tests for new feature

setup() {
    TEST_DIR="$(mktemp -d)"
    export CLAUDE_DIR="$TEST_DIR/.claude"
    mkdir -p "$CLAUDE_DIR/projects"
    source "$BATS_TEST_DIRNAME/../../claude-manager.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "new feature: basic functionality" {
    # Test implementation
}
EOF

# Make executable
chmod +x tests/unit/test_new_feature.bats
```

### 2. Write Test First (TDD)
```bash
@test "feature: should do X" {
    # Arrange
    input="test_input"

    # Act
    result=$(_my_function "$input")

    # Assert
    [ "$result" = "expected_output" ]
}
```

### 3. Run Test (Should Fail)
```bash
bats tests/unit/test_new_feature.bats
# Expected: FAIL (function not implemented yet)
```

### 4. Implement Feature
```bash
# Edit claude-manager.sh
vim claude-manager.sh

# Add function implementation
```

### 5. Run Test (Should Pass)
```bash
bats tests/unit/test_new_feature.bats
# Expected: PASS
```

### 6. Commit with Passing Tests
```bash
git add tests/unit/test_new_feature.bats
git add claude-manager.sh
git commit -m "feat: implement new feature with tests"
```

## Common Test Patterns

### Test with Expected Success
```bash
@test "function should succeed" {
    result=$(_my_function "input")
    [ "$result" = "expected" ]
}
```

### Test with Expected Failure
```bash
@test "function should fail on invalid input" {
    run _my_function ""
    [ "$status" -ne 0 ]
}
```

### Test with Output Validation
```bash
@test "function should output warning" {
    run _my_function "risky_input"
    [ "$status" -eq 0 ]
    [[ "$output" == *"WARNING"* ]]
}
```

### Test with File Operations
```bash
@test "function should create file" {
    _my_function "$TEST_DIR/output.txt"
    [ -f "$TEST_DIR/output.txt" ]
    grep -q "expected content" "$TEST_DIR/output.txt"
}
```

### Test with Mocking
```bash
@test "function should call helper" {
    # Mock helper function
    _helper_function() {
        echo "mocked"
    }
    export -f _helper_function

    result=$(_my_function)
    [[ "$result" == *"mocked"* ]]
}
```

## Debugging Failed Tests

### Run with Verbose Output
```bash
bats -T tests/unit/test_path_transformer.bats
```

### Add Debug Output to Tests
```bash
@test "debug test" {
    echo "TEST_DIR: $TEST_DIR" >&3
    echo "CLAUDE_DIR: $CLAUDE_DIR" >&3
    result=$(_my_function)
    echo "Result: $result" >&3
    [ -n "$result" ]
}
```

### Run Single Test
```bash
# Find test name
grep "@test" tests/unit/test_path_transformer.bats

# Run just that test
bats -f "specific test name" tests/unit/test_path_transformer.bats
```

### Inspect Test Environment
```bash
@test "inspect environment" {
    skip "Debug only"
    echo "Environment:"
    env | grep CLAUDE
    ls -la "$CLAUDE_DIR"
    return 1  # Fail to stop execution
}
```

## Pre-commit Hook Setup

### Install Pre-commit Hook
```bash
# Create hook
cat > .git/hooks/pre-commit <<'EOF'
#!/bin/bash
set -e

echo "Running critical tests before commit..."

# Run critical tests only (fast)
bats tests/unit/test_path_transformer.bats
bats tests/regression/test_transformer_regression.bats

echo "✅ All critical tests passed"
EOF

# Make executable
chmod +x .git/hooks/pre-commit
```

### Test Pre-commit Hook
```bash
# Make a change
echo "# test" >> claude-manager.sh

# Try to commit (should run tests)
git add claude-manager.sh
git commit -m "test: verify pre-commit hook"
```

## CI/CD Integration

### GitHub Actions
See `.github/workflows/test.yml` for full workflow.

### Quick CI Check Locally
```bash
# Simulate CI environment
export CI=true
export CLAUDE_DIR=/tmp/test-claude-dir
export CLAUDE_INTERACTIVE=false

# Run tests
bats tests/
```

## Troubleshooting

### Bats Not Found
```bash
# Check installation
which bats

# If not found, install
brew install bats-core  # macOS
```

### Tests Fail with Permission Errors
```bash
# Ensure test files are executable
chmod +x tests/**/*.bats

# Ensure script is sourced correctly
source "$BATS_TEST_DIRNAME/../../claude-manager.sh"
```

### Tests Pass Locally, Fail in CI
- Check environment variables (HOME, TMPDIR, etc.)
- Use $TEST_DIR instead of hardcoded paths
- Verify all dependencies are installed in CI

### Tests Are Slow
```bash
# Run in parallel
bats -j 4 tests/

# Run critical tests only
task test:critical

# Profile slow tests
time bats tests/unit/test_path_transformer.bats
```

## Best Practices

1. **Always run tests before committing**
2. **Write tests first (TDD)**
3. **Keep tests independent** (no shared state)
4. **Use descriptive test names**
5. **Test both success and failure cases**
6. **Use fixtures for test data**
7. **Clean up in teardown()**
8. **Mock external dependencies**
9. **Keep tests fast** (< 1s per test)
10. **Document complex test logic**

## Getting Help

- **Bats Documentation:** https://bats-core.readthedocs.io/
- **Test Strategy:** See `docs/TEST_AUTOMATION_STRATEGY.md`
- **Examples:** Look at existing tests in `tests/unit/`

## Quick Reference Card

```bash
# Run all tests
bats tests/

# Run critical tests (fast)
task test:critical

# Run with verbose output
bats -T tests/unit/test_path_transformer.bats

# Run single test
bats -f "test name" tests/unit/file.bats

# Run in parallel
bats -j 4 tests/

# Debug a test
CLAUDE_DEBUG=1 bats tests/unit/test.bats

# Generate TAP output
bats --tap tests/ > results.tap

# Watch for changes
task test:watch
```

---

**Ready to start testing!** 🚀

Run your first test:
```bash
bats tests/unit/test_path_transformer.bats
```
