# Resume Feature Implementation Roadmap

## 🎯 Feature Overview

**User Journey**:
```
1. Open claude-session-tui
2. Browse 1,300+ sessions (sorted newest first)
3. Find session you want to continue
4. Press 'r' to resume
5. See preview with command ready to execute
6. Copy or execute to cd into project + run 'ccr'
```

## 📋 Implementation Stages

### Stage 1: Simple Resume (Phase 1.5) - 2-3 Hours

**What It Does**:
- Extract session UUID from filename
- Guess project directory from stored path
- Show interactive preview modal
- Allow user to copy command or execute directly

**Key Code Changes**:
1. Add 'r' key handler in `handle_key_event()`
2. Create `ResumeModal` component
3. Implement `extract_uuid_from_path()`
4. Implement `guess_project_from_path()`

**Files to Modify**:
- `src/ui/app.rs` - Add resume handler
- `src/ui/mod.rs` - Add modal components

**Example Preview Modal**:
```
╔═══════════════════════════════════════╗
║         Resume Session                ║
╠═══════════════════════════════════════╣
║                                       ║
║  Session UUID:                        ║
║  abc-123-def-456-ghi-789             ║
║                                       ║
║  Created: 2025-10-29 14:32:15 UTC    ║
║  Blocks: 47 messages                  ║
║                                       ║
║  Inferred Project:                    ║
║  ~/nabia/tools/my-project            ║
║                                       ║
║  Command to execute:                  ║
║  ┌───────────────────────────────────┐ ║
║  │cd ~/nabia/tools/my-project        │ ║
║  │ccr abc-123-def-456-ghi-789        │ ║
║  └───────────────────────────────────┘ ║
║                                       ║
║  [c]opy  [e]xecute  [ESC] cancel     ║
╚═══════════════════════════════════════╝
```

### Stage 2: riff-cli Integration (Phase 2) - 4-6 Hours

**What It Does**:
- Call riff-cli to analyze session DAG
- Extract project context from messages
- Intelligently resolve real project directory
- Show git status and file changes

**Key Integration**:
```rust
// Call riff-cli as subprocess
fn resolve_project_with_riff(session_uuid: &str) -> Result<PathBuf> {
    let output = Command::new("riff")
        .args(&["resolve-project", session_uuid])
        .output()?;

    let project_path = String::from_utf8(output.stdout)?
        .trim()
        .parse::<PathBuf>()?;

    Ok(project_path)
}
```

**Files to Modify**:
- `src/ui/app.rs` - Enhance resume handler
- `src/ui/mod.rs` - Enhanced modal with git status
- Add riff-cli dependency to Cargo.toml

**Enhanced Modal**:
```
╔═══════════════════════════════════════╗
║         Resume Session                ║
╠═══════════════════════════════════════╣
║                                       ║
║  Session: session-abc-123-def (47 msg) ║
║  Created: 2025-10-29 14:32:15 UTC    ║
║                                       ║
║  📁 Project: ~/nabia/tools/my-project ║
║                                       ║
║  Git Status:                          ║
║  ├─ Branch: feature/awesome          ║
║  ├─ Status: 3 modified, 1 new        ║
║  └─ Commits: 2 ahead of origin/main  ║
║                                       ║
║  Command:                             ║
║  ┌───────────────────────────────────┐ ║
║  │cd ~/nabia/tools/my-project        │ ║
║  │ccr abc-123-def-456                │ ║
║  └───────────────────────────────────┘ ║
║                                       ║
║  [c]opy  [e]xecute  [ESC] cancel     ║
╚═══════════════════════════════════════╝
```

### Stage 3: Full Federation Integration (Phase 3) - 8-12 Hours

**What It Does**:
- Load session from SurrealDB via riff-cli
- Show conversation DAG visualization
- Restore project context
- Offer smart branch/workspace setup

**Files to Modify**:
- Multiple integration points with nabi-cli
- Enhanced TUI rendering for DAG view

## 🛠️ Technical Details

### UUID Extraction
```rust
fn extract_uuid_from_path(file_path: &str) -> Option<String> {
    // Input: "session-abc-123-def-456.jsonl"
    // Output: "abc-123-def-456"

    let filename = Path::new(file_path)
        .file_stem()?
        .to_str()?;

    if let Some(uuid) = filename.strip_prefix("session-") {
        Some(uuid.to_string())
    } else {
        None
    }
}
```

