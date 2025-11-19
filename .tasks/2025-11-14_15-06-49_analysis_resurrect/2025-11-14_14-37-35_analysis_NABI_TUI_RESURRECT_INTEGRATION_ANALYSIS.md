# Nabi-TUI Session Picker Integration Analysis
## tmux-resurrect-fireproof Data Integration Study

**Analysis Date**: 2025-11-14  
**Subject**: Claude-Session-TUI ↔ tmux-resurrect-fireproof Integration  
**Scope**: Integrating persistent tmux session data into the nabi-tui session picker

---

## 1. CODEBASE LOCATION & ARCHITECTURE

### Session Picker Location
**Primary File**: `/Users/tryk/nabia/tools/claude-manager/claude-session-tui/src/ui/app.rs`
- **Lines of Code**: 1,927 lines
- **Purpose**: Main TUI application logic with session list, search, and view modes
- **Language**: Rust (ratatui framework)

### Supporting Files in TUI Stack
```
/Users/tryk/nabia/tools/claude-manager/claude-session-tui/src/
├── main.rs (231 lines) - Entry point, event loop, async runtime
├── ui/
│   ├── app.rs (1927 lines) ⭐ PRIMARY TARGET
│   ├── session_tree.rs (527 lines) - Alternative tree view component
│   └── mod.rs (6 lines) - Module exports
├── models.rs (17,416 bytes) - Data structures
├── parser.rs (48,374 bytes) - Session file parsing
├── extractor.rs (24,382 bytes) - Feature extraction
├── api.rs (28,025 bytes) - Public API
├── lib.rs (2,112 bytes) - Library root
└── Cargo.toml - Dependencies
```

### tmux-resurrect-fireproof Location
**Primary Script**: `/Users/tryk/nabia/tools/tmux-resurrect-fireproof/tmux-resurrect-fireproof.sh`
**Data Source**: `~/.local/share/tmux/resurrect/` (26KB backup files, tab-delimited format)

---

## 2. CURRENT DATA SOURCE & LOADING MECHANISM

### Where Session Data Currently Comes From

**Data Source**: Claude Code session files (JSONL format)
```bash
~/.claude/projects/-Users-tryk--nabia/session-abc123.jsonl
~/.claude/projects/-Users-tryk--config-nabi/session-def456.jsonl
```

**Loading Flow**:
```rust
App::new()
  ↓
load_sessions(directory: PathBuf)
  ↓
parse_session_directory(dir) [from parser.rs]
  ↓
Vec<Session> (sorted by file mtime, most recent first)
  ↓
filtered_sessions: Vec<Session> (for search/display)
```

### Current Data Binding
- **Architecture Pattern**: Direct file parsing + in-memory filtering
- **Data Source Layer**: None (directly loads from filesystem)
- **Refresh Mechanism**: One-time load at startup (see line 89-110 in app.rs)
- **External Integration**: None currently

### Key Data Loading Functions

**Lines 89-110** - `App::load_sessions()`:
```rust
pub async fn load_sessions(&mut self, directory: PathBuf) -> anyhow::Result<()> {
    match parse_session_directory(directory).await {
        Ok(mut sessions) => {
            // Sort by mtime in reverse (most recent first)
            sessions.sort_by(|a, b| {
                let a_mtime = std::fs::metadata(&a.metadata.file_path)
                    .and_then(|m| m.modified())
                    .unwrap_or(a.metadata.created_at.into());
                let b_mtime = std::fs::metadata(&b.metadata.file_path)
                    .and_then(|m| m.modified())
                    .unwrap_or(b.metadata.created_at.into());
                b_mtime.cmp(&a_mtime) // Reverse: most recent first
            });
            self.selected = 0;
            self.sessions = sessions;
            self.filtered_sessions = self.sessions.clone();
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}
```

**Lines 112-135** - `App::load_sessions_from_files()`:
- Alternative loading path for pre-filtered files
- Same sorting and initialization logic

---

## 3. RENDERING & PRESENTATION

### Session Display Format (Lines 211-248)

Current rendering shows:
```
[Index]  [Project Path]  [UUID]
  1      ~/nabia/tools/project    [abc-1234]
  2      ~/config/nabi            [def-5678]
```

**Components Used**:
- **View Mode**: Summary (lines 307-379), FullJson (381-436), SnippetBrowser (438-442)
- **Left Pane**: 35% width, fuzzy-searchable list of sessions
- **Right Pane**: 65% width, details (status, JSON, or snippets)

### Search & Filter (Lines 1507-1627)

Current implementation:
- Fuzzy matching using `SkimMatcherV2`
- Intent expansion with domain-specific keywords
- Scoring mechanism for relevance
- Result limit: 50 sessions, 200 matches

---

## 4. TMUX RESURRECT DATA FORMAT

### Resurrect File Structure
**Location**: `~/.local/share/tmux/resurrect/tmux_resurrect_TIMESTAMP.txt`  
**Format**: Tab-delimited (one line per pane)  
**Size**: ~24KB per backup

**Data Format** (each line is a pane):
```
pane	SESSION_NAME	WINDOW_INDEX	PANE_INDEX	WINDOW_NAME	ACTIVE_PANE	PWD_DISPLAY	PWD_FULL	PANE_NUM	SHELL	SHELL_COMMAND
pane	36	1	1	:*	1	~	:/Users/tryk	1	zsh	:-zsh
pane	agent-beru	1	1	:*	1	path	:/Users/tryk/leGen	1	bash	:sleep 30
```

**Extractable Data**:
- `SESSION_NAME`: Persistent tmux session identifier
- `PWD_FULL`: Full working directory path
- `PWD_DISPLAY`: Human-readable path (e.g., `~`)
- `SHELL`: Shell type (zsh, bash, fish, etc.)
- `SHELL_COMMAND`: Current command/state
- `WINDOW_NAME`: Tmux window identifier

**Backups Available**:
- Latest: `/Users/tryk/.local/share/tmux/resurrect/tmux_resurrect_20251020T133836.txt`
- Historical: Multiple timestamped versions retained

---

## 5. DATA ARCHITECTURE ANALYSIS

### Current Monolithic Design

```
App Struct (app.rs, line 45)
├── sessions: Vec<Session> ────────────────── From JSONL files
├── filtered_sessions: Vec<Session> ────────── Search results
├── search_matches: Vec<SearchMatch> ──────── Match details
├── selected: usize ────────────────────────── Selection index
├── search_query: String ──────────────────── Search state
└── view_mode: ViewMode ──────────────────── UI state
```

### No Service Layer

The app directly:
1. Calls `parse_session_directory()` (parser.rs)
2. Sorts results in-memory
3. Stores all data as `Vec<Session>`
4. Filters on-demand

**Problem**: No abstraction for external data sources

---

## 6. SESSION PICKER COMPONENT ANALYSIS

### Primary Component: `App` struct (app.rs:45)

**State Fields**:
```rust
pub struct App {
    sessions: Vec<Session>,                    // All loaded sessions
    filtered_sessions: Vec<Session>,           // Filtered search results
    search_matches: Vec<SearchMatch>,          // Match details
    selected: usize,                          // Currently selected index
    snippet_index: usize,                     // Snippet navigation
    snippet_scroll_offset: usize,
    json_scroll_offset: usize,
    search_query: String,
    is_searching: bool,
    view_mode: ViewMode,
    pane_focus: PaneFocus,
    error_message: Option<String>,
    should_quit: bool,
    show_resume_modal: bool,
    resume_session_uuid: Option<String>,
    resume_project_path: Option<String>,
}
```

### Alternative: `SessionTree` component (session_tree.rs:25-46)

Less mature component with tree view support:
```rust
pub struct SessionTree {
    sessions: Vec<Session>,
    filtered_sessions: Vec<SessionEntry>,    // Richer metadata
    projects: HashMap<String, Vec<usize>>,   // Project organization
    expanded_projects: HashSet<String>,
    view_mode: ViewMode,                      // Flat vs Tree
    ...
}
```

---

## 7. INTEGRATION DIFFICULTY ASSESSMENT

### Difficulty Level: **MEDIUM** (6/10)

#### Why Not Easy (1-3):
- Requires new data source abstraction
- Need to parse new file format (tab-delimited vs JSON)
- Must handle data synchronization
- Both sessions (JSONL) and resurrect (text) data shown together

#### Why Not Hard (7-10):
- No changes needed to UI rendering (just source data)
- Small file size (~24KB) means simple parsing
- Rust already has strong parsing libraries
- Existing search/filter logic can remain unchanged

---

## 8. FILES REQUIRING CHANGES

### Essential Changes (3 files)

**1. `models.rs` (17KB)** - Add resurrect data structures
```rust
// NEW struct to represent tmux session
pub struct TmuxSession {
    pub session_name: String,
    pub window_count: usize,
    pub pane_count: usize,
    pub working_dirs: Vec<String>,
    pub active_window: Option<u16>,
    pub last_activity: DateTime<Utc>,
}

// EXTEND Session to optionally include resurrect data
pub struct Session {
    // ... existing fields ...
    pub tmux_session: Option<TmuxSession>,  // NEW
    pub resurrection_backup: Option<ResurrectionMetadata>,  // NEW
}
```

**2. `parser.rs` (48KB)** - Add resurrect file parser
```rust
// NEW function to parse resurrect backup
pub async fn parse_resurrect_backup(path: &Path) -> Result<Vec<TmuxSession>> {
    // Parse tab-delimited format
    // Extract session names, paths, counts
    // Return structured data
}

// NEW function to correlate with Claude sessions
pub fn merge_resurrect_data(
    sessions: Vec<Session>,
    resurrect: Vec<TmuxSession>
) -> Vec<Session> {
    // Link Claude sessions to tmux sessions by matching paths
    // Enrich Session with tmux context
}
```

**3. `ui/app.rs` (1,927 lines)** - Minor changes to display
```rust
// EXTEND rendering (line 211-248) to show tmux status
// IF session has tmux data:
//   ├─ Show session badge
//   ├─ Show window/pane count
//   └─ Show working directories

// EXTEND search to include tmux session names
// EXTEND session selection to include context about tmux state
```

### Nice-to-Have Changes (2 files)

**4. `lib.rs` (2KB)** - Export new functions

**5. `main.rs` (231 lines)** - Optional: Add command-line flag
```rust
// --with-resurrect flag to enable tmux session integration
// Load resurrect backup if available
```

---

## 9. EXISTING DATA INTEGRATION PATTERNS

### How to Extend the Architecture

**Current Pattern** (single source):
```
load_sessions_from_files(Vec<PathBuf>)
  → parse_session_files()
  → Vec<Session>
  → filter → render
```

**Proposed Pattern** (multi-source):
```
load_sessions_from_files(Vec<PathBuf>)
  → parse_session_files()
  → Vec<Session>
  
load_resurrect_data()
  → parse_resurrect_backup()
  → Vec<TmuxSession>
  
merge_data(sessions, resurrect)
  → Vec<Session with .tmux_session: Option>
  
filter → render (unchanged logic)
```

### Reference: `Session` Enrichment Pattern

Already used in models.rs for similar enrichment:
- `Session.insights: SessionInsights` (line 15)
- `Session.statistics: SessionStatistics` (line 16)
- `Session.tool_usage: ToolUsageStats` (line 17)
- `Session.working_context: WorkingContext` (line 18)

**Proposed Addition**:
- `Session.tmux_session: Option<TmuxSession>` (new)
- `Session.resurrection_metadata: Option<ResurrectionMetadata>` (new)

---

## 10. ESTIMATED IMPLEMENTATION EFFORT

### Code Changes Summary

| File | Changes | Est. Lines | Difficulty |
|------|---------|-----------|------------|
| models.rs | Add 2 new structs | 50-75 | Easy |
| parser.rs | Add resurrect parser + merge fn | 150-200 | Medium |
| ui/app.rs | Extend rendering (3 locations) | 40-60 | Easy |
| main.rs | Add optional loading | 15-25 | Easy |
| Cargo.toml | No changes needed | 0 | N/A |
| **TOTAL** | | **255-360** | **Medium** |

### Implementation Timeline

- **Phase 1** (2-3 hours): Data structures + parser
  - Add models.rs structs
  - Write parser.rs resurrect file reader
  - Test with real resurrect file

- **Phase 2** (1-2 hours): Integration
  - Add merge logic
  - Update App::load_sessions() to load resurrect data
  - Handle missing resurrect files gracefully

- **Phase 3** (1-2 hours): UI Display
  - Extend render() to show tmux context
  - Add badges/indicators for tmux sessions
  - Update search to include tmux session names

- **Phase 4** (30-60 min): Polish
  - Add command-line flags
  - Update documentation
  - Write tests

**Total**: 5-8 hours estimated development time

---

## 11. COMPLEXITY FACTORS

### Simple Aspects
✅ Resurrect file is small and simple to parse (tab-delimited)  
✅ Parsing logic is straightforward (regex + split)  
✅ No changes to existing UI layout needed  
✅ Search/filter logic remains unchanged  
✅ Can be made optional (graceful degradation)  

### Complex Aspects
⚠️ Data correlation (matching Claude sessions to tmux sessions)  
⚠️ Handling timing skew (resurrect backups may not align with Claude sessions)  
⚠️ Version compatibility (resurrect format may evolve)  
⚠️ State synchronization (keeping tmux state current)  

---

## 12. BREAKDOWN: CHANGES PER COMPONENT

### `models.rs` - Add 50-75 lines

**New Structs**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxSession {
    pub session_name: String,
    pub window_count: usize,
    pub pane_count: usize,
    pub working_directories: Vec<String>,
    pub active_window_index: Option<u16>,
    pub primary_shell: String,
    pub backup_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResurrectionMetadata {
    pub backup_file: String,
    pub backup_timestamp: DateTime<Utc>,
    pub resurrect_version: String,
    pub checksum: Option<String>,
}
```

**Extend Session**:
```rust
pub struct Session {
    // ... existing 18 fields ...
    pub tmux_context: Option<TmuxSession>,
    pub resurrection_metadata: Option<ResurrectionMetadata>,
}
```

---

### `parser.rs` - Add 150-200 lines

**New Functions**:
```rust
pub async fn parse_resurrect_backup(path: &Path) -> Result<Vec<TmuxSession>> {
    // Read file
    // Split by \n
    // Filter lines starting with "pane"
    // Parse tab-delimited fields
    // Group by session name
    // Return Vec<TmuxSession>
}

pub fn correlate_sessions_with_resurrect(
    sessions: Vec<Session>,
    resurrect_sessions: Vec<TmuxSession>,
) -> Vec<Session> {
    // Match by working directory path
    // Match by partial path matching
    // Attach TmuxSession to enriched Session
    // Return enriched Vec<Session>
}
```

**Rough Implementation**:
```rust
fn parse_resurrect_line(line: &str) -> Option<(String, PaneInfo)> {
    let fields: Vec<&str> = line.split('\t').collect();
    // fields[1] = session_name
    // fields[6] = pwd_display
    // fields[7] = pwd_full
    // fields[8] = pane_num
    // fields[9] = shell
    // fields[10] = shell_command
}
```

---

### `ui/app.rs` - Add 40-60 lines

**Changes at Three Locations**:

1. **Lines 211-248** (Session list rendering):
   ```rust
   // Add tmux badge if session has resurrect data
   if s.tmux_context.is_some() {
       label = format!("{} 🔄 {} windows", label, window_count);
   }
   ```

2. **Lines 322-370** (Status pane):
   ```rust
   // Show tmux status
   if let Some(tmux) = &selected_session.tmux_context {
       right_content.push(Line::from(format!(
           "Tmux Sessions: {} windows, {} panes",
           tmux.window_count, tmux.pane_count
       )));
   }
   ```

3. **Lines 1539-1559** (Search expansion):
   ```rust
   // Include tmux session names in search
   let search_text = format!(
       "{} {} {} {}",
       session.metadata.file_path,
       session.tmux_context
           .as_ref()
           .map(|t| &t.session_name)
           .unwrap_or(&String::new()),
       // ... rest of search text
   );
   ```

---

### `main.rs` - Add 15-25 lines

**Optional Command-Line Integration**:
```rust
#[derive(Parser)]
struct Args {
    #[arg(long, help = "Include tmux resurrect session data")]
    with_resurrect: bool,
    
    #[arg(long, help = "Path to resurrect backup file")]
    resurrect_path: Option<PathBuf>,
}

// In main():
if args.with_resurrect {
    let resurrect_data = parse_resurrect_backup(
        &args.resurrect_path.unwrap_or_default()
    ).await?;
    app.sessions = correlate_sessions_with_resurrect(
        app.sessions,
        resurrect_data
    );
}
```

---

## 13. DATA ADAPTER PATTERN RECOMMENDATION

### Proposed Architecture (Future-Proof)

Instead of direct integration, create a **Data Provider trait**:

```rust
// In a new file: src/data/mod.rs

#[async_trait]
pub trait SessionDataProvider {
    async fn load_sessions(&self) -> Result<Vec<Session>>;
}

pub struct ClaudeSessionProvider {
    search_dir: PathBuf,
}

pub struct TmuxResurrectProvider {
    resurrect_path: PathBuf,
}

#[async_trait]
impl SessionDataProvider for TmuxResurrectProvider {
    async fn load_sessions(&self) -> Result<Vec<Session>> {
        // Load and parse resurrect backup
    }
}

pub struct CompositeProvider {
    providers: Vec<Box<dyn SessionDataProvider>>,
}

#[async_trait]
impl SessionDataProvider for CompositeProvider {
    async fn load_sessions(&self) -> Result<Vec<Session>> {
        // Load from all providers
        // Merge and deduplicate
        // Return combined results
    }
}
```

**Usage**:
```rust
let provider = CompositeProvider::new()
    .add(ClaudeSessionProvider::new(claude_dir))
    .add(TmuxResurrectProvider::new(resurrect_path));

app.sessions = provider.load_sessions().await?;
```

**Benefits**:
- ✅ Easy to add more data sources (Zellij, screens, etc.)
- ✅ Testable (mock providers)
- ✅ Decoupled from App struct
- ✅ Follows Open/Closed Principle

---

## 14. RISK ASSESSMENT

### Implementation Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Resurrect file format changes | Medium | High | Version detection, graceful fallback |
| Path correlation mismatch | High | Low | Fuzzy matching, manual override |
| Performance regression | Low | Medium | Benchmark parsing, lazy loading |
| Breaking changes to models.rs | Low | High | Semantic versioning, migration guide |

### Testing Strategy

**Unit Tests** (new in tests/):
```rust
#[test]
fn test_parse_resurrect_line() { /* ... */ }

#[test]
fn test_correlate_sessions() { /* ... */ }

#[test]
fn test_merge_data_idempotent() { /* ... */ }
```

**Integration Tests**:
- Load real Claude sessions + resurrect backup
- Verify correlation accuracy
- Check performance with 100+ sessions

---

## 15. SUMMARY & RECOMMENDATIONS

### Executive Summary

**Integration is FEASIBLE and MEDIUM difficulty**

The nabi-tui session picker can be enhanced to show tmux resurrect session context by:

1. **Adding data structures** to models.rs (50-75 lines)
2. **Writing a parser** in parser.rs (150-200 lines)
3. **Enhancing rendering** in ui/app.rs (40-60 lines)
4. **Optional CLI support** in main.rs (15-25 lines)

**Total investment**: 255-360 lines of code, 5-8 hours development time

### Recommended Approach

**Phase 1 (Immediate)**: Monolithic integration
- Parse resurrect file at startup
- Merge with existing sessions
- Show badges in UI

**Phase 2 (Future)**: Refactor to provider pattern
- Extract data loading logic
- Support multiple sources (Zellij, screens, etc.)
- Enable pluggable data sources

### Key Files to Modify

| Priority | File | Change Type |
|----------|------|-------------|
| P1 | models.rs | Add 2 structs |
| P1 | parser.rs | Add parser function |
| P1 | ui/app.rs | Extend rendering (3 locations) |
| P2 | main.rs | Add CLI support |
| P3 | Cargo.toml | None needed |

### Next Steps

1. ✅ **This analysis** - Understand architecture
2. **Prototype parser** - Test resurrect file parsing (1 hour)
3. **Add models** - Define data structures (30 min)
4. **Implement merge** - Link Claude ↔ tmux sessions (1 hour)
5. **Update UI** - Show resurrect context (1 hour)
6. **Test thoroughly** - Edge cases, performance (2 hours)
7. **Document** - User guide, developer notes (1 hour)

---

## 16. APPENDIX: CODE LOCATION MAP

### Quick Reference

**Session Picker Entry Point**:
- `/Users/tryk/nabia/tools/claude-manager/claude-session-tui/src/ui/app.rs:45`
- `struct App { ... }`

**Session Loading**:
- `/Users/tryk/nabia/tools/claude-manager/claude-session-tui/src/ui/app.rs:89-110`
- `pub async fn load_sessions()`

**Session Display**:
- `/Users/tryk/nabia/tools/claude-manager/claude-session-tui/src/ui/app.rs:211-248`
- Session list rendering loop

**Search Implementation**:
- `/Users/tryk/nabia/tools/claude-manager/claude-session-tui/src/ui/app.rs:1507-1627`
- `pub fn search_sessions()`

**Data Models**:
- `/Users/tryk/nabia/tools/claude-manager/claude-session-tui/src/models.rs:1-17416`
- `pub struct Session { ... }`

**File Parsing**:
- `/Users/tryk/nabia/tools/claude-manager/claude-session-tui/src/parser.rs:1-48374`
- `pub async fn parse_session_directory()`

**Resurrect Source**:
- `/Users/tryk/.local/share/tmux/resurrect/tmux_resurrect_*.txt`
- Tab-delimited pane/window/session data

---

**End of Analysis**
