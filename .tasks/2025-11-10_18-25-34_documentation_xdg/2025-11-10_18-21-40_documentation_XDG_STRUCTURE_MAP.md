# XDG Directory Structure Map for Claude Manager TUI Integration
# Generated: 2025-11-10
# Purpose: Document hub/spoke navigation and TUI binary placement

## 1. HUB STRUCTURE (~/.nabi/)
## Navigation Hub with Symlink Spokes

~/.nabi/ (Hub Directory)
├── cache@        → ~/.cache/nabi           (Ephemeral build artifacts, hot cache)
├── config@       → ~/.config/nabi          (TOML schemas, auras, hooks, tool registry)
├── data@         → ~/.local/share/nabi     (Permanent storage: binaries, databases, tools)
├── docs@         → ~/Sync/docs             (Syncthing-synced documentation)
├── platform@     → ~/nabia/platform        (Platform layer code)
├── state@        → ~/.local/state/nabi     (Runtime state: coordination, logs, ephemeral)
└── venvs@        → ~/.cache                (Python virtual environments symlink - NOTE: BROKEN)

**NOTE on venvs**: Current symlink points to ~/.cache instead of ~/.cache/nabi/venvs
  - Should be: ~/.nabi/venvs@ → ~/.cache/nabi/venvs/
  - Impact: Python venvs may not be discoverable
  - Status: Known issue, low priority


## 2. DATA LAYER (~/.local/share/nabi/)
## Permanent storage for tools, binaries, and data

~/.local/share/nabi/
├── bin/                    (138 binaries + scripts)
│   ├── claude-manager*          (Shell script, 93KB, v1.0.0)
│   ├── claude-session-tui@      (Symlink → ~/nabia/tools/claude-manager/target/release/)
│   ├── claude-loki-bridge@      (Symlink → uv tools)
│   ├── consolidate-claude-session*
│   ├── consolidate-session*
│   ├── merge-sessions*
│   ├── capture-tui@
│   ├── [100+ other utility scripts/binaries]
│   └── [Python scripts, shell scripts, Rust binaries]
│
├── lib/                    (42 subdirectories)
│   ├── __pycache__/        (Python cache)
│   ├── atomic/             (Atomic commit system)
│   └── vigil/              (Strategic oversight system)
│       ├── [Loki configs, Grafana dashboards, monitoring code]
│
├── link-mapper/            (17 subdirectories)
│   ├── config.toml         (Link mapper configuration)
│   └── [Federation link mapping system]
│
├── surrealdb/              (5 subdirectories)
│   ├── [Database files, schemas, backups]
│
├── tmux-tests/             (12 subdirectories)
│   ├── [Tmux coordination test data]
│
├── vigil/                  (9 subdirectories)
│   ├── [Monitoring system data]
│
├── archives/               (4 subdirectories)
│   ├── [Archived tool versions]
│
├── aura/                   (4 subdirectories)
│   ├── [Aura configurations]
│
├── backups/                (6 subdirectories)
│   ├── [Backup snapshots]
│
├── tools.json              (Tool registry: lists riff CLI)
├── consciousness.db        (Knowledge database)
├── claude-export-index.jsonl (Large export file, 12MB)
├── DEPENDENCY_MAP.md       (Tool dependency tracking)
└── [Various metadata files and databases]


## 3. INSTALLATION STRATEGY FOR CLAUDE-SESSION-TUI
## Current Pattern & Integration Points

### Current State (As of 2025-11-10)
- Rust binary location: ~/nabia/tools/claude-manager/claude-session-tui/target/release/claude-session-tui
- Size: 4.7MB (Mach-O 64-bit executable arm64)
- Symlink location: ~/.local/share/nabi/bin/claude-session-tui@
- Status: Linked but possibly not registered in tool discovery

### Installation Pattern Observed:
1. **Source:** ~/nabia/tools/<tool-name>/target/release/<binary>
   (Rust projects compiled in monorepo)

2. **Distribution:** ~/.local/share/nabi/bin/<binary>
   (Via symlink, preserves monorepo source of truth)

3. **Discovery:** 
   - Direct: PATH includes ~/.local/bin (via symlink to ~/.local/share/nabi/bin)
   - Via tool registry: ~/.config/nabi/tools/<tool>.toml
   - Via nabi CLI: `nabi exec <tool>`

4. **Configuration:** ~/.config/nabi/tools/claude-manager.toml
   (Declarative tool metadata)


## 4. RELATED TOOLS STRUCTURE
## Understanding integration patterns

### Python-based tools (via uv):
- Location: ~/.local/share/uv/tools/<project>/bin/
- Example: claude-loki-bridge@ → ~/.local/share/uv/tools/memchain/bin/claude-loki-bridge
- Pattern: Symlinked from ~/.local/share/nabi/bin/

