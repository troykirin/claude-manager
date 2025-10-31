# Multi-Phase Architecture Vision: From TUI to Federation

## 📐 Three-Phase Convergence Architecture

### Phase 1: Claude Manager + Session TUI ✅ (Completed)
**Location**: `~/nabia/tools/claude-manager/`
**Status**: Production-ready
**Scope**: Session browsing, searching, and path migration

```
claude-manager (CLI)
├── Session path migration (handles ~/old → ~/new transformations)
├── Backup/undo mechanisms
└── Federation event emission (Loki)
        ↓
claude-session-tui (Rust TUI)
├── Browse 1,300+ sessions (sorted newest first)
├── Fuzzy search with intent expansion
├── Pane-aware navigation (left: list, right: content)
├── Multiple view modes (Summary/JSON/Snippets)
└── Visual focus indicators + responsive scrolling
        ↓
[PHASE 2: riff-cli integration point]
```

### Phase 2: Riff CLI - Archive Search + DAG Analysis 🚧
**Location**: `~/nabia/tools/riff-cli/`
**Status**: Week 1 complete, Week 2+ in progress
**Scope**: Semantic search, conversation repair, DAG visualization

```
riff-cli (Python CLI)
├── Qdrant semantic search (384-dim vectors)
├── JSONL repair (scan/fix malformed files)
├── DAG analysis (conversation threads)
├── SurrealDB synchronization (immutable events)
├── Graph visualization (Mermaid/DOT format)
└── TUI module (vim-style navigation)
        ↓
Key Integration Point: Path Transformer
├── Extracts session UUID from .claude/projects/...
├── Resolves original project directory (via riff graph)
└── Generates resume command: cd /project && ccr <uuid>
        ↓
[PHASE 3: nabi-cli/nabi-tui integration point]
```

### Phase 3: Nabi CLI/TUI - Unified Federation Hub 📅
**Location**: `~/nabia/core/nabi-cli/` (or similar)
**Status**: Future (post Phase 2)
**Scope**: Unified command interface, federation coordination, DAG-TUI

```
nabi-cli (Rust router)
├── Tool registration & discovery
├── Path resolution & expansion
├── MCP tool integration
└── Federation coordination
        ↓
nabi-tui (DAG-based Terminal UI)
├── Conversation DAG visualization
├── Session browsing (integrated with claude-session-tui)
├── Semantic search (integrated with riff-cli)
├── Project navigation (integrated with path transformer)
└── Resume workflow: Press 'r' in session → Auto-resume in project dir
        ↓
Federation Message Bus (Loki + Memchain)
├── Session lifecycle events
├── Search/resume operation tracking
└── Cross-tool coordination
```

## 🔗 Resume Feature Integration Path

### Current State (Phase 1 Complete)
```
TUI Session Browser
    ↓ Press 'r' to resume
[NOT YET IMPLEMENTED]
```

### Phase 1.5 (Next Step - Minimal)
```
TUI Session Browser
    ↓ Press 'r' to resume
Extract from Session metadata:
├── UUID (from filename: session-<UUID>.jsonl)
├── File path (.claude/projects/-Users-tryk--nabi/...)
└── Created timestamp

Output interactive preview:
├── Session UUID: abc-123-def-456
├── Original path: ~/.claude/projects/-Users-tryk--nabia/
├── Command: cd ~/<project>/path && ccr abc-123-def-456
└── [COPY/EXECUTE/CANCEL]
```

### Phase 2 Integration (With riff-cli)
```
TUI Session Browser
    ↓ Press 'r' to resume
Call riff-cli path transformer:
├── riff.graph.loaders.JSONLLoader
├── Extract original project path from DAG
└── Resolve symlinks & transformations

Output intelligent preview:
├── Session UUID: abc-123-def-456
├── Found project: ~/nabia/tools/my-project
├── Full command: cd ~/nabia/tools/my-project && ccr abc-123-def-456
├── Project context:
│   ├── Last modified: 2025-10-29
│   ├── Files changed: 5
│   └── Status: Clean working tree
└── [COPY/EXECUTE/CANCEL]

Execute flow:
├── Verify project directory exists
├── Check working tree status (git)
├── Show summary of changes since session
└── cd to project & execute 'ccr' command
```

