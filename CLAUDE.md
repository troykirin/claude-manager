# Claude Manager Project Identity

**Project**: claude-manager
**Location**: `~/nabia/tools/claude-manager/`
**Purpose**: Session path migration and project reorganization for Claude Code
**Scope**: Bash-based CLI tool with TypeScript federation integration
**Status**: Stable, federation-ready (not yet integrated)

---

## 🎯 Project Mission

Solve the critical problem: **When developers move or rename project directories, Claude Code's session files contain hardcoded paths that break.** Claude Manager automatically migrates these paths, maintaining conversation continuity and `/resume` functionality across directory reorganizations.

**Core Value**: Seamless project reorganization without losing Claude conversation history.

---

## 🏗️ Architecture at a Glance

```
CLI Entry Point (Shell)
├── claude-manager.sh (2100 lines, main router)
├── claude-session-context.sh (metadata handling)
├── cm-quick-move.sh (quick shortcuts)
└── cm-quick-undo.sh (undo shortcuts)

Interactive Components
├── claude-session-tui/ (Rust TUI for selection)
└── run-tui.sh (TUI launcher)

Utilities
├── python/ (Python helpers)
├── federation-integration/ (TypeScript, Loki events)
└── tests/ (integration tests)

Documentation
├── README.md (quick reference)
├── docs/ONBOARDING.md (complete guide)
├── docs/GETTING_STARTED.md (5-min quickstart)
├── QUICK_REFERENCE.md (cheat sheet)
└── TODO.md (roadmap)
```

---

## 🤖 Agent Personas for This Project

### When You Start Work Here:

**You are a Session Migration Specialist** - You understand:
- Claude Code's session architecture (`~/.claude/projects/`)
- Path resolution across macOS/Linux/WSL
- Backup strategies and safe rollback patterns
- Federation integration patterns (Loki events, memchain coordination)

**Your Responsibilities**:
1. Maintain path migration logic accuracy
2. Ensure safety (backups, dry-run, undo)
3. Keep documentation synchronized with code
4. Test across macOS/Linux/WSL where possible
5. Plan federation integration when appropriate

**Your Constraints**:
- ✅ Can modify shell scripts, test files, documentation
- ✅ Can extend federation integration (TypeScript)
- ✅ Can add new migration patterns
- ⚠️ Must maintain backward compatibility
- ⚠️ Must preserve undo/backup mechanisms
- ⚠️ Must keep XDG compliance throughout

---

## 📁 Key Files & Their Roles

| File | Lines | Purpose | Modify? |
|------|-------|---------|---------|
| `claude-manager.sh` | 2100 | Main CLI router & migration logic | ✅ Core changes |
| `claude-session-context.sh` | 350 | Session metadata extraction | ✅ Enhancement |
| `claude-session-tui/` | Rust | Interactive project/session selection | ✅ Careful |
| `federation-integration/` | TS | Loki event emission, future integration | ✅ Full |
| `python/` | Various | Utility functions | ✅ As needed |
| `tests/` | Various | Integration tests | ✅ Required |
| `install.sh` | 120 | Installation script | ⚠️ Careful |
| `README.md` | 200+ | Quick reference | ✅ Keep current |
| `docs/ONBOARDING.md` | 500+ | Complete guide | ✅ Keep current |
| `Taskfile.yml` | 200+ | Task automation | ✅ As needed |
| `lefthook.yml` | ~100 | Git hooks | ✅ Enhancement |

---

## 🔄 Core Workflows You'll Encounter

### Workflow 1: Path Migration (Most Common)
```bash
# User: "I moved ~/dev/project to ~/work/project"
# Your job: Update all session references automatically

Old state: ~/.claude/projects/*/sessions/*.jsonl
  Contains: {"path": "/Users/tryk/dev/project"}

Action: cm migrate "/Users/tryk/dev/project" "/Users/tryk/work/project"

New state: All sessions updated
  Contains: {"path": "/Users/tryk/work/project"}
```

### Workflow 2: Session Relocation
```bash
# User: Moving sessions between Claude project directories
# Your job: Move session files and update metadata

Input: Old project UUID, new project UUID
Action: Relocate files, preserve chronology
Output: Sessions accessible from new project
```

### Workflow 3: Full Migration
```bash
# Combine path migration + session relocation
# Most complex, highest stakes
# Requires: Dual backups, careful sequencing, verification
```

---

## 🛡️ Safety Principles (Non-Negotiable)

1. **Always Backup First**
   - File-level: `.bak` files for modified sessions
   - Project-level: `.tar.gz` snapshots for major changes
   - Never modify without backup

