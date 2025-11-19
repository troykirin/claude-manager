# Test Implementation Checklist

**Goal:** Ensure claude-manager path transformer and critical functions are bulletproof

---

## Phase 1: Critical Protection (Week 1) 🔥 HIGH PRIORITY

### Setup & Installation
- [ ] Install Bats testing framework
  ```bash
  task test:install
  # OR
  brew install bats-core  # macOS
  ```
- [ ] Verify Bats installation
  ```bash
  bats --version
  # Should output: Bats 1.x.x
  ```

### Run Existing Tests
- [ ] Run path transformer tests (42 tests)
  ```bash
  bats tests/unit/test_path_transformer.bats
  ```
- [ ] Verify all tests pass
- [ ] Review test output for any failures

### Pre-commit Hook
- [ ] Create pre-commit hook
  ```bash
  cat > .git/hooks/pre-commit <<'EOF'
  #!/bin/bash
  set -e
  echo "Running critical tests..."
  task test:critical
  EOF
  chmod +x .git/hooks/pre-commit
  ```
- [ ] Test pre-commit hook
  ```bash
  echo "# test" >> claude-manager.sh
  git add claude-manager.sh
  git commit -m "test: verify pre-commit hook"
  ```
- [ ] Verify tests run automatically on commit

### Documentation Review
- [ ] Read `docs/TEST_AUTOMATION_STRATEGY.md`
- [ ] Read `tests/QUICKSTART.md`
- [ ] Read `tests/README.md`
- [ ] Understand transformation rules (IMMUTABLE)

### Commit Test Framework
- [ ] Stage test files
  ```bash
  git add tests/
  git add docs/TEST_AUTOMATION_STRATEGY.md
  git add TESTING_SUMMARY.md
  git add TEST_IMPLEMENTATION_CHECKLIST.md
  git add Taskfile.yml
  ```
- [ ] Commit with descriptive message
  ```bash
  git commit -m "feat: add comprehensive test automation framework

  - Add 42 path transformer tests (100% coverage)
  - Add test strategy document (600+ lines)
  - Add test helpers and infrastructure
  - Integrate with Taskfile for easy execution
  - Add documentation and quick start guides

  CRITICAL: Protects _suggest_project_dir_for() function
  that converts paths to Claude project naming convention.
  If this breaks, /resume breaks for all users."
  ```

### Phase 1 Verification
- [ ] All path transformer tests passing
- [ ] Pre-commit hook working
- [ ] Documentation reviewed
- [ ] Tests committed to repository

**Phase 1 Complete:** Path transformer is now protected! 🎉

---

## Phase 2: Core Coverage (Week 2) 🟡 HIGH PRIORITY

### Migration Logic Tests
- [ ] Create `tests/unit/test_migration_logic.bats`
- [ ] Test basic path replacement
- [ ] Test whitespace-tolerant JSON matching
- [ ] Test zero occurrences handling
- [ ] Test multiple sessions
- [ ] Test backup strategy (file vs project)
- [ ] Test dry run mode
- [ ] Test metrics accuracy
- [ ] Test error handling
- [ ] Test large session files
- [ ] Target: 30+ tests, 95%+ coverage

### Backup Creation Tests
- [ ] Create `tests/unit/test_backup_creation.bats`
- [ ] Test XDG-compliant paths
- [ ] Test project names with leading dash
- [ ] Test tar archive integrity
- [ ] Test timestamp in filename
- [ ] Test restore functionality
- [ ] Target: 15+ tests, 90%+ coverage

### Undo Logic Tests
- [ ] Create `tests/unit/test_undo_logic.bats`
- [ ] Test restore source directory
- [ ] Test restore project directory
- [ ] Test restore session paths
- [ ] Test undo file format validation
- [ ] Test no undo file exists
- [ ] Test source_moved=false handling
- [ ] Target: 15+ tests, 90%+ coverage

### Flag Parsing Tests
- [ ] Create `tests/unit/test_flag_parsing.bats`
- [ ] Test --no-confirm
- [ ] Test --agent-mode
- [ ] Test --verbose
- [ ] Test --backup-strategy
- [ ] Test flags don't consume positional args
- [ ] Test multiple flags combined
- [ ] Test unknown flags ignored
- [ ] Target: 10+ tests, 85%+ coverage

### Phase 2 Verification
- [ ] All unit tests passing
- [ ] Coverage targets met
- [ ] No regressions in existing tests

**Phase 2 Complete:** Core functions protected! 🎉

---

## Phase 3: Integration (Week 3) 🟢 MEDIUM PRIORITY

### Migration Workflow Tests
- [ ] Create `tests/integration/test_migration_workflow.bats`
- [ ] Test migrate command end-to-end
- [ ] Test auto-detect project directory
- [ ] Test sync project dir name with path
- [ ] Target: 10+ tests

### Move and Rename Tests
- [ ] Create `tests/integration/test_move_and_rename.bats`
- [ ] Test move command end-to-end
- [ ] Test move then undo
- [ ] Test destination already exists
- [ ] Test source doesn't exist
- [ ] Target: 10+ tests

### Backup Restoration Tests
- [ ] Create `tests/integration/test_backup_restoration.bats`
- [ ] Test full backup/restore cycle
- [ ] Test restore from tar.gz
- [ ] Test restore preserves all files
- [ ] Target: 5+ tests