### Phase 3 Integration (With nabi-tui)
```
nabi-tui DAG Session Viewer
    ↓ Press 'r' to resume
Native integration with:
├── Path transformer (federation-aware)
├── Project context via nabi-cli
├── Federation message tracking
└── Session history/bookmarks

Advanced resume workflow:
├── Show project state at session creation
├── Diff: Then vs Now
├── Smart context restoration:
│   ├── Restore branch/workspace
│   ├── Load relevant files
│   └── Populate memory context
└── Launch claude-resume with full context
```

## 📊 Data Flow: Session UUID → Project Path → Resume

### Current Path Resolution (Phase 1)
```
.claude/projects/
  └─ -Users-tryk--nabia/
      └─ session-abc-123-def.jsonl
         ├─ Path stored in metadata
         ├─ UUID extracted from filename
         └─ But: Can't resolve where ~/nabia actually maps to
```

**Problem**: The `-Users-tryk--nabia` is an encoded path, not the real project directory.

### Solution: Use riff-cli Graph Module

**riff-cli/src/riff/graph/loaders.py** extracts:
```python
class JSONLLoader:
    def load_messages(self, session_id) -> list[Message]:
        # Scans .claude/projects/ for session files
        # Returns Message objects with all metadata
        # Key: Message.session_id + Message.metadata

    # Metadata contains hints about original project:
    # - File mentions (e.g., "src/main.rs")
    # - Error messages with paths
    # - User context from conversation
```

**riff-cli/src/riff/graph/analysis.py** performs:
```python
class ConversationDAG:
    def analyze_project_context(self, session):
        # Extract file paths from messages
        # Identify project structure
        # Resolve to actual ~/project/path
        # Return with confidence score
```

### Resume Command Assembly

**Option A: Simple Direct Resume** (Phase 1.5)
```bash
# Extract from session metadata
SESSION_UUID="abc-123-def-456"
cd ~/.claude/projects/-Users-tryk--nabia
# User must resolve path manually, OR:
cd ~/nabia/tools/my-project  # User copies from preview
ccr $SESSION_UUID
```

**Option B: Intelligent Resume** (Phase 2)
```bash
# riff-cli path transformer finds project
SESSION_UUID="abc-123-def-456"
PROJECT_PATH=$(riff-cli resolve-project $SESSION_UUID)
# OUTPUT: ~/nabia/tools/my-project
cd $PROJECT_PATH && ccr $SESSION_UUID
```

**Option C: Federation-Aware Resume** (Phase 3)
```bash
# nabi-cli coordinates across federation
SESSION_UUID="abc-123-def-456"
PROJECT_PATH=$(nabi resolve project-for-session $SESSION_UUID)
PROJECT_CONTEXT=$(nabi context load $SESSION_UUID)
cd $PROJECT_PATH && ccr $SESSION_UUID --context=$PROJECT_CONTEXT
```

## 🏗️ Architecture Diagram

```
PHASE 1: Session Discovery & Browsing
┌─────────────────────────────────────────────────┐
│ claude-session-tui (Rust)                       │
│ ├─ Browse sessions                              │
│ ├─ Search with fuzzy matching                   │
│ ├─ View modes: Summary/JSON/Snippets           │
│ └─ Pane-aware navigation                        │
└─────────────────┬───────────────────────────────┘
                  │ Session metadata
                  ↓
        Session UUID + Path
        [SESSION METADATA]
        ├─ Created: 2025-10-29
        ├─ File: -Users-tryk--nabia/session-abc-123.jsonl
        ├─ Blocks: 47
        └─ Status: Complete

                  │ User presses 'r'
                  ↓
┌─────────────────────────────────────────────────┐
│ RESUME FEATURE (Phase 1.5)                      │
│ ├─ Extract UUID: abc-123                        │
│ ├─ Show preview with command                    │
│ └─ Interactive: Copy/Execute/Cancel             │
└─────────────────┬───────────────────────────────┘
                  │ [FUTURE]
                  ↓
┌─────────────────────────────────────────────────┐
│ PHASE 2: riff-cli Integration                   │
│ ├─ Load session via JSONLLoader                 │
│ ├─ Analyze conversation DAG                     │
│ ├─ Extract project context from messages        │
│ └─ Resolve ~/nabia/tools/my-project            │
└─────────────────┬───────────────────────────────┘
                  │ Resolve command:
                  │ cd ~/nabia/tools/my-project
                  │ ccr abc-123
                  ↓
        Project Directory
        ├─ Files available
        ├─ Working tree status
        └─ Git branch context

                  │ [FUTURE PHASE 3]
                  ↓
┌─────────────────────────────────────────────────┐
│ PHASE 3: nabi-tui with Full Context            │
│ ├─ Show before/after diffs                      │
│ ├─ Memory context restoration                   │
│ └─ Smart workspace setup                        │
└─────────────────────────────────────────────────┘
```

## 🎯 Implementation Roadmap for Resume Feature

### Immediate (Phase 1.5) - Simple Interactive Resume
**Effort**: 2-3 hours
**Dependencies**: None (use current session metadata)
**Deliverable**: 'r' key in TUI shows preview + command

```rust
// In handle_key_event()
KeyCode::Char('r') => {
    if self.pane_focus == PaneFocus::Left {
        // Show resume preview modal
        self.show_resume_preview(selected_session);
    } else if self.is_searching {
        self.search_query.push('r');
    }
}

// New method
fn show_resume_preview(&mut self, session: &Session) {
    let uuid = extract_uuid_from_path(&session.metadata.file_path);
    let project_hint = guess_project_from_path(&session.metadata.file_path);

    // Display modal:
    // Session UUID: abc-123
    // Inferred project: ~/nabia/...
    // Command: cd <project> && ccr abc-123
    // [Copy] [Execute] [Cancel]
}
```

### Short-term (Phase 2) - riff-cli Integration
**Effort**: 4-6 hours
**Dependencies**: riff-cli Python module
**Deliverable**: Smart project resolution via DAG analysis

```python
# riff-cli new command
def resolve_project_for_session(session_uuid: str) -> Optional[Path]:
    """
    Find original project directory for a session using:
    1. Encoded path (-Users-tryk--nabia) decoding
    2. File path extraction from messages
    3. DAG analysis of project structure
    4. Git repo detection
    """
    loader = JSONLLoader(Path.home() / ".claude" / "projects")
    session = loader.load_session(session_uuid)

    # Extract project clues from messages
    analyzer = ConversationDAG(session)
    project_hint = analyzer.infer_project_context()

    # Try to resolve to actual path
    return resolve_path_hint(project_hint)
```

### Long-term (Phase 3) - Full nabi-tui Integration
**Effort**: 8-12 hours
**Dependencies**: nabi-cli federation integration
**Deliverable**: Unified resume with context restoration

```rust
// In nabi-tui
'r' in session viewer →
  ├─ Resolve project via federation
  ├─ Load session DAG from SurrealDB
  ├─ Show project status (git, files)
  ├─ Offer context restoration options
  └─ Execute smart resume
```

## 🔗 Key Integration Points

### Phase 1.5 → Phase 2 Bridge
```
claude-session-tui
    ↓ spawns subprocess
riff resolve-project <uuid>
    ↓ returns
~/nabia/tools/my-project
    ↓ used in
cd ~/nabia/tools/my-project && ccr <uuid>
```

### Phase 2 → Phase 3 Bridge
```
riff-cli search results
    ↓ exposed via MCP
nabi-cli tool registry
    ↓ used by
nabi-tui (integrated browsing)
    ↓ coordinates with
claude-session-tui (seamless hand-off)
```

## 📋 Summary: The Complete Picture

**Phase 1** (NOW): Browse and search sessions
**Phase 1.5** (NEXT): Simple resume with preview
**Phase 2** (SOON): Intelligent project resolution
**Phase 3** (FUTURE): Unified federation TUI with full context

The **resume feature** elegantly ties all three phases together:
- Session discovery (Phase 1)
- Project resolution via DAG (Phase 2)
- Unified experience (Phase 3)

Starting with Phase 1.5 requires minimal work but sets up the foundation for sophisticated Phase 2/3 integration!
