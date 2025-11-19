# Phase 1: Staleness Analysis Report
Generated: $(date)

## Critical Findings

### Nested Directory Structure
- Root: /Users/tryk/nabia/tools/claude-manager/
- Level 1: claude-session-tui/ (AUTHORITATIVE)
- Level 2: claude-session-tui/claude-session-tui/ (STALE - TO BE REMOVED)

### File Staleness Evidence

#### models.rs
- Nested (STALE): 666 lines, 17,991 bytes, modified Nov 14 14:57
- Level 1 (CURRENT): 694 lines, 18,902 bytes, modified Nov 14 14:40
- Delta: 28 lines behind, 911 bytes smaller

#### parser.rs  
- Nested (STALE): 1,029 lines, 38,026 bytes, modified Oct 31 13:47
- Level 1 (CURRENT): 1,545 lines, 56,934 bytes, modified Nov 14 14:41
- Delta: 516 lines behind, 18,908 bytes smaller (SEVERELY OUTDATED)

#### Cargo.toml
- Nested: INCOMPLETE (missing license, repository, homepage, keywords, categories, deps: dirs, shellexpand, atty)
- Level 1: COMPLETE with full metadata

### Files to Remove (35 total in nested directory)
- claude-session-tui/claude-session-tui/src/ (all Rust source files)
- claude-session-tui/claude-session-tui/Cargo.toml (incomplete)
- claude-session-tui/claude-session-tui/Cargo.lock (if exists)

### Safety Measures
- Backup created: .backup-pre-flattening-20251119-040832.tar.gz (2.2MB)
- Diff generated: .analysis/src-diff.txt
- Git status preserved for rollback

## Recommended Action
Proceed with Phase 2: Remove stale nested directory entirely
Wed Nov 19 04:09:01 PST 2025