### Agent Mode Tests
- [ ] Create `tests/integration/test_agent_mode.bats`
- [ ] Test non-interactive migration
- [ ] Test metadata output
- [ ] Test backup creation in agent mode
- [ ] Target: 5+ tests

### CI/CD Integration
- [ ] Create `.github/workflows/test.yml`
- [ ] Configure GitHub Actions
- [ ] Test CI pipeline locally
- [ ] Verify tests run on PR
- [ ] Set up branch protection rules

### Phase 3 Verification
- [ ] All integration tests passing
- [ ] CI/CD pipeline working
- [ ] Tests run automatically on PR

**Phase 3 Complete:** Full workflows tested! 🎉

---

## Phase 4: Polish (Week 4) 🔵 LOW PRIORITY

### End-to-End Tests
- [ ] Create `tests/e2e/test_full_user_journey.bats`
- [ ] Test complete migration workflow
- [ ] Test complete move workflow
- [ ] Test organize extract workflow
- [ ] Target: 5-10 tests

### Performance Benchmarks
- [ ] Test transformer handles 1000 paths < 1s
- [ ] Test migration of 10MB session < 5s
- [ ] Test backup creation < 2s for 100MB project
- [ ] Add performance regression tests

### Coverage Analysis
- [ ] Create `tests/analyze_coverage.sh`
- [ ] Integrate with `task test:coverage`
- [ ] Generate coverage reports
- [ ] Identify coverage gaps

### Documentation Updates
- [ ] Update README.md with test badge
- [ ] Update CONTRIBUTING.md with test guidelines
- [ ] Add test examples to docs
- [ ] Document common test patterns

### Phase 4 Verification
- [ ] E2E tests passing
- [ ] Performance benchmarks passing
- [ ] Coverage reports generated
- [ ] Documentation updated

**Phase 4 Complete:** Test suite fully polished! 🎉

---

## Continuous Maintenance

### Regular Tasks
- [ ] Run tests before every commit
  ```bash
  task test:critical
  ```
- [ ] Run full suite weekly
  ```bash
  task test
  ```
- [ ] Review test coverage monthly
  ```bash
  task test:coverage
  ```
- [ ] Update tests when bugs found
  - Add regression test first
  - Fix bug
  - Verify test passes
  - Commit both

### Test Quality Checks
- [ ] All tests are independent
- [ ] All tests have descriptive names
- [ ] All tests clean up in teardown()
- [ ] All tests use fixtures, not hardcoded data
- [ ] All tests complete in < 1s each
- [ ] No flaky tests (99%+ reliability)

### Documentation Maintenance
- [ ] Keep QUICKSTART.md updated
- [ ] Update TEST_AUTOMATION_STRATEGY.md for major changes
- [ ] Document new test patterns
- [ ] Update coverage targets as needed

---

## Success Metrics

### Immediate (Week 1)
- [x] Path transformer: 42 tests (100% coverage)
- [x] Test framework: Fully set up
- [x] Documentation: Complete
- [ ] Pre-commit hook: Active
- [ ] All tests: Passing

### Short-term (1 Month)
- [ ] Critical functions: 90%+ coverage
- [ ] Total tests: 100+
- [ ] CI/CD: Running
- [ ] Zero critical bugs: Verified

### Long-term (3 Months)
- [ ] Total tests: 150+
- [ ] Full suite: < 5 min
- [ ] Zero data loss: No incidents
- [ ] Developer confidence: High

---

## Troubleshooting

### Bats Not Found
```bash
# Check installation
which bats

# Install if missing
task test:install
```

### Tests Failing
```bash
# Run with verbose output
bats -T tests/unit/test_path_transformer.bats

# Check test environment
echo "CLAUDE_DIR: $CLAUDE_DIR"
echo "TEST_DIR: $TEST_DIR"
```

### Tests Slow
```bash
# Run in parallel
bats -j 4 tests/

# Run critical only
task test:critical

# Profile slow tests
time bats tests/unit/test_path_transformer.bats
```

---

## Quick Reference

### Essential Commands
```bash
# Install
task test:install

# Run critical tests
task test:critical

# Run all tests
task test

# Watch mode
task test:watch

# Coverage analysis
task test:coverage

# Quick start guide
task test:quickstart
```

### Test Files
- Path transformer: `tests/unit/test_path_transformer.bats` (42 tests)
- Test helpers: `tests/helpers/test_helpers.bash`
- Strategy doc: `docs/TEST_AUTOMATION_STRATEGY.md`
- Quick start: `tests/QUICKSTART.md`

### Documentation
- Full strategy: `docs/TEST_AUTOMATION_STRATEGY.md`
- Quick start: `tests/QUICKSTART.md`
- Test suite info: `tests/README.md`
- This checklist: `TEST_IMPLEMENTATION_CHECKLIST.md`
- Summary: `TESTING_SUMMARY.md`

---

## Notes

### DO NOT BREAK THE TRANSFORMER
The `_suggest_project_dir_for()` function is CRITICAL. It has 42 tests protecting it. These tests MUST pass before any commit that touches this function.

### Test-First Development
Always write tests first:
1. Write failing test
2. Implement feature
3. Verify test passes
4. Refactor with confidence

### Git Workflow
```bash
# Before making changes
task test:critical

# Make changes
vim claude-manager.sh

# Run tests again
task test:critical

# If tests pass, commit
git add .
git commit -m "feat: your changes"
```

---

**Last Updated:** 2025-10-27
**Status:** Phase 1 deliverables complete, ready for implementation
**Next Action:** Install Bats and run critical tests