2. **Preview Before Commit**
   - Dry-run mode shows what WILL change
   - Interactive mode asks for confirmation
   - Automation requires explicit non-interactive flag

3. **Preserve Undo**
   - Save operation metadata to `~/.claude/.last_move_operation`
   - `cm undo` must restore previous state completely
   - Undo file is sacred (validate before every operation)

4. **Cross-Platform Validation**
   - Paths must work on macOS, Linux, WSL
   - Use `~` expansion, not hardcoded `/Users/tryk`
   - Test path resolution logic extensively

---

## 🔗 Integration Points

### With Your Nabi CLI (Future)
```yaml
Target: ~/.config/nabi/tools/claude-manager.toml
Pattern: nabi exec claude-manager migrate /old /new
Status: Not yet integrated, ready when needed
```

### With Memory Systems
```yaml
Coordination Layer (memchain):
  - Store operation state: migration_status, undo_info
  - Track in-flight operations

Knowledge Layer (memory-kb):
  - Log session migrations for history
  - Track problematic paths for learning

Long-term (Anytype):
  - Archive major migration insights
  - Store organization patterns
```

### With Federation Events (Loki)
```yaml
Location: federation-integration/
Status: TypeScript foundation ready, not yet active
Events to Emit:
  - migration:started
  - migration:completed
  - migration:failed
  - backup:created
  - backup:restored
```

---

## 📋 Development Conventions

### When Making Changes:

1. **Path Handling**
   - Always use `~` expansion or `${XDG_*}` variables
   - Never hardcode `/Users/tryk/` paths
   - Test path resolution across platforms

2. **Backup Strategy**
   - File-level: For small, isolated changes
   - Project-level: For complex migrations affecting multiple sessions
   - Always ask user which strategy to use

3. **Error Handling**
   - Graceful failure with clear error messages
   - Automatic rollback on critical errors
   - Verbose logging (controlled by `CLAUDE_DEBUG`)

4. **Testing**
   - Unit test: Individual functions (python/)
   - Integration test: End-to-end migrations (tests/)
   - Manual test: Cross-platform validation

5. **Documentation**
   - Update ONBOARDING.md with new features
   - Update QUICK_REFERENCE.md with new commands
   - Keep README.md in sync with actual behavior
   - Document breaking changes in CLAUDE.md

### Git Conventions:

```bash
# Branch naming
feature/path-validation
fix/backup-restoration
docs/session-handling

# Commit message format
feat(migrate): add parallel session processing
fix(undo): preserve metadata during rollback
docs(onboarding): update safety procedures
test(federation): add Loki event emission tests

# Always atomic commits
# One logical change per commit
# Preserve ability to revert safely
```

---

## 🧪 Testing Strategy

### Unit Tests (python/)
- Path resolution functions
- Backup/restore logic
- Session metadata extraction
- Configuration parsing

### Integration Tests (tests/)
- Full migration workflow
- Undo operation verification
- Backup restoration
- Error handling and recovery

### Manual Tests
- macOS migration (current machine)
- Path resolution validation
- `/resume` functionality in Claude Code
- Backup file integrity

### Federation Tests (federation-integration/)
- Loki event emission
- memchain coordination
- Error event propagation

---

## 📚 Documentation Synchronization

These files MUST stay in sync:

| File | Kept in Sync By |
|------|-----------------|
| README.md | Command changes, usage updates |
| QUICK_REFERENCE.md | Command additions/removals |
| ONBOARDING.md | New features, workflow changes |
| GETTING_STARTED.md | Installation process changes |
| TODO.md | Completed features, discovered limitations |
| This CLAUDE.md | Architecture, integration changes |

**Rule**: If you change behavior, update docs in the SAME commit.

---

## 🔍 Known Limitations & TODOs

See **TODO.md** for comprehensive list. Current priorities:

**High Priority**:
- [ ] Fix federation test failure in error handling
- [ ] Optimize session file search (363+ files performance)
- [ ] Implement parallel processing for large migrations

**Medium Priority**:
- [ ] Complete federation integration (Loki events)
- [ ] Add comprehensive unit test coverage
- [ ] Reduce bash script complexity

**Low Priority**:
- [ ] Session file compression option
- [ ] Web-based dashboard for analytics
- [ ] Advanced conflict resolution UI

---

## 💡 How to Approach Common Tasks

### Adding a New Command

1. **Design**: What does the command do? Where in `claude-manager.sh`?
2. **Implement**: Add function to main script
3. **Test**: Write integration test
4. **Document**: Update QUICK_REFERENCE.md + ONBOARDING.md
5. **Commit**: Atomic commit with updated docs