### Rust-based tools (from monorepo):
- Source location: ~/nabia/<project>/target/release/<binary>
- Dist location: ~/.local/share/nabi/bin/<binary>
- Method: Symlink
- Example: claude-session-tui (current project)

### Shell scripts:
- Stored directly: ~/.local/share/nabi/bin/<script>
- No symlink needed
- Examples: claude-manager, atomic, consolidate-*


## 5. CONFIGURATION & DISCOVERY
## How tools are registered and discovered

### Tool Registry (Schema-driven):
- Location: ~/.config/nabi/tools/<tool>.toml
- Format: TOML with standardized schema
- Contents: name, version, description, source, runtime, capabilities, commands

### Tool Registration JSON:
- Location: ~/.local/share/nabi/tools.json
- Content: riff CLI registered (v2.0.0)
- Use: Fallback discovery if TOML not found

### Nabi CLI Integration:
- Command: nabi exec <tool>
- Behavior: Loads from ~/.config/nabi/tools/*.toml
- Example: nabi exec claude-manager → runs ~/.local/bin/claude-manager
- Status: Active and working

### Environment Override Pattern:
- Example: NABI_TOOL_RIFF_PATH for tool path resolution
- Purpose: Allow runtime override of tool location
- Used for: Reliable path discovery across platforms


## 6. BIN AND LIB DIRECTORIES STRUCTURE
## Detailed breakdown

### ~/.local/share/nabi/bin/
Total: 138 entries (Mach-O executables, scripts, symlinks, directories)

#### Binary Types:
1. Rust Binaries (Mach-O 64-bit arm64):
   - claude-session-tui@ (symlink)
   - [Others built in monorepo]

2. Python Scripts:
   - ascii-diagram-check
   - ascii-diagram-validator
   - codex-wrapped
   - [40+ Python utilities]

3. Shell Scripts:
   - claude-manager (bash, 93KB)
   - consolidate-*.sh
   - asciinema-*.sh
   - [60+ shell utilities]

4. Symlinks to External Tools:
   - claude-loki-bridge@ → ~/.local/share/uv/tools/memchain/bin/
   - capture-tui@ → ~/nabia/plugins/obsidian-tui-capture/scripts/
   - cx*, cx-log*, cx-rollup*, cx-ship* → Development scripts
   - [10+ external tool symlinks]

5. Subdirectories:
   - adapters/ (4 items)
   - backups/ (6 items)

#### Size Analysis:
- Total bin directory: 7.1MB
- Largest executable: gopro (117MB - external binary)
- Average script: 1-20KB
- Pattern: Mix of maintained tools and legacy binaries


### ~/.local/share/nabi/lib/
Total: 42 entries (Mostly subdirectories with config/code)

#### Structure:
1. Code/Config Directories:
   - atomic/ (9 subdirs) - Atomic commit system
   - vigil/ (18 subdirs) - Strategic oversight system
   - link-mapper/ - Not in lib, but referenced

2. Python Cache:
   - __pycache__/ (9 subdirs) - Compiled Python bytecode

3. Metadata:
   - PORT_REGISTRY_RESILIENCE.md
   - PORT_VALIDATION_QUICK_REFERENCE.md
   - RESILIENCE_IMPLEMENTATION_SUMMARY.md
   - [Documentation and analysis files]

#### Purpose:
- lib/ = Libraries, modules, shared code
- bin/ = Executable entry points
- Pattern matches monorepo structure


## 7. XDG COMPLIANCE ANALYSIS
## Current state and recommendations

### Compliant Areas:
✅ Cache directory: ~/.cache/nabi (XDG_CACHE_HOME)
✅ Config directory: ~/.config/nabi (XDG_CONFIG_HOME)
✅ Data directory: ~/.local/share/nabi (XDG_DATA_HOME)
✅ State directory: ~/.local/state/nabi (XDG_STATE_HOME)
✅ Hub symlinks: Clear spoke pattern with ~/ relative paths

### Non-compliant Areas:
❌ docs symlink: Points to ~/Sync/docs (not XDG path)
❌ venvs symlink: Points to ~/.cache instead of ~/.cache/nabi/venvs

### Path Resolution:
✅ Uses $HOME/.nabi/ expansion (portable)
✅ Handles ~ prefix expansion
✅ No hardcoded /Users/tryk/ paths in tool configs
⚠️ Some legacy tools may have absolute paths


## 8. CLAUDE-SESSION-TUI INTEGRATION CHECKLIST
## Current status and next steps

### Current Integration:
✅ Binary compiled: ~/nabia/tools/claude-manager/claude-session-tui/target/release/
✅ Symlinked: ~/.local/share/nabi/bin/claude-session-tui@
✅ Accessible via PATH: ~/.local/bin symlinks to ~/.local/share/nabi/bin
✅ Documented: TUI_QUICK_START.md exists

### Still Needed:
❌ Configuration file: ~/.config/nabi/tools/claude-session-tui.toml
❌ Tool registry entry: tools.json update
❌ Nabi CLI integration: nabi exec claude-session-tui
❌ Installation documentation: clear setup guide
⚠️ Version tracking: Not integrated with version system

### Path to Integration:
1. Create ~/.config/nabi/tools/claude-session-tui.toml
   - Pattern: Use claude-manager.toml as template
   - Entry point: claude-session-tui (matches binary name)
   - Capabilities: federation_aware = false (for now)
   - Location: ~/nabia/tools/claude-manager/

2. Update ~/.local/share/nabi/tools.json
   - Add claude-session-tui entry
   - Version: Extract from binary metadata or Cargo.toml
   - Source: Link to Cargo project

3. Test discovery:
   - Command: which claude-session-tui
   - Command: nabi exec claude-session-tui --help
   - Command: ~/.local/bin/claude-session-tui list

4. Update installation guide:
   - Document binary installation
   - Explain symlink structure
   - Provide version update instructions


## 9. DIRECTORY SUMMARY TABLE
## Quick reference for all XDG locations

| Location | Type | Purpose | Size | Status |
|----------|------|---------|------|--------|
| ~/.nabi/ | Hub | Symlink navigation | 2.6MB | ✅ Active |
| ~/.cache/nabi | Cache | Ephemeral artifacts | 408MB | ✅ Active |
| ~/.config/nabi | Config | Schemas, auras, tools | 360MB | ✅ Active |
| ~/.local/share/nabi | Data | Binaries, databases, libs | 24.8MB | ✅ Active |
| ~/.local/state/nabi | State | Runtime coordination | ~50MB | ✅ Active |
| ~/Sync/docs | Docs | Syncthing-synced | External | ✅ Active |
| ~/nabia/platform | Code | Platform layer | External | ✅ Active |
| ~/.local/bin | Symlinks | User PATH binaries | 832MB | ✅ Active |


## 10. KNOWN ISSUES & OBSERVATIONS
## Things to be aware of

1. **venvs symlink broken**: Points to ~/.cache instead of ~/.cache/nabi/venvs/
   - Impact: Python venvs may not auto-discover
   - Fix: Update symlink target
   - Priority: Low

2. **Large external binaries in ~/.local/bin/**:
   - gopro: 117MB
   - ghci-dap: 128MB
   - gp_splitter: 20MB
   - gp_sync: 20MB
   - These should probably move to ~/.local/share/nabi/bin/
   - But current setup works fine

3. **Tool registry incomplete**:
   - tools.json only lists riff CLI
   - Most tools discovered via PATH + TOML configs
   - This is acceptable pattern

4. **Binary symlinks in ~/.local/bin/**:
   - Approach: ~/.local/bin/ contains symlinks to files/directories
   - Example: claude@ → /Users/tryk/.local/share/claude/versions/2.0.34
   - Pattern: Works, but creates many 50 entry directory
   - Could consolidate, but not critical

5. **Documentation scattered**:
   - Some in ~/.local/share/nabi/lib/
   - Some in ~/Sync/docs (syncthing)
   - Some in project source
   - No unified documentation registry

## RECOMMENDATIONS FOR TUI PLACEMENT

1. **Binary Location**: ✅ Already correct
   - Source: ~/nabia/tools/claude-manager/claude-session-tui/target/release/claude-session-tui
   - Dist: ~/.local/share/nabi/bin/claude-session-tui@ (symlink)
   - Reason: Preserves monorepo as source of truth

2. **Configuration**: Create ~/.config/nabi/tools/claude-session-tui.toml
   - Template from: claude-manager.toml
   - Entry: point = "claude-session-tui"
   - Capabilities: federation_aware = false (unless planning integration)

3. **Discovery**: Update tools.json registry
   - Add: claude-session-tui entry
   - Version: From Cargo.toml
   - Description: "Interactive TUI for browsing Claude sessions"

4. **Integration**: No changes needed to CLI structure
   - Tool already in PATH via ~/.local/bin symlinks
   - Symlink structure working correctly
   - User can run: claude-session-tui [args]
   - Nabi can run: nabi exec claude-session-tui [args]

5. **Installation**: Update install.sh script
   - Verify symlink exists
   - Update version in Cargo.toml
   - Document TUI in README.md
   - Add to QUICK_REFERENCE.md

