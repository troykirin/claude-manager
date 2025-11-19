# XDG Directory Visual Hierarchy
**Complete map of hub/spoke navigation structure** | **Generated**: 2025-11-10

## High-Level Hub Architecture

```
                        ~/.nabi/ [Hub Root]
                        ├─ 77 directories
                        ├─ 46 markdown files
                        └─ 2.6MB total

            ┌───────────────┬───────────────┬───────────────┬───────────────┐
            │               │               │               │               │
            ▼               ▼               ▼               ▼               ▼
         cache@          config@           data@          state@        venvs@
    [Ephemeral]     [Tool Registry]  [Binaries]     [Runtime]     [BROKEN]
         │               │                 │              │            │
         ▼               ▼                 ▼              ▼            ▼
    ~/.cache/        ~/.config/      ~/.local/      ~/.local/    ~/.cache/
    nabi/            nabi/          share/nabi/     state/nabi/
                                                      
    408MB          360MB             24.8MB         ~50MB         (wrong target)
```

## Detailed Hub Spokes

### 1. Cache Spoke (~/.nabi/cache@ → ~/.cache/nabi/)

```
~/.cache/nabi/ [408MB - Ephemeral]
├── builds/                    [Build artifacts]
│   └── [cached compilation outputs]
├── build-cache/              [Intermediate builds]
│   └── [cc objects, deps]
├── codebase-graphs/          [15 code graph caches]
│   ├── function-graphs/
│   ├── call-graphs/
│   └── dependency-graphs/
├── hook-backups/             [6 hook configuration backups]
├── hooks/                    [6 active hook systems]
│   └── [hook scripts, metadata]
├── nabi-cli/                 [CLI cache]
│   └── [command completions, cache]
├── context/                  [AI context files]
├── mcp/                      [MCP server cache]
├── health_cache.json         [System health snapshot]
└── completion-debug.log      [177MB - Shell completion debug]

🔑 KEY: This is EPHEMERAL - rebuilt on next run
   Used for: Build optimization, completion caching
   Safe to delete: YES (will rebuild)
```

### 2. Config Spoke (~/.nabi/config@ → ~/.config/nabi/)

```
~/.config/nabi/ [360MB - Configuration]
├── .git/                     [Git repository for config]
│   └── [version control of config files]
├── .config-state/            [Config state tracking]
├── .sync-state/              [Syncthing state]
├── agents/                   [4 agent configurations]
│   └── [agent personality configs]
├── auras/                    [6 aura definitions]
│   └── [visual/behavioral styles]
├── adapters/                 [6 adapter configs]
│   └── [service adapters]
├── tools/                    [20 tool TOML configs]
│   ├── claude-manager.toml       ✅ EXISTS
│   ├── claude-session-tui.toml   ❌ MISSING
│   ├── riff-cli.toml
│   ├── atomic-flow/
│   ├── link-mapper.toml
│   └── [15 others]
├── archived/                 [Tool configs no longer active]
├── cli/                      [CLI configuration]
│   └── [CLI-specific settings]
├── asciinema.conf           [ASCII recording config]
├── broker.toml              [Message broker config]
├── federation-registry.toml [Federation service registry]
├── README_DOCUMENTATION.md  [Config guide]
├── CLAUDE.md               [Config project identity]
├── CONFIG_GOVERNANCE_SURVEY_2025-11-07.md
└── [6 other config files]

🔑 KEY: This is AUTHORITATIVE SOURCE for tool behavior
   Used for: Tool discovery, schema validation
   Safe to delete: NO (configs would be lost)
```

### 3. Data Spoke (~/.nabi/data@ → ~/.local/share/nabi/)

