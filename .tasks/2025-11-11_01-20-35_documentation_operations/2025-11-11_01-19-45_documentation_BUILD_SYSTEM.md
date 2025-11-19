# Claude Manager Build System Documentation

**Status**: Production-Ready
**Build Tool**: Justfile (replaces Makefile)
**XDG Compliance**: Full
**Pattern**: Modeled after `~/nabia/core/nabi-cli`

---

## Quick Start

### Build & Install
```bash
just quick              # One-command build and install
just install            # Build, generate completions, install
just verify             # Check installation success
```

### Development
```bash
just dev                # Build, install, then watch for changes
just watch              # Watch-only mode (after initial build)
just test               # Run tests
```

### Cleanup
```bash
just clean              # Remove build artifacts
just uninstall          # Remove installed binaries and completions
```

---

## System Architecture

### XDG Directory Structure

```
~/.cache/nabi/claude-manager/
├── target/              (build artifacts)
│   └── release/
│       └── claude-session-tui  (binary)
└── [build cache]

~/.local/bin/            (PATH)
└── claude-session-tui   (installed binary)

~/.local/share/nabi/bin/
└── claude-session-tui   (data layer copy)

~/.cache/zsh/completions/
└── _claude-session-tui  (zsh completion)

~/.config/nabi/tools/
└── claude-session-tui.toml  (tool configuration)
```

### Build Flow

```
Source Code (monorepo)
    ↓
Just Rules (justfile)
    ├─ Config Generation (.cargo/config.toml from template)
    ├─ Cargo Build (XDG target directory)
    ├─ Completion Generation (template-based)
    ├─ Binary Installation (dual location)
    └─ Verification
```

---

## Justfile Targets

### Core Targets

#### `build`
Compiles the TUI binary with XDG-compliant paths.

```bash
just build
```

**What it does**:
1. Generates `.cargo/config.toml` from template with XDG paths
2. Runs `cargo build --release` in TUI directory
3. Stores artifacts in `~/.cache/nabi/claude-manager/target/`

#### `quick-build`
Rebuilds without regenerating cargo config (faster iteration).

```bash
just quick-build
```

#### `install`
Builds, generates completions, and installs to PATH + data layer.

```bash
just install
```

**Installs to**:
- `~/.local/bin/claude-session-tui` (PATH)
- `~/.local/share/nabi/bin/claude-session-tui` (data layer)
- `~/.cache/zsh/completions/_claude-session-tui` (completions)

#### `completions`
Generates zsh completions from template.

```bash
just completions
```

**Process**:
1. Creates template-based completion file
2. Validates syntax with `bash -n`
3. Deploys to `~/.cache/zsh/completions/`

#### `completions-only`
Regenerates completions without rebuilding binary.

```bash
just completions-only
```

### Verification & Testing

#### `verify`
Verifies all installation components are correct.

```bash
just verify
```

**Checks**:
- Binary in data layer
- Binary in PATH
- PATH discovery working (`command -v`)
- Binary functional (`--help` works)
- Completions available

#### `test`
Runs unit tests.

```bash
just test
```

#### `test-verbose`
Runs tests with output.

```bash
just test-verbose
```

### Development Mode

#### `watch`
Continuously rebuilds and reinstalls on source changes.

```bash
just watch
```

**Requires one of**:
- `cargo-watch`: `cargo install cargo-watch`
- `fswatch`: `brew install fswatch`
- `entr`: `brew install entr`

#### `dev`
Initial build + install, then starts watch mode.

```bash
just dev
```

**One-command development setup**:
- Builds everything
- Installs binaries & completions
- Starts watching for changes
- Automatically rebuilds on file changes

### Maintenance

#### `clean`
Removes all build artifacts.

```bash
just clean
```

**Removes**:
- `~/.cache/nabi/claude-manager/target/`
- `.cargo/config.toml`
- `.build/` directory

#### `uninstall`
Removes installed binaries and completions.

```bash
just uninstall
```

**Removes**:
- `~/.local/bin/claude-session-tui`
- `~/.local/share/nabi/bin/claude-session-tui`
- `~/.cache/zsh/completions/_claude-session-tui`

### Utilities

#### `config`
Shows build configuration.

```bash
just config
```

**Displays**:
- XDG directory settings
- Build paths
- Project information

#### `help`
Shows all available targets.

```bash
just help
```

### Workflows

#### `quick`
One-command build and install.

```bash
just quick
```

Equivalent to: `just build install`

#### `setup`
Full development environment setup.

```bash
just setup
```

Equivalent to: `just dev` (build + install + watch)

#### `release-workflow`
Production release process.

```bash
just release-workflow
```

**Steps**:
1. Clean build artifacts
2. Run tests
3. Build and install
4. Verify installation

---

## Configuration Files

### Cargo Config Template (`.cargo/config.toml.template`)

```toml
[build]
target-dir = "$XDG_CACHE_HOME/nabi/claude-manager/target"
jobs = 8

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = false
```

**Processing**:
- `just build` substitutes `$XDG_CACHE_HOME` with actual path
- Creates `.cargo/config.toml` (not committed)
- Reused on subsequent builds until cleaned

### Tool Configuration (`~/.config/nabi/tools/claude-session-tui.toml`)

```toml
[tool]
id = "claude-session-tui"
name = "Claude Session TUI"
version = "0.1.0"
description = "Interactive TUI for browsing Claude sessions"
status = "active"

[runtime]
language = "rust"
entry_point = "claude-session-tui"
```

**Purpose**:
- Federation tool registry
- Enables `nabi exec claude-session-tui`
- Schema-driven governance

---