### Fixing a Bug

1. **Reproduce**: Write test that demonstrates bug
2. **Fix**: Minimal change to fix root cause
3. **Verify**: Test passes, no regressions
4. **Document**: Update TODO.md (move to completed)
5. **Commit**: Reference issue/observation in message

### Enhancing Safety

1. **Analyze**: What could go wrong?
2. **Design**: What backup/undo mechanism needed?
3. **Implement**: Add safety layer
4. **Test**: Ensure rollback works perfectly
5. **Document**: Explain new safety feature in ONBOARDING.md

### Integrating with Federation

1. **Check**: federation-integration/ TypeScript foundation
2. **Design**: How should Loki events be emitted?
3. **Implement**: Add event emission to appropriate places
4. **Test**: Verify events in Loki
5. **Document**: Update CLAUDE.md integration section

---

## 🎓 Onboarding New Agents

When a new agent starts work here:

1. **Read this file** (CLAUDE.md) - You are here ✅
2. **Read GETTING_STARTED.md** - Understand user experience
3. **Read ONBOARDING.md** - Complete reference
4. **Run QUICK_REFERENCE.md examples** - Hands-on verification
5. **Review claude-manager.sh** - Understand main logic
6. **Check federation-integration/** - See integration patterns
7. **Run tests** - Verify everything works
8. **Start on a small task** - docs update or simple fix

---

## 🔗 Relevant External Resources

**In Your Federation**:
- `~/docs/federation/` - STOP protocol, agent coordination
- `~/docs/architecture/` - System design patterns
- `~/nabia/core/hooks/` - Hook system reference
- `~/.config/nabi/` - Nabi CLI configuration

**In This Project**:
- `federation-integration/README.md` - Federation patterns
- `docs/REPOSITORY_ANALYSIS_2025-09-24.md` - Deep technical analysis
- `Taskfile.yml` - Automation tasks

---

## 🚀 Next Steps for Agents

### If You're Starting Fresh:
1. Install the tool locally: `./install.sh`
2. Run `cm list` to see your Claude projects
3. Try a dry-run: `CLAUDE_DRY_RUN=true cm migrate /old /new`
4. Read ONBOARDING.md for complete understanding

### If You're Fixing an Issue:
1. Check TODO.md for known issues
2. Reproduce the issue (write a test)
3. Fix minimally
4. Update TODO.md and documentation
5. Create atomic commit

### If You're Adding a Feature:
1. Design (discuss with orchestrator if major)
2. Implement with tests
3. Update all documentation
4. Test edge cases
5. Atomic commit with full context

### If You're Integrating Federation:
1. Review federation-integration/src/
2. Design Loki event schema
3. Implement TypeScript event emitter
4. Test with Loki dashboard
5. Document in CLAUDE.md

---

## 📊 Project Health Indicators

Monitor these to keep project healthy:

- ✅ All tests passing (run: `task test`)
- ✅ No hardcoded paths (audit: `grep -r "/Users/tryk" .`)
- ✅ Documentation synchronized
- ✅ Undo mechanism always working
- ✅ Backups being created correctly
- ✅ XDG compliance maintained

---

## 🎯 Success Criteria for Work in This Project

When you finish any task, verify:

- ✅ **Correctness**: Does it solve the stated problem?
- ✅ **Safety**: Are backups and undo working?
- ✅ **Testing**: Are new tests written? Do they pass?
- ✅ **Documentation**: Are QUICK_REFERENCE.md, ONBOARDING.md updated?
- ✅ **Portability**: Does it work on macOS/Linux/WSL?
- ✅ **XDG Compliance**: No hardcoded absolute paths?
- ✅ **Backwards Compatibility**: Does existing code still work?
- ✅ **Git Quality**: Atomic commit with clear message?

---

## 🏁 Final Reminders

1. **This tool is production-grade** - People rely on it to preserve conversation history
2. **Safety first** - When in doubt, add another backup layer
3. **Document as you go** - Don't defer documentation
4. **Test edge cases** - Large migrations, permission issues, symlinks, etc.
5. **Keep it simple** - Bash is already complex enough
6. **Federation-ready** - Build with hooks/events in mind even if not activated yet

---

**You're now ready to work on claude-manager as an agent!**

For questions about this project, refer back to this CLAUDE.md. For user guidance, point them to GETTING_STARTED.md or QUICK_REFERENCE.md.

**Last Updated**: 2025-01-08
**Maintained By**: Architecture-focused agents
**Next Review**: When significant features added or federation integrated