```
~/.local/share/nabi/ [24.8MB - Permanent Storage]

├── bin/                      [138 binaries + scripts - 7.1MB]
│   ├── claude-manager*                  [bash script, 93KB]
│   ├── claude-session-tui@              [symlink → monorepo] ⭐ TUI TARGET
│   ├── claude-highlight*               [bash script, 2KB]
│   ├── consolidate-claude-session*     [bash script]
│   ├── consolidate-session*            [bash script]
│   ├── merge-sessions*                 [bash script]
│   ├── capture-tui@                    [symlink → obsidian plugin]
│   ├── [50+ Python utility scripts]
│   │   ├── ascii-diagram-check
│   │   ├── ascii-diagram-validator
│   │   ├── codex-wrapped
│   │   └── [47 others]
│   ├── [30+ shell utility scripts]
│   ├── [10+ symlinks to external tools]
│   │   ├── claude-loki-bridge@ → ~/.local/share/uv/tools/
│   │   ├── claude_search@ → ~/nabia/embed-store/
│   │   ├── cm@ → ~/.local/bin/claude-manager
│   │   └── [7 others]
│   └── adapters/                      [subdirectory of adapters]
│
├── lib/                      [42 directories - Libraries/Modules]
│   ├── __pycache__/          [Python compilation cache]
│   ├── atomic/               [9 subdirs - Atomic commit system]
│   │   ├── validator/
│   │   ├── processor/
│   │   └── [7 others]
│   ├── vigil/                [18 subdirs - Monitoring system]
│   │   ├── grafana/          [Grafana dashboard configs]
│   │   ├── loki/             [Log aggregation configs]
│   │   ├── prometheus/       [Metrics]
│   │   └── [15 others]
│   ├── [39 other Python/library subdirs]
│   │
│   └── Metadata Files:
│       ├── PORT_REGISTRY_RESILIENCE.md
│       ├── PORT_VALIDATION_QUICK_REFERENCE.md
│       ├── RESILIENCE_IMPLEMENTATION_SUMMARY.md
│       └── [3 more doc files]
│
├── link-mapper/              [17 subdirs - Link mapping]
│   ├── config.toml          [Schema-driven config]
│   ├── nodes/               [Link mapper data]
│   └── [15 others]
│
├── surrealdb/                [5 subdirs - Database]
│   ├── data/                [Database files]
│   ├── schema/              [Database schema definitions]
│   └── [3 others]
│
├── tmux-tests/               [12 subdirs - Tmux test data]
│   ├── session-*.json       [Test session files]
│   └── [11 others]
│
├── vigil/                    [9 subdirs - Monitoring data]
│   ├── dashboards/
│   ├── alerts/
│   └── [7 others]
│
├── archives/                 [4 subdirs - Version archives]
│   └── [Old tool versions]
│
├── aura/                     [4 subdirs - Aura data]
│   └── [Personality configs]
│
├── backups/                  [6 subdirs - Backup snapshots]
│   └── [Backup files]
│
├── tools/                    [Utility tools directory]
├── artifacts/                [Build artifacts]
├── embeddings/               [Vector embeddings]
├── history/                  [Command history]
├── knowledge/                [Knowledge base files]
├── logs/                     [System logs]
├── manifests/                [7 subdirs - Manifest files]
├── metrics/                  [Performance metrics]
├── registry/                 [Tool registry files]
├── tools.json                [Tool discovery registry - INCOMPLETE]
├── consciousness.db          [Knowledge database]
├── claude-export-index.jsonl [12MB - Session export]
└── DEPENDENCY_MAP.md         [Dependency tracking]

🔑 KEY: This is PERMANENT STORAGE - do not delete
   Used for: Binary distribution, permanent data
   Safe to delete: NO (would lose tools and data)
```

### 4. State Spoke (~/.nabi/state@ → ~/.local/state/nabi/)

```
~/.local/state/nabi/ [~50MB - Runtime State]
├── tools/                    [Tool runtime state]
│   ├── registry.json        [Derived from TOML schemas]
│   └── [command state]
├── hooks/                    [Hook execution state]
├── coordination/             [Multi-agent coordination]
├── logs/                     [Runtime logs]
│   ├── memchain.log
│   ├── nabi-cli.log
│   ├── hook-execution.log
│   └── [federation logs]
├── backups/                  [Session backups]
│   └── claude-manager.last_move_operation
├── cache/                    [State cache]
└── [ephemeral coordination files]

🔑 KEY: This is RUNTIME STATE - can be rebuilt
   Used for: Coordination, logging, temporary state
   Safe to delete: PARTIALLY (deletes logs/state, rebuilds on next run)
```

### 5. Docs Spoke (~/.nabi/docs@ → ~/Sync/docs/)

```
~/Sync/docs/ [External to XDG]
├── [130+ documentation files]
├── architecture/            [System design docs]
├── federation/              [Federation patterns]
├── infrastructure/          [Infrastructure docs]
├── knowledge/               [Knowledge base]
├── tools/                   [Tool documentation]
├── projects/                [Project docs]
└── [many subdirs]

⚠️  NOTE: This is synced via Syncthing, NOT XDG
   Issue: Breaks portability, not standard location
   Recommendation: Eventually move to ~/.local/share/nabi/docs
```

### 6. Platform Spoke (~/.nabi/platform@ → ~/nabia/platform/)

```
~/nabia/platform/ [Code repository]
├── [Platform layer code]
├── drivers/
├── adapters/
└── [implementation files]

🔑 KEY: This is SOURCE CODE - points to monorepo
   Used for: Platform abstraction, driver management
   Part of: ~/nabia monorepo structure
```

