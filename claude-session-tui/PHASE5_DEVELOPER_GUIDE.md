# Phase 5: Developer Quick Reference Guide

## Search Integration API

### Data Structures

```rust
// models.rs
pub struct Session {
    // ... existing fields
    pub resurrection: Option<ResurrectionMetadata>,
}

pub struct ResurrectionMetadata {
    pub tmux: Option<TmuxMetadata>,
    pub path_match_confidence: f64,
    pub has_tmux_history: bool,
}

pub struct TmuxMetadata {
    pub session_name: Option<String>,
    pub shell_command: Option<String>,
    pub working_directory: Option<String>,
}
```

### Match Sources

```rust
// ui/app.rs
pub enum MatchSource {
    Claude,           // Match from Claude session content
    TmuxSession,      // Match from tmux session name
    TmuxCommand,      // Match from shell command
    TmuxDirectory,    // Match from working directory
}

pub struct SearchMatch {
    pub session_index: usize,
    pub block_index: usize,
    pub score: i64,
    pub snippet: String,
    pub full_json: String,
    pub source: MatchSource,  // NEW in Phase 5
}
```

## How to Populate Resurrection Data

When loading sessions (from Phase 1-3):

```rust
// Example: Setting tmux metadata
session.resurrection = Some(ResurrectionMetadata {
    tmux: Some(TmuxMetadata {
        session_name: Some("my-dev-session".to_string()),
        shell_command: Some("bash -l".to_string()),
        working_directory: Some("/Users/tryk/project".to_string()),
    }),
    path_match_confidence: 0.95,
    has_tmux_history: true,
});
```

## Search Scoring Algorithm

```
Claude session name match:    Base 1000, fuzzy boosted up to 2000+
Tmux session name match:      Base 80, fuzzy boosted up to 100
Shell command match:          Fixed 60 (exact substring)
Working directory match:      Base 40, fuzzy boosted up to 50

Result ordering: Descending by score (highest first)
```

## Testing Checklist

### Unit Testing
- [ ] Test search with no resurrection data (Claude-only)
- [ ] Test search with partial tmux metadata (some fields None)
- [ ] Test search with complete tmux metadata
- [ ] Test fuzzy matching accuracy for session names
- [ ] Test fuzzy matching accuracy for directories
- [ ] Test exact matching for shell commands

### Integration Testing
- [ ] Verify source indicators display correctly
- [ ] Verify result ordering by score
- [ ] Verify snippet formatting for tmux matches
- [ ] Verify search with multiple match types per session

### Performance Testing
- [ ] Measure search time with 100 sessions
- [ ] Measure search time with 500 sessions
- [ ] Verify <500ms requirement for 100 sessions
- [ ] Profile memory usage with resurrection data

## UI Display Format

```
Snippet Browser View:
┌────────────────────────────────────────────────┐
│ Match 1/5 (Score: 85) - Session 3 Block 1 (from tmux session)
│ ────────────────────────────────────────────────
│ Tmux session: schema-driven-dev
│
│ | n/p navigate, v view full, q exit
└────────────────────────────────────────────────┘
```

## Common Integration Patterns

### Pattern 1: Claude + Tmux Session Match
```
User searches: "schema"

Results:
1. Claude session with "schema-driven" in content (Score: 1500)
2. Claude session with tmux session "my-schema-work" (Score: 90)
3. Claude session with dir "/project/schema/" (Score: 45)
```

### Pattern 2: Tmux-Only Match
```
User searches: "bash"

Results:
1. Claude session with tmux session "bash-dev" (Score: 95)
2. Claude session with command "bash -l" (Score: 60)
```

### Pattern 3: Directory Path Match
```
User searches: "/Users/tryk/nabia"

Results:
1. Claude session with project path match (Score: 1000+)
2. Claude session with tmux dir "/Users/tryk/nabia/core" (Score: 48)
```

## Debugging Tips

### Enable Debug Logging
```rust
// Add to search_sessions()
eprintln!("Searching session {}: has_resurrection={}",
    session_idx,
    session.resurrection.is_some()
);
```

### Check Match Sources
```rust
// After search_sessions()
for m in &self.search_matches {
    eprintln!("Match: score={}, source={:?}", m.score, m.source);
}
```

### Verify Fuzzy Scores
```rust
// In tmux session search
if let Some(fuzzy_score) = matcher.fuzzy_match(session_name, query) {
    eprintln!("Fuzzy score for '{}': {}", session_name, fuzzy_score);
}
```

## Extension Points

### Adding New Match Sources
1. Add variant to `MatchSource` enum
2. Extend resurrection metadata with new field
3. Add search logic in `search_sessions()`
4. Add source indicator in `render_snippet_browser()`

### Customizing Search Weights
Modify weights in `search_sessions()`:
```rust
let score = 80;  // Change base weight here
```

### Customizing Fuzzy Match Boost
```rust
let score = 80 + (fuzzy_score / 10).min(20);
              //  ^----------^     ^----^
              //  divisor          max boost
```

## Migration Notes

### From Phase 4 to Phase 5
- No breaking changes to existing search
- Gracefully handles missing resurrection data
- Existing Claude search behavior unchanged
- New features activated automatically when data present

### Database Schema (if applicable)
No database changes required. Resurrection metadata is:
- In-memory only during session load
- Serialized with session JSON
- Not persisted separately

## Performance Optimization Tips

### Avoid Redundant Searches
```rust
// Good: Check once per session
if let Some(ref resurrection) = session.resurrection {
    // All tmux searches here
}

// Bad: Multiple resurrection checks
if session.resurrection.is_some() { /* session name */ }
if session.resurrection.is_some() { /* command */ }
if session.resurrection.is_some() { /* directory */ }
```

### Reuse Matcher Instance
```rust
// Good: Single matcher for all searches
let matcher = SkimMatcherV2::default();
for session in sessions {
    matcher.fuzzy_match(...);  // Reuse
}

// Bad: New matcher per match
for session in sessions {
    let matcher = SkimMatcherV2::default();  // Expensive
    matcher.fuzzy_match(...);
}
```

### Limit Match Results Early
```rust
// Truncate matches before sorting (already implemented)
search_matches.truncate(200);
```

## Known Limitations

1. **No Multi-Pane Support**: Currently searches only first pane of tmux session
2. **No Live Tmux State**: Searches resurrect backups, not live tmux state
3. **No Command History**: Searches current command, not command history
4. **Fixed Weights**: Score weights are hardcoded, not user-configurable

## Future Enhancement Ideas

- [ ] Color-coded match sources in UI
- [ ] Filter by match source (dropdown/flag)
- [ ] Search across multiple tmux panes
- [ ] Integration with live tmux API
- [ ] User-configurable search weights
- [ ] Regex support for command matching
- [ ] Path component matching (e.g., "core" matches "*/core/*")

---

**Quick Links**:
- [Implementation Summary](./PHASE5_SEARCH_INTEGRATION_SUMMARY.md)
- [Models Source](./claude-session-tui/src/models.rs)
- [App Source](./claude-session-tui/src/ui/app.rs)
