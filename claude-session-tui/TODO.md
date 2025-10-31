# TODO

## High Priority

### Performance Optimization
- [ ] Fix federation test failure in error handling expectation
- [ ] Optimize session file search performance for large Claude installations (363+ files)
- [ ] Add session file indexing or caching mechanism for faster lookups
- [ ] Implement parallel processing for multi-file operations

### Security Enhancements
- [ ] Implement content sanitization for sensitive data in sessions
- [ ] Add dependency vulnerability scanning to CI pipeline (cargo-audit, npm audit)
- [ ] Enhance federation security with authentication layer
- [ ] Consider session data encryption for sensitive environments

## Medium Priority

### Documentation & Organization
- [ ] Complete /docs directory reorganization (in progress)
- [ ] Archive historical debugging documents
- [ ] Create developer onboarding checklist
- [ ] Document federation integration patterns
- [ ] Add architecture diagrams

### Code Quality
- [ ] Reduce bash script complexity (215 conditionals)
- [ ] Add comprehensive unit tests for bash components
- [ ] Increase documentation coverage to 15%+ across all components
- [ ] Standardize error handling across language boundaries

### User Experience
- [ ] Improve error messages in CLI for better UX
- [ ] Add progress indicators for long-running operations
- [ ] Implement better conflict resolution UI
- [ ] Add verbose mode for debugging

## Low Priority

### Testing & Benchmarking
- [ ] Add performance benchmarks to CI pipeline
- [ ] Create integration test suite covering CLI → TUI → Federation
- [ ] Add automated security testing
- [ ] Implement load testing for large session files

### Feature Enhancements
- [ ] Add session file compression option
- [ ] Implement session diff visualization
- [ ] Create web-based dashboard for session analytics
- [ ] Add export formats (JSON, CSV) for session data

### Technical Debt
- [ ] Remove V2 Shadow Architecture from Rust TUI (over-engineered)
- [ ] Consolidate Python utilities purpose and documentation
- [ ] Extract shared patterns into reusable libraries
- [ ] Implement structured logging across all components

## Completed (Recent)

### ✅ Fixed Issues
- [x] Arithmetic syntax error in session file update (commit ddb5403)
- [x] Path resolution robustness improvements (commit 736916c)
- [x] Tilde expansion in path resolution (commit 213dcd3)
- [x] Shell RC configuration installation (commit e283e31)
- [x] Context-sensitive session management (commit 1a25d8a)
- [x] Enhanced error handling and search capabilities (commit aeab546)
- [x] Prioritized project detection system (commit 372ef9a)

### ✅ Documentation
- [x] Comprehensive repository analysis completed
- [x] Security posture evaluation completed
- [x] Code quality metrics assessment completed
- [x] Maintainability review completed

## Notes

- Federation integration has 1 failing test that needs investigation
- Performance issues observed with 363+ session files
- Root directory cleanup recommended for better organization
- Consider implementing session file garbage collection for old/unused sessions