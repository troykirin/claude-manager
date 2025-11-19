# Phase 5: Usage Examples

**Audience**: End users, testers, QA engineers
**Purpose**: Demonstrate search behavior with tmux integration

---

## Basic Search Examples

### Example 1: Search by Project Name

**Query**: `"nabia"`

**Expected Results**:
```
Match 1/8 (Score: 1520) - Session 15 Block 7
  "...working on nabia federation system..."

Match 2/8 (Score: 95) - Session 3 Block 1 (from tmux session)
  Tmux session: nabia-core-work

Match 3/8 (Score: 48) - Session 12 Block 1 (from tmux dir)
  Working dir: /Users/tryk/nabia/platform
```

**Explanation**:
- Match 1: Claude session content mentions "nabia" (highest score)
- Match 2: Tmux session named "nabia-core-work" (fuzzy match)
- Match 3: Working directory contains "nabia" (fuzzy path match)

---

### Example 2: Search by Shell Command

**Query**: `"bash"`

**Expected Results**:
```
Match 1/4 (Score: 95) - Session 8 Block 1 (from tmux session)
  Tmux session: bash-scripting

Match 2/4 (Score: 60) - Session 22 Block 1 (from tmux command)
  Running: bash -l

Match 3/4 (Score: 60) - Session 14 Block 1 (from tmux command)
  Running: /bin/bash --login
```

**Explanation**:
- Match 1: Tmux session with "bash" in name (fuzzy + high weight)
- Match 2-3: Shell commands containing "bash" (exact substring)

---

### Example 3: Search by Directory Path

**Query**: `"/Users/tryk/nabia/core"`

**Expected Results**:
```
Match 1/3 (Score: 1200) - Session 19 Block 3
  "...modified /Users/tryk/nabia/core/hooks..."

Match 2/3 (Score: 48) - Session 7 Block 1 (from tmux dir)
  Working dir: /Users/tryk/nabia/core

Match 3/3 (Score: 45) - Session 11 Block 1 (from tmux dir)
  Working dir: /Users/tryk/nabia/core/kernel
```

**Explanation**:
- Match 1: Claude session discusses the exact path
- Match 2: Tmux session working in exact directory (fuzzy match)
- Match 3: Tmux session in subdirectory (fuzzy path match)

---

## Advanced Search Scenarios

### Scenario 1: Multiple Match Types (Same Session)

**Query**: `"schema-driven"`

**Session 5 has**:
- Claude content: "implementing schema-driven config"
- Tmux session: "schema-driven-work"
- Working dir: "/Users/tryk/nabia/schema-driven"

**Expected Results**:
```
Match 1/6 (Score: 1800) - Session 5 Block 2
  "...implementing schema-driven config system..."

Match 2/6 (Score: 98) - Session 5 Block 1 (from tmux session)
  Tmux session: schema-driven-work

Match 3/6 (Score: 48) - Session 5 Block 1 (from tmux dir)
  Working dir: /Users/tryk/nabia/schema-driven
```

**Note**: Session 5 appears 3 times with different match types!

---

### Scenario 2: Fuzzy Matching

**Query**: `"fdrtion"` (typo for "federation")

**Expected Results** (fuzzy matching enabled):
```
Match 1/2 (Score: 92) - Session 16 Block 1 (from tmux session)
  Tmux session: federation-work

Match 2/2 (Score: 46) - Session 23 Block 1 (from tmux dir)
  Working dir: /Users/tryk/federation-core
```

**Explanation**:
- Fuzzy matcher finds "federation" despite typo
- Sessions without exact match still appear

---

### Scenario 3: Partial Word Match

**Query**: `"nav"`

**Expected Results**:
```
Match 1/5 (Score: 600) - Session 3 Block 4
  "...navigation system implementation..."

Match 2/5 (Score: 88) - Session 9 Block 1 (from tmux session)
  Tmux session: navbar-design

Match 3/5 (Score: 45) - Session 12 Block 1 (from tmux dir)
  Working dir: /Users/tryk/projects/navigation
```

**Explanation**:
- Fuzzy matching finds words containing "nav"
- Partial matches still scored appropriately

---

## Edge Cases

### Edge Case 1: No Tmux Data

**Session 1**: Claude session only (no resurrection data)
**Query**: `"testing"`

**Expected Results**:
```
Match 1/1 (Score: 1000) - Session 1 Block 8
  "...writing comprehensive testing suite..."
```

**Behavior**: Works normally, only searches Claude content

---

### Edge Case 2: Partial Tmux Data

**Session 2**: Has tmux session name, but no command/directory
**Query**: `"debug"`

**Expected Results**:
```
Match 1/2 (Score: 92) - Session 2 Block 1 (from tmux session)
  Tmux session: debug-session

Match 2/2 (Score: 500) - Session 2 Block 5
  "...debugging the authentication flow..."
```

**Behavior**: Searches available fields only, no errors

---

### Edge Case 3: Empty Search

**Query**: `""` (empty string)

**Expected Results**:
```
(Shows all sessions, no filtering)
```

**Behavior**: Existing behavior preserved, shows all sessions

---

### Edge Case 4: No Matches

**Query**: `"xyzabc123notfound"`

**Expected Results**:
```
No matches found
(Displays all sessions as fallback)
```

**Behavior**: Graceful fallback to unfiltered list

---

## Performance Examples

### Fast Search (< 100ms)
**Session Count**: 50
**Query**: `"rust"`
**Expected Time**: ~50-80ms

**What makes it fast**:
- Small session count
- Short query
- Existing fuzzy matcher optimization

---

### Medium Search (100-300ms)
**Session Count**: 200
**Query**: `"/Users/tryk"`
**Expected Time**: ~150-250ms

**What affects speed**:
- More sessions to scan
- Longer query (path)
- Multiple resurrection field checks

---

### Slow Search (300-500ms)
**Session Count**: 500
**Query**: `"implementation"`
**Expected Time**: ~350-450ms

**What makes it slower**:
- Large session count
- Long query requiring fuzzy matching
- Multiple match types per session

**Note**: Still within <500ms target for 100 sessions

---

## UI Interaction Examples

### Workflow 1: Browse Search Results

1. **Type search**: `/schema` → Enter
2. **Results appear**: "5 matches found (press 'v' to browse snippets)"
3. **Press 'v'**: Switch to snippet browser view
4. **See first match**:
   ```
   Match 1/5 (Score: 1600) - Session 8 Block 3
   ────────────────────────────────────────────────
   "Implementing schema-driven configuration system
   with TOML validation and JSON transformation..."
   ```
5. **Press 'n'**: Next match
6. **See second match**:
   ```
   Match 2/5 (Score: 95) - Session 8 Block 1 (from tmux session)
   ────────────────────────────────────────────────
   Tmux session: schema-driven-config
   ```

---

### Workflow 2: Navigate Between Match Types

**Session 10** has 3 matches:
- Claude content (score: 1400)
- Tmux session (score: 90)
- Working dir (score: 42)

**Navigation**:
1. First result shows Claude match (highest score)
2. Press 'n' → Shows tmux session match (with source indicator)
3. Press 'n' → Shows working dir match (with source indicator)
4. Press 'n' → Wraps to next session's matches

---

### Workflow 3: Filter by Score

**All results sorted by score**:
```
Match 1/10 (Score: 1800) - Session 5 Block 2
Match 2/10 (Score: 1500) - Session 12 Block 7
Match 3/10 (Score: 1200) - Session 3 Block 4
Match 4/10 (Score: 95) - Session 5 Block 1 (from tmux session)
Match 5/10 (Score: 90) - Session 8 Block 1 (from tmux session)
Match 6/10 (Score: 85) - Session 14 Block 1 (from tmux session)
Match 7/10 (Score: 60) - Session 7 Block 1 (from tmux command)
Match 8/10 (Score: 60) - Session 11 Block 1 (from tmux command)
Match 9/10 (Score: 48) - Session 5 Block 1 (from tmux dir)
Match 10/10 (Score: 42) - Session 19 Block 1 (from tmux dir)
```

**Pattern**: Claude → Tmux Session → Command → Directory

---

## Testing Checklist

### Manual Testing
- [ ] Search with various queries (short, long, paths)
- [ ] Verify source indicators display correctly
- [ ] Check score ordering is correct
- [ ] Test fuzzy matching accuracy
- [ ] Verify edge cases (no data, partial data)

### Automated Testing
- [ ] Unit test search scoring algorithm
- [ ] Integration test match source tagging
- [ ] Performance test with 100/500 sessions
- [ ] Regression test existing Claude search

### User Acceptance Testing
- [ ] Can users find sessions by tmux name?
- [ ] Are source indicators helpful?
- [ ] Is result ordering intuitive?
- [ ] Any confusing behavior?

---

## Troubleshooting

### "No matches found" but expected results

**Check**:
1. Is resurrection data loaded? (Phase 1-3 complete?)
2. Is query case-sensitive? (shouldn't be, but verify)
3. Are tmux fields populated? (check session.resurrection)

**Debug**:
```rust
// In load_sessions()
for session in &sessions {
    if let Some(ref res) = session.resurrection {
        println!("Session has resurrection data: {:?}", res.tmux);
    }
}
```

---

### Match scores seem wrong

**Check**:
1. Verify base weights: 80 (session), 60 (command), 40 (dir)
2. Check fuzzy boost calculation: score + (fuzzy/10).min(20)
3. Confirm substring fallback working

**Debug**:
```rust
// In search_sessions()
println!("Match score: {} from source: {:?}", score, source);
```

---

### Source indicators not displaying

**Check**:
1. Verify MatchSource enum values are set
2. Check render_snippet_browser() source_indicator logic
3. Confirm match.source field is populated

**Debug**:
```rust
// In render_snippet_browser()
println!("Match source: {:?}", match_info.source);
```

---

## Summary

Phase 5 search integration provides:
- ✅ Seamless multi-source search
- ✅ Intuitive result ordering
- ✅ Clear match source indicators
- ✅ Graceful edge case handling
- ✅ Consistent user experience

**Ready for Phase 6 testing and user validation.**
