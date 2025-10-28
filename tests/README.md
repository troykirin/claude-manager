# Claude Manager Test Suite

**Comprehensive test automation for claude-manager.sh**

## Quick Start

```bash
# Install test framework
task test:install

# Run critical tests (under 30s)
task test:critical

# Run all tests
task test

# Read the quick start guide
task test:quickstart
```

## Test Structure

```
tests/
├── unit/                              # Function-level tests (60% of tests)
│   └── test_path_transformer.bats    # 🔥 CRITICAL - 100% coverage required
├── integration/                       # Component interaction tests (30%)
├── regression/                        # Known bug prevention tests
├── e2e/                              # Full user workflow tests (10%)
├── helpers/                          # Shared test utilities
│   └── test_helpers.bash             # Common test functions
├── fixtures/                         # Test data and sample files
├── QUICKSTART.md                     # Quick start guide
└── README.md                         # This file
```

## Critical Function Protection

### Path Transformer (`_suggest_project_dir_for`)

**Status:** 🔥 CRITICAL - DO NOT BREAK

This function transforms working directory paths to Claude's project naming convention. If it breaks, `/resume` breaks for all users.

**Rules:**
1. Remove leading `/`
2. Convert `.` → `-`
3. Convert `/` → `-`
4. Prefix with `-`

**Example:**
```
/Users/tryk/nabia/tui/production/riff-dag-tui
         ↓
-Users-tryk-nabia-tui-production-riff-dag-tui
```

**Test Coverage:** 40+ test cases in `tests/unit/test_path_transformer.bats`

## Running Tests

### Via Task (Recommended)

```bash
# Run all tests
task test

# Run specific suite
task test:unit
task test:integration
task test:regression

# Run critical tests only (fast)
task test:critical

# Watch for changes
task test:watch

# Analyze coverage
task test:coverage
```

### Via Bats Directly

```bash
# Run all tests
bats tests/

# Run specific file
bats tests/unit/test_path_transformer.bats

# Run with verbose output
bats -T tests/unit/test_path_transformer.bats

# Run in parallel (faster)
bats -j 4 tests/

# Run specific test by name
bats -f "transformer: basic" tests/unit/test_path_transformer.bats
```

## Test Coverage Targets

| Component | Target | Status |
|-----------|--------|--------|
| `_suggest_project_dir_for()` | **100%** | 🟢 40+ tests |
| `_migrate_project()` | **95%+** | 🟡 In progress |
| `_create_project_backup()` | **90%+** | 🔴 TODO |
| `_undo_last_operation()` | **90%+** | 🔴 TODO |
| `_parse_agent_flags()` | **85%+** | 🔴 TODO |

## Writing Tests

### Test Template

```bash
#!/usr/bin/env bats

# Load helpers if needed
load ../helpers/test_helpers

setup() {
    TEST_DIR="$(mktemp -d)"
    export CLAUDE_DIR="$TEST_DIR/.claude"
    mkdir -p "$CLAUDE_DIR/projects"
    source "$BATS_TEST_DIRNAME/../../claude-manager.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "descriptive test name" {
    # Arrange
    input="test_value"

    # Act
    result=$(_function_under_test "$input")

    # Assert
    [ "$result" = "expected_output" ]
}
```

### Best Practices

1. **Descriptive names:** `@test "transformer: path with dots becomes dashes"`
2. **Independent tests:** No shared state between tests
3. **Clean setup/teardown:** Always cleanup in `teardown()`
4. **Use helpers:** Leverage `test_helpers.bash` for common operations
5. **Fast tests:** Keep each test under 1 second
6. **Test both paths:** Success AND failure cases
7. **Meaningful assertions:** Clear failure messages

## CI/CD Integration

### Pre-commit Hook

```bash
# Install pre-commit hook
cat > .git/hooks/pre-commit <<'EOF'
#!/bin/bash
set -e
echo "Running critical tests..."
task test:critical
echo "✅ Tests passed"
EOF

chmod +x .git/hooks/pre-commit
```

### GitHub Actions

See `.github/workflows/test.yml` for CI workflow configuration.

## Debugging Failed Tests

### Verbose Output

```bash
# Show test execution details
bats -T tests/unit/test_path_transformer.bats

# Show all output (including debug statements)
CLAUDE_DEBUG=1 bats tests/unit/test_path_transformer.bats
```

### Run Single Test

```bash
# Find test name
grep "@test" tests/unit/test_path_transformer.bats

# Run specific test
bats -f "exact test name" tests/unit/test_path_transformer.bats
```

### Add Debug Output

```bash
@test "debug test" {
    echo "TEST_DIR: $TEST_DIR" >&3
    echo "CLAUDE_DIR: $CLAUDE_DIR" >&3
    result=$(_my_function)
    echo "Result: $result" >&3
    [ -n "$result" ]
}
```

## Performance Testing

```bash
# Time test execution
time bats tests/unit/test_path_transformer.bats

# Run with parallel execution
bats -j 4 tests/

# Profile individual tests
bats -T tests/unit/test_path_transformer.bats
```

## Test Maintenance

### Adding New Tests

1. **Write test first** (TDD approach)
2. **Verify it fails** (before implementation)
3. **Implement feature**
4. **Verify it passes**
5. **Commit with tests**

### Updating Existing Tests

1. **Understand the change** (why is it needed?)
2. **Update test cases** (maintain coverage)
3. **Run full suite** (no regressions)
4. **Update documentation** (if behavior changed)

### Removing Tests

- **Never remove tests without review**
- **Document why** (in commit message)
- **Verify coverage maintained** (no gaps)

## Common Issues

### Bats Not Found

```bash
# Install bats
task test:install

# Or manually
brew install bats-core  # macOS
```

### Tests Pass Locally, Fail in CI

- Check environment variables (HOME, TMPDIR)
- Use `$TEST_DIR` instead of hardcoded paths
- Verify dependencies in CI environment

### Slow Test Execution

- Run tests in parallel: `bats -j 4 tests/`
- Run critical tests only: `task test:critical`
- Profile slow tests with `bats -T`

## Resources

- **Test Strategy:** `docs/TEST_AUTOMATION_STRATEGY.md`
- **Quick Start:** `tests/QUICKSTART.md`
- **Bats Documentation:** https://bats-core.readthedocs.io/
- **Examples:** `tests/unit/test_path_transformer.bats`

## Test Statistics

```bash
# Count test files
find tests -name "*.bats" | wc -l

# Count test cases
grep -r "@test" tests/ | wc -l

# Show test distribution
echo "Unit:        $(grep -r "@test" tests/unit/ | wc -l)"
echo "Integration: $(grep -r "@test" tests/integration/ | wc -l)"
echo "Regression:  $(grep -r "@test" tests/regression/ | wc -l)"
echo "E2E:         $(grep -r "@test" tests/e2e/ | wc -l)"
```

## Contributing

1. **All new features must have tests**
2. **All bug fixes must have regression tests**
3. **Critical functions require 100% coverage**
4. **Run `task test:critical` before every commit**
5. **Update documentation with behavior changes**

---

**Questions?** See `tests/QUICKSTART.md` or `docs/TEST_AUTOMATION_STRATEGY.md`