## Environment Variables

### Customizable XDG Paths

```bash
# Override default locations
XDG_CACHE_HOME=~/custom/cache just install
XDG_CONFIG_HOME=~/custom/config just install
XDG_DATA_HOME=~/custom/data just install
```

### Build Configuration

```bash
# Number of parallel jobs (default: 8)
JOBS=4 just build

# Enable verbose logging
VERBOSE=1 just build
```

---

## Build Output Locations

| Artifact | Location | Purpose |
|----------|----------|---------|
| **Source** | `claude-session-tui/src/` | Rust source code |
| **Manifest** | `claude-session-tui/Cargo.toml` | Build configuration |
| **Binary** | `~/.cache/nabi/claude-manager/target/release/claude-session-tui` | Build output |
| **Installed** | `~/.local/bin/claude-session-tui` | PATH executable |
| **Data** | `~/.local/share/nabi/bin/claude-session-tui` | Registry copy |
| **Completion** | `~/.cache/zsh/completions/_claude-session-tui` | Zsh completion |
| **Config** | `~/.config/nabi/tools/claude-session-tui.toml` | Tool registry |

---

## Build Process Details

### Step 1: Configuration Generation
```bash
sed "s|\$XDG_CACHE_HOME|/Users/tryk/.cache|g" .cargo/config.toml.template > .cargo/config.toml
```

Substitutes environment variable into cargo config, directing build artifacts to XDG cache instead of repo root.

### Step 2: Cargo Build
```bash
cd claude-session-tui && cargo build --release
```

Compiles binary using the configured target directory and optimizations.

### Step 3: Completion Generation
```bash
printf '%s\n' ... > ~/.cache/zsh/completions/_claude-session-tui
```

Generates zsh completion file from template.

### Step 4: Binary Installation
```bash
cp ~/.cache/nabi/claude-manager/target/release/claude-session-tui ~/.local/bin/
cp ~/.cache/nabi/claude-manager/target/release/claude-session-tui ~/.local/share/nabi/bin/
```

Places binary in two locations:
- PATH location for direct discovery
- Data layer for federation registry

### Step 5: Verification
```bash
which claude-session-tui  # Check PATH discovery
claude-session-tui --help # Verify binary works
```

Confirms all pieces installed correctly.

---

## Performance

### Build Times
- **Clean build**: ~2-3 minutes (dependencies compile)
- **Incremental build**: ~5-30 seconds (depends on changes)
- **Quick-build**: ~5-30 seconds (cargo config cached)

### Install Time
- **Full install**: ~10 seconds (includes completions generation)
- **Quick install**: ~2 seconds (just copy binary)

### Watch Mode Overhead
- ~1 second per rebuild (cargo watches, recompiles, reinstalls)
- Negligible performance impact

---

## Troubleshooting

### Issue: "Binary not found" during install

**Cause**: Build didn't complete successfully.

**Solution**:
```bash
just clean
just build
```

### Issue: Completions not appearing in zsh

**Cause**: Completion file not in zsh `fpath`.

**Solution**:
```bash
# Check fpath includes the cache directory
echo $fpath

# If not, add to ~/.zshrc:
fpath=(~/.cache/zsh/completions $fpath)
autoload -U compinit && compinit
```

### Issue: "Device not configured" error

**Cause**: TUI binary terminal initialization failing (known issue).

**Status**: See `CLAUDE_SESSION_TUI_HANG_INVESTIGATION.md`

**Workaround**: Not applicable for build system; affects runtime only.

### Issue: Cargo config regeneration errors

**Cause**: Template file missing or malformed.

**Solution**:
```bash
ls -la .cargo/config.toml.template
# Should exist and contain XDG_CACHE_HOME placeholder
```

---

## Integration with Federation

### Schema-Driven Pattern
```
TOML (claude-session-tui.toml)
    ↓
Transform (nabi tools transform)
    ↓
JSON (~/.local/state/nabi/tools/registry.json)
```

The justfile follows this pattern by:
1. Installing binary to data layer
2. Creating TOML config automatically
3. Supporting `nabi exec claude-session-tui`

### Future Enhancements
- [ ] Automatic version sync from `Cargo.toml`
- [ ] Registry JSON updates
- [ ] Loki event emission on build
- [ ] Federated binary distribution

---

## Comparison with nabi-cli Pattern

| Aspect | Claude-Manager | nabi-cli Reference |
|--------|---|---|
| **Build Tool** | justfile ✓ | Makefile |
| **XDG Paths** | Full ✓ | Full |
| **Completion Gen** | Template-based | Clap binary |
| **Installation** | Dual location | PATH only |
| **Watch Mode** | Supported ✓ | Supported |
| **Release Process** | Defined ✓ | Defined |

---

## File Reference

| File | Purpose | Committed |
|------|---------|-----------|
| `justfile` | Build rules | ✓ |
| `.cargo/config.toml.template` | Cargo config template | ✓ |
| `.cargo/config.toml` | Generated (per build) | ✗ |
| `claude-session-tui/Cargo.toml` | TUI project manifest | ✓ |
| `.build/` | Intermediate files | ✗ |

---

## Summary

The Claude Manager build system provides:

✅ **XDG Compliance**: All artifacts stored in standard locations
✅ **Clean Install**: No modifications to repo root
✅ **Fast Iteration**: Watch mode for continuous development
✅ **Reproducibility**: Template-based configuration
✅ **Federation Ready**: Dual installation locations for registry
✅ **Simple Interface**: One-word targets like `just quick`, `just dev`

**Start development**: `just dev`
**Quick install**: `just quick`
**Verify setup**: `just verify`