---

## Symlink Chain for claude-session-tui

### Complete Discovery Path

```
User runs: claude-session-tui

Step 1: Shell searches PATH
   ├─ Checks: /usr/local/bin
   ├─ Checks: /usr/bin
   └─ Checks: ~/.local/bin  ◄─── FOUND HERE

Step 2: Resolve ~/.local/bin/claude-session-tui
   └─ Is symlink? YES
   └─ Points to: [See ~/.local/bin directory listing]
   └─ Actual target: ~/.local/share/nabi/bin/claude-session-tui

Step 3: Resolve ~/.local/share/nabi/bin/claude-session-tui@
   └─ Is symlink? YES
   └─ Points to: /Users/tryk/nabia/tools/claude-manager/claude-session-tui/target/release/claude-session-tui

Step 4: Execute final binary
   ├─ Type: Mach-O 64-bit executable arm64
   ├─ Size: 4.7MB
   ├─ Built: Oct 31, 2025
   └─ Status: ✅ EXECUTABLE

TOTAL RESOLUTION TIME: ~5-10ms
```

### Visual Symlink Chain

```
User PATH
    │
    ├─ ~/.local/bin/ (50 items including symlinks)
    │   │
    │   └─ claude-session-tui@ ─────┐
    │       (symlink)                 │
    │                                  │
    └──────────────────────────────────┤
                                       │
                    ┌──────────────────┘
                    │
                    ▼
    ~/.local/share/nabi/bin/ (138 items)
         │
         └─ claude-session-tui@ ─────┐
             (symlink)                 │
             [Maintained here]         │
                                       │
                    ┌──────────────────┘
                    │
                    ▼
    ~/nabia/tools/claude-manager/ (Monorepo)
         │
         └─ claude-session-tui/
             ├─ src/              [Rust source]
             ├─ Cargo.toml        [Build config]
             └─ target/
                 └─ release/
                     └─ claude-session-tui ◄─── EXECUTABLE BINARY
                         (4.7MB arm64 Mach-O)
                         (Source of truth for updates)
```

---

## Bin Directory Detailed Inventory

### ~/.local/share/nabi/bin/ (138 items, 7.1MB)

#### Binary Types Distribution

```
Type                Count      Size        Status
────────────────────────────────────────────────────
Rust Binaries       2          4.7MB       ✅ Active
Python Scripts      50+        ~2MB        ✅ Active
Shell Scripts       60+        ~500KB      ✅ Active
Symlinks (ext)      10         varies      ✅ Active
Symlinks (local)    8          ~200KB      ✅ Active
Subdirectories      2          ~100KB      ✅ Active
────────────────────────────────────────────────
TOTAL               138        7.1MB
```

#### Rust Binaries in bin/

```
✅ claude-session-tui@
   Type: Symlink → monorepo
   Size: 4.7MB (resolved)
   Built: Oct 31, 2025
   Status: Production-ready
   Config: MISSING - needs ~/.config/nabi/tools/claude-session-tui.toml

[Potentially others not yet catalogued]
```

#### Python Scripts Sample

```
ascii-diagram-check           (Diagram validation)
ascii-diagram-validator       (Validation tool)
codex-wrapped                 (Code analysis wrapper)
codegraph-post-index.sh       (Code graph post-processing)
codegraph-pre-index.sh        (Code graph pre-processing)
codegraph-pre-query.sh        (Code graph query hook)
[40+ more Python utilities]
```

#### Shell Scripts Sample

```
claude-manager*               (93KB - Session migration tool)
consolidate-claude-session*   (Session consolidation)
consolidate-session*          (Generic session consolidation)
merge-sessions*               (Session merging)
asciinema-shadowed.sh        (ASCII recording)
asciinema-tmux.sh            (Tmux ASCII recording)
backup-zip.sh                (Backup utility)
[50+ more shell scripts]
```

#### External Tool Symlinks

```
claude-loki-bridge@          → ~/.local/share/uv/tools/memchain/bin/
capture-tui@                 → ~/nabia/plugins/obsidian-tui-capture/scripts/
claude_search@               → ~/nabia/embed-store/claude_search
cx@, cx-log@, cx-rollup@     → Development scripts
cm@                          → ~/.local/bin/claude-manager
[5+ more external links]
```

---

## Lib Directory Detailed Structure

### ~/.local/share/nabi/lib/ (42 directories, 2.3MB)

