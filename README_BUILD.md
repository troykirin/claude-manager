# Claude Manager - Build & Organization Guide

> **TL;DR**: `just quick` to build and install, `just dev` for development with watch mode.

---

## What's Here?

Claude Manager is a comprehensive suite of tools for managing Claude Code sessions:

- **claude-session-tui**: Interactive TUI for browsing sessions
- **claude-manager**: Bash script for session migration and management
- **Build System**: Modern justfile-based build system
- **Federation Integration**: Ready for nabi CLI integration

---

## Getting Started (60 seconds)

### Install & Run
```bash
just quick              # Build + install binaries
just verify             # Confirm everything works
claude-session-tui      # Run the TUI
```

### Develop
```bash
just dev                # Build, install, then watch for changes
# Edit code in claude-session-tui/src/
# Files automatically rebuild and reinstall
```

---

## Key Files & Purposes

| File | Purpose |
|------|---------|
| **justfile** | Main build interface (replaces Makefile) |
| **.cargo/config.toml.template** | XDG-compliant Cargo configuration |
| **claude-session-tui/** | TUI source code (Rust) |
| **BUILD_SYSTEM.md** | Complete build documentation |
| **ORGANIZATION_COMPLETE.md** | What was organized and why |
| **XDG_INTEGRATION_COMPLETE.md** | Federation integration status |

---

## Build System Features

✅ **XDG Compliant**: Build artifacts in cache, binaries in ~/.local/bin
✅ **Fast Iteration**: Watch mode rebuilds on file changes
✅ **Verified Install**: Built-in verification of all components
✅ **Federation Ready**: Registered in tool TOML and data layer
✅ **Clean Process**: One-command workflows that do everything

---

## Common Tasks

### Development Workflow
```bash
# Start developing with auto-rebuild
just dev

# Or just watch (if already built)
just watch

# Run tests
just test
```

### Release/Production
```bash
# Full release process
just release-workflow  # clean → test → install → verify

# Or quick rebuild
just clean && just install
```

### Maintenance
```bash
# Check configuration
just config

# Verify installation
just verify

# Uninstall everything
just uninstall
```

---

## Where Things Are Installed

### Binaries
```
Source:  ~/nabia/tools/claude-manager/claude-session-tui/target/release/
PATH:    ~/.local/bin/claude-session-tui
Data:    ~/.local/share/nabi/bin/claude-session-tui
```

### Completions
```
Generated: ~/.cache/zsh/completions/_claude-session-tui
```

### Configuration
```
TOML:      ~/.config/nabi/tools/claude-session-tui.toml
```

---

## Architecture Overview

```
justfile (build rules)
    ├─ build      → Compiles in cache directory
    ├─ install    → Copies to PATH + data layer
    ├─ completions → Generates zsh completions
    ├─ test       → Runs test suite
    ├─ watch      → Auto-rebuilds on changes
    └─ verify     → Checks all components
```

### XDG Structure
```
~/.nabi/data@           → ~/.local/share/nabi/
~/.nabi/config@         → ~/.config/nabi/
~/.cache/nabi/          → Build artifacts
~/.local/bin/           → PATH executables
```

---

## Troubleshooting

### "Binary not found"
```bash
just clean
just build
just install
```

### Zsh completions not working
```bash
# Add to ~/.zshrc if not already there
fpath=(~/.cache/zsh/completions $fpath)
autoload -U compinit && compinit

# Then
just completions-only
```

### Clean rebuild
```bash
just clean
just install
just verify
```

### Slow compilation
Use incremental builds:
```bash
# Instead of:
just build

# Use:
just quick-build    # Faster for small changes
```

---

## File Organization

### Source Code
```
claude-session-tui/
├── src/                      (Rust source)
│   ├── main.rs              (Entry point)
│   ├── cli.rs               (CLI argument handling)
│   ├── parser.rs            (JSONL parser)
│   ├── lib.rs               (Library exports)
│   └── [other modules]
├── Cargo.toml               (Project manifest)
├── Cargo.lock               (Dependency versions)
└── tests/                   (Integration tests)
```

### Build Outputs
```
.cargo/config.toml          (Generated from template)
.build/                     (Intermediate files)
target/ → (linked to cache)
~/.cache/nabi/claude-manager/target/
                            (Actual build artifacts)
```

### Documentation
```
BUILD_SYSTEM.md             (Detailed build docs)
ORGANIZATION_COMPLETE.md    (What was organized)
XDG_INTEGRATION_*.md        (Integration details)
README_BUILD.md             (This file)
```

---

## Quick Reference

### All Available Targets
```bash
just help                   # Show all targets
```

### Common Workflows
```bash
just quick                  # Build + install (5 min)
just dev                    # Development with watch (continuous)
just test                   # Run tests
just verify                 # Check installation
just release-workflow       # Production release
```

### Configuration
```bash
just config                 # Show build settings
```

---

## Understanding the Build System

### Why justfile instead of Makefile?
- Modern syntax (easier to read)
- Better multi-line command support
- Active development & community
- Pattern from reference implementation (`nabi-cli`)

### Why .cargo/config.toml.template?
- Substitutes actual XDG paths at build time
- Keeps build artifacts out of repo
- Follows XDG Base Directory specification
- Clean separation: source vs build

### Why dual installation locations?
- `~/.local/bin/`: Traditional PATH discovery
- `~/.local/share/nabi/bin/`: Federation registry
- Enables both direct use and nabi CLI integration

---

## Federation Integration Status

### Ready Now ✅
- [x] Tool TOML configuration (`~/.config/nabi/tools/claude-session-tui.toml`)
- [x] Binary in data layer (`~/.local/share/nabi/bin/`)
- [x] Dual installation locations
- [x] Zsh completions generated

### Soon 🔄
- [ ] Registry JSON update (`~/.local/share/nabi/tools.json`)
- [ ] Run `nabi tools transform`
- [ ] Test `nabi exec claude-session-tui`

### Future 📋
- [ ] Loki event hooks
- [ ] memchain coordination
- [ ] Multi-platform binaries (Linux, WSL)

---

## Performance Notes

- **Clean build**: ~2-3 minutes
- **Incremental build**: ~5-30 seconds
- **Install**: ~2 seconds
- **Completions**: ~1 second
- **Watch rebuild**: ~5-30 seconds

---

## Next Steps

### For Users
1. Run `just quick` to install
2. Run `claude-session-tui` to try the TUI
3. See `claude-session-tui --help` for options

### For Development
1. Run `just dev` to start development mode
2. Edit code in `claude-session-tui/src/`
3. Changes automatically rebuild and reinstall

### For Operators
1. Run `just release-workflow` for production builds
2. Run `just verify` to confirm installation
3. See `BUILD_SYSTEM.md` for advanced options

---

## Documentation Map

- **This file** (`README_BUILD.md`): Quick start guide
- **BUILD_SYSTEM.md**: Comprehensive build documentation
- **ORGANIZATION_COMPLETE.md**: What was organized and why
- **XDG_INTEGRATION_COMPLETE.md**: Federation integration status
- **justfile**: Actual build rules (self-documenting)

---

## Support

For detailed information:
- Build processes: See `BUILD_SYSTEM.md`
- Integration: See `XDG_INTEGRATION_COMPLETE.md`
- Architecture: See `ORGANIZATION_COMPLETE.md`
- Known issues: See `CLAUDE_SESSION_TUI_HANG_INVESTIGATION.md`

---

## Summary

✅ **Modern Build System**: justfile-based, XDG-compliant
✅ **One-Command Install**: `just quick` gets you running
✅ **Development Ready**: `just dev` for continuous rebuilds
✅ **Federation Integrated**: TOML config + data layer
✅ **Well Documented**: Comprehensive guides included

**Start now**: `just quick`
**Develop**: `just dev`
**Learn more**: See `BUILD_SYSTEM.md`