### Project Path Guessing
```rust
fn guess_project_from_path(stored_path: &str) -> Option<String> {
    // Decode: "-Users-tryk--nabia" → "nabia"
    // Find matching ~/nabia/tools/* or ~/nabia/*

    let encoded = Path::new(stored_path)
        .parent()?
        .file_name()?
        .to_str()?;

    // Decode: "-Users-tryk--" prefix removal
    let project_hint = encoded
        .trim_start_matches("-Users-tryk--");

    // Search common locations
    for search_path in &[
        PathBuf::from("~/nabia"),
        PathBuf::from("~/nabia/tools"),
    ] {
        for entry in std::fs::read_dir(search_path).ok()? {
            let entry = entry.ok()?;
            if entry.file_name().to_string_lossy().contains(project_hint) {
                return Some(entry.path().to_string_lossy().to_string());
            }
        }
    }

    None
}
```

### Modal Rendering
```rust
fn render_resume_modal(&self, frame: &mut Frame, area: Rect) {
    // Create centered popup
    let modal_height = 20;
    let modal_width = 45;

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min((area.height - modal_height) / 2),
            Constraint::Length(modal_height),
            Constraint::Min((area.height - modal_height) / 2),
        ]);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min((area.width - modal_width) / 2),
            Constraint::Length(modal_width),
            Constraint::Min((area.width - modal_width) / 2),
        ]);

    let modal_area = vertical.split(area)[1];
    let modal_area = horizontal.split(modal_area)[1];

    // Render modal content...
}
```

## 📝 Key Decisions

### Why 'r' for resume?
- Mnemonic and intuitive
- Doesn't conflict with search or navigation
- Vim-style consistency (others like 'j/k' for nav)

### Why interactive modal vs auto-execute?
- Safety: Review command before executing
- Flexibility: Copy command to different context
- Learning: Users understand what 'ccr' does

### Why support both copy and execute?
- **Copy**: Users can review in shell first
- **Execute**: Frictionless for trusted sessions

## 🧪 Testing Strategy

### Unit Tests
```rust
#[test]
fn test_extract_uuid_from_session_path() {
    let path = "/Users/.../projects/dir/session-abc-123-def.jsonl";
    assert_eq!(extract_uuid_from_path(path), Some("abc-123-def".to_string()));
}

#[test]
fn test_project_path_guessing() {
    let stored_path = "~/.../nabia/session-abc.jsonl";
    let guessed = guess_project_from_path(stored_path);
    assert!(guessed.contains("nabia"));
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_resume_workflow() {
    // 1. Load TUI
    // 2. Select session
    // 3. Press 'r'
    // 4. Verify modal appears
    // 5. Copy command
    // 6. Verify clipboard contains command
}
```

### Manual Testing
1. Open TUI with `claude-session-tui --since 7d`
2. Select a session with 'j/k'
3. Press 'r'
4. Verify modal shows correct UUID
5. Copy command with 'c'
6. Paste in shell: `ccr <uuid>`
7. Verify Claude resumes session

## 📊 Dependency Tree

```
Stage 1 (Simple)
└─ No external dependencies
   ├─ Requires: UUID extraction + path guessing
   └─ Time: 2-3 hours

Stage 2 (riff-cli)
└─ Requires: riff-cli subprocess calls
   ├─ Requires: Stage 1 complete
   ├─ Dependency: riff-cli in PATH
   └─ Time: 4-6 hours

Stage 3 (Full Federation)
└─ Requires: nabi-cli + SurrealDB
   ├─ Requires: Stage 1 + 2 complete
   ├─ Dependency: nabi-cli integration
   └─ Time: 8-12 hours
```

## ✅ Success Criteria

### Stage 1 Success
- [ ] Press 'r' shows modal
- [ ] UUID correctly extracted
- [ ] Project path appears (guessed)
- [ ] Command shown in preview
- [ ] Copy button works (command in clipboard)
- [ ] Cancel closes modal without side effects

### Stage 2 Success
- [ ] riff-cli called successfully
- [ ] Project path resolved accurately
- [ ] Git status shown in modal
- [ ] Execute button works (spawns shell)
- [ ] Works for 95%+ of sessions (robust guessing)

### Stage 3 Success
- [ ] SurrealDB data loaded
- [ ] DAG visualization renders
- [ ] Context restoration works
- [ ] Smart branch/workspace setup assists
- [ ] Seamless hand-off to ccr

## 🎯 Estimated Timeline

- **This week**: Stage 1 (simple resume with preview)
- **Next week**: Stage 2 (riff-cli integration)
- **Phase 3 prep**: Architecture for federation integration

## 📚 References

- Phase Architecture: `PHASE_ARCHITECTURE_VISION.md`
- Riff-cli Docs: `~/nabia/tools/riff-cli/docs/START_HERE.md`
- Current TUI Code: `src/ui/app.rs`
- Session Model: Claude-session-tui lib.rs

---

**Next Step**: Implement Stage 1 (Simple Resume)