```
atomic/ [9 subdirs]
├── validator/         [Atomic commit validation]
├── processor/         [Atomic processing logic]
├── [7 others]        
└── [Total: ~500KB for atomic subsystem]

vigil/ [18 subdirs]
├── grafana/          [Dashboard definitions]
│   ├── provisioning/
│   ├── configs/
│   └── dashboards/
├── loki/             [Log aggregation]
│   ├── config/
│   └── rules/
├── prometheus/       [Metrics]
├── alerting/         [Alert rules]
└── [11 others]
└── [Total: ~1MB for monitoring subsystem]

__pycache__/ [9 subdirs]
├── [Python bytecode cache for tools]
└── [Auto-regenerated]

[29 other Python package directories]
└── [Various library implementations]

[3 metadata markdown files]
├── PORT_REGISTRY_RESILIENCE.md
├── PORT_VALIDATION_QUICK_REFERENCE.md
└── RESILIENCE_IMPLEMENTATION_SUMMARY.md
```

---

## Path Resolution Summary Table

| Layer | Directory | Type | Size | Purpose | Status |
|-------|-----------|------|------|---------|--------|
| **Hub** | ~/.nabi/ | Symlink hub | 2.6MB | Navigation | ✅ |
| **Cache** | ~/.cache/nabi/ | Ephemeral | 408MB | Build artifacts | ✅ |
| **Config** | ~/.config/nabi/ | Authoritative | 360MB | Tool schemas | ✅ |
| **Data** | ~/.local/share/nabi/ | Permanent | 24.8MB | Binaries + libs | ✅ |
| **State** | ~/.local/state/nabi/ | Runtime | ~50MB | Coordination | ✅ |
| **Docs** | ~/Sync/docs/ | Synced | External | Documentation | ⚠️ Non-XDG |
| **Platform** | ~/nabia/platform/ | Code | External | Platform layer | ✅ |
| **PATH** | ~/.local/bin/ | Symlinks | 832MB | User binaries | ✅ |

---

## Integration Checklist

### TUI Binary Integration Status

```
╔════════════════════════════════════════════════════════════╗
║         Claude-Session-TUI Integration Matrix              ║
╠════════════════════════════════════════════════════════════╣
║                                                            ║
║ ✅ Binary compiled and working                             ║
║    Location: ~/nabia/tools/claude-manager/target/release/ ║
║    Size: 4.7MB (arm64 Mach-O)                             ║
║                                                            ║
║ ✅ Symlinked to data layer                                 ║
║    Location: ~/.local/share/nabi/bin/claude-session-tui@  ║
║    Method: Preserves monorepo as source of truth           ║
║                                                            ║
║ ✅ Accessible via PATH                                     ║
║    Command: claude-session-tui [args]                     ║
║    Resolution: < 10ms                                      ║
║                                                            ║
║ ✅ User documentation                                      ║
║    File: TUI_QUICK_START.md                               ║
║    Content: Complete usage guide                          ║
║                                                            ║
║ ❌ MISSING: Tool configuration file                        ║
║    File: ~/.config/nabi/tools/claude-session-tui.toml     ║
║    Action: Create from template (see summary doc)         ║
║    Priority: HIGH - Required for nabi CLI integration     ║
║                                                            ║
║ ❌ MISSING: Tool registry entry                            ║
║    File: ~/.local/share/nabi/tools.json                   ║
║    Action: Add claude-session-tui entry                   ║
║    Priority: MEDIUM - For discovery enhancement           ║
║                                                            ║
║ ⚠️  INCOMPLETE: Version tracking                           ║
║    Status: Only in source Cargo.toml                      ║
║    Action: Add version to TOML config                     ║
║    Priority: MEDIUM - For release management              ║
║                                                            ║
╚════════════════════════════════════════════════════════════╝
```

---

## Quick Navigation Reference

### To access TUI:
```bash
# Direct command (works now)
claude-session-tui [args]

# Via nabi (after TOML created)
nabi exec claude-session-tui [args]

# Edit config
vim ~/.config/nabi/tools/claude-session-tui.toml

# Check PATH
which claude-session-tui

# Verify symlink
readlink -f ~/.local/share/nabi/bin/claude-session-tui
```

### To understand structure:
```bash
# View hub spokes
ls -la ~/.nabi/

# Check symlink targets
cd ~/.nabi && for link in cache config data state; do
  echo "$link → $(readlink -f $link)"
done

# List all 138 binaries
ls -lh ~/.local/share/nabi/bin/ | wc -l

# View tools configuration
ls -la ~/.config/nabi/tools/
```

---

**END OF VISUAL HIERARCHY**

This document is the companion to XDG_STRUCTURE_MAP.md (detailed contents) and XDG_INTEGRATION_SUMMARY.md (integration checklist).
