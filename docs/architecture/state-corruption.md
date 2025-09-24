# Claude Session State Corruption Analysis

## Overview

Through operational experience and reverse engineering, we have identified several critical patterns that lead to Claude session state corruption. This document catalogs these patterns, their root causes, and prevention strategies.

## Corruption Pattern 1: Session Branch/Split Collision

### Description
Creating a new session branch while the original session remains active leads to guaranteed state corruption.

### Scenario
1. User has active Claude session (Session A)
2. User creates new session branch/split (Session B) from Session A
3. User attempts to resume original Session A
4. **RESULT**: Session A becomes corrupted and unrecoverable

### Root Cause Analysis
```
Timeline of Corruption:
T0: Session A active, writing to:
    - ~/.claude/todos/{session-a-uuid}-agent-*.json
    - ~/.claude/shell-snapshots/snapshot-*.sh
    - ~/.claude/statsig/session_id.*

T1: User creates Session B branch
    - New process spawned
    - Session B creates own state files
    - Session A process CONTINUES RUNNING

T2: Session A continues operations
    - Updates todos with new tasks
    - Creates new shell snapshots
    - Updates statsig metadata

T3: User tries to resume Session A
    - Finds inconsistent state across systems
    - Todo files reference tasks not in JSONL
    - Shell snapshots don't match conversation context
    - System cannot reconcile state → CORRUPTION
```

### Technical Details
- **Primary Issue**: Race condition between Session A and B state updates
- **Secondary Issue**: Session UUID cross-references become invalid
- **Tertiary Issue**: Shell environment state diverges from conversation state

### Symptoms
- Cannot resume conversation
- History navigation broken (double-tap ESC fails)
- Error messages about invalid session state
- Orphaned todo files
- Mismatched shell snapshots

### Recovery Method
User has developed CLI recovery tool that:
1. Analyzes corrupted session state
2. Rebuilds consistent cross-references
3. Reconciles todo state with JSONL content
4. Restores session resumability

## Corruption Pattern 2: Migration During Active Sessions

### Description
Attempting to migrate sessions while Claude processes are actively running.

### Scenario
1. Multiple Claude processes active (30+ processes observed)
2. User runs migration script
3. Migration attempts to move/update files
4. Active processes continue writing to old locations
5. **RESULT**: Partial migration with orphaned state

### Root Cause Analysis
- **File Locking**: Active processes hold locks on session files
- **Atomic Operation Failure**: Migration cannot complete atomically
- **Process Race Conditions**: Multiple processes updating same session UUID

### Technical Impact
```bash
# Before Migration:
~/.claude/projects/-Users-tryk-nabia-claude-project-migrator/
    6fd63673-de16-46f0-bf8b-20e667da9657.jsonl
~/.claude/todos/
    6fd63673-de16-46f0-bf8b-20e667da9657-agent-*.json

# After Failed Migration:
~/.claude/projects/-Users-tryk-nabia-claude-project-migrator/
    6fd63673-de16-46f0-bf8b-20e667da9657.jsonl  # OLD LOCATION
~/.claude/projects/-Users-tryk-nabia-claude-manager/
    6fd63673-de16-46f0-bf8b-20e667da9657.jsonl  # NEW LOCATION (partial)
~/.claude/todos/
    6fd63673-de16-46f0-bf8b-20e667da9657-agent-*.json  # STALE REFS
```

### Prevention Strategy
1. **Process Detection**: Scan for active Claude processes
2. **Graceful Shutdown**: Request clean session closure
3. **Lock Detection**: Check for file locks before migration
4. **Atomic Operations**: Use transaction-like migration approach

## Corruption Pattern 3: Cross-System Inconsistency

### Description
Session UUID exists in multiple systems with inconsistent or conflicting state.

### Common Scenarios
- Session in projects directory but no corresponding todos
- Todo files reference non-existent sessions
- Shell snapshots timestamp mismatches
- Statsig session metadata out of sync

### Detection Method
```bash
# Cross-system validation check:
for session_uuid in $(ls ~/.claude/projects/*/); do
    project_exists=$(ls ~/.claude/projects/*/${session_uuid}.jsonl 2>/dev/null | wc -l)
    todo_exists=$(ls ~/.claude/todos/${session_uuid}-agent-*.json 2>/dev/null | wc -l)
    
    if [[ $project_exists -gt 0 && $todo_exists -eq 0 ]]; then
        echo "WARNING: Orphaned project session: $session_uuid"
    fi
    
    if [[ $project_exists -eq 0 && $todo_exists -gt 0 ]]; then
        echo "WARNING: Orphaned todo session: $session_uuid"
    fi
done
```

## Corruption Pattern 4: Directory Name Mismatch

### Description
Project directory name no longer matches the `cwd` paths stored in session JSONL files.

### Scenario
1. Session started in `/Users/tryk/project-old`
2. Project renamed to `/Users/tryk/project-new`
3. Directory: `~/.claude/projects/-Users-tryk-project-old/`
4. JSONL content: `"cwd": "/Users/tryk/project-old"`
5. User works in `/Users/tryk/project-new`
6. **RESULT**: Path mismatch causes session loading failures

### Technical Root Cause
Claude's path resolution logic:
1. Encodes current working directory into project directory name
2. Stores actual paths in JSONL `cwd` fields
3. Uses both for session resolution
4. Mismatch between encoded name and stored paths = failure

## Prevention Strategies

### 1. Process Management
```bash
# Safe session closure protocol:
claude_pids=$(ps aux | grep claude | grep -v grep | awk '{print $2}')
if [[ -n "$claude_pids" ]]; then
    echo "Requesting graceful shutdown..."
    kill -TERM $claude_pids
    sleep 5
    
    # Verify closure
    remaining=$(ps aux | grep claude | grep -v grep | wc -l)
    if [[ $remaining -gt 0 ]]; then
        echo "ERROR: Active Claude processes detected. Migration unsafe."
        exit 1
    fi
fi
```

### 2. State Validation
```bash
# Pre-migration validation:
validate_session_state() {
    local session_uuid="$1"
    
    # Check project exists
    project_file=$(find ~/.claude/projects -name "${session_uuid}.jsonl" 2>/dev/null)
    [[ -n "$project_file" ]] || return 1
    
    # Check todos exist
    todo_count=$(ls ~/.claude/todos/${session_uuid}-agent-*.json 2>/dev/null | wc -l)
    [[ $todo_count -gt 0 ]] || return 1
    
    # Check timestamp consistency
    project_mtime=$(stat -c %Y "$project_file" 2>/dev/null || stat -f %m "$project_file")
    todo_mtime=$(stat -c %Y ~/.claude/todos/${session_uuid}-agent-*.json 2>/dev/null | head -1 || stat -f %m ~/.claude/todos/${session_uuid}-agent-*.json | head -1)
    
    # Allow 1-hour difference for clock skew
    time_diff=$((project_mtime - todo_mtime))
    [[ ${time_diff#-} -lt 3600 ]] || return 1
    
    return 0
}
```

### 3. Atomic Migration Protocol
```bash
# Transaction-like migration:
migrate_session_atomic() {
    local session_uuid="$1"
    local old_path="$2" 
    local new_path="$3"
    
    # Phase 1: Backup all related files
    backup_dir="/tmp/claude-migration-backup-$(date +%s)"
    mkdir -p "$backup_dir"
    
    # Backup projects, todos, shell-snapshots, statsig
    cp -r ~/.claude/projects/*/${session_uuid}.jsonl "$backup_dir/" 2>/dev/null
    cp -r ~/.claude/todos/${session_uuid}-agent-*.json "$backup_dir/" 2>/dev/null
    
    # Phase 2: Update all systems atomically
    if ! update_project_paths "$session_uuid" "$old_path" "$new_path"; then
        restore_from_backup "$backup_dir"
        return 1
    fi
    
    # Phase 3: Verify consistency
    if ! validate_session_state "$session_uuid"; then
        restore_from_backup "$backup_dir"
        return 1
    fi
    
    # Phase 4: Cleanup backup
    rm -rf "$backup_dir"
    return 0
}
```

## Recovery Procedures

### Automated Recovery Tool Specifications
Based on user's existing CLI recovery tool, a production recovery system should:

1. **State Analysis Engine**
   - Scan all four data stores
   - Build dependency graph
   - Identify inconsistencies

2. **Conflict Resolution**
   - Merge conflicting todo states
   - Rebuild session chronology
   - Restore cross-references

3. **Validation Framework**
   - Test session resumability
   - Verify history navigation
   - Confirm environment consistency

### Manual Recovery Procedures
When automated recovery fails:

1. **Identify Session UUID**: `grep -r "sessionId" ~/.claude/projects/*/`
2. **Backup All Related Files**: Before any manual intervention
3. **Rebuild Todo State**: Extract from JSONL conversation history
4. **Restore Shell Environment**: Use most recent valid snapshot
5. **Test Resumability**: Attempt to resume session before declaring success

## Production Monitoring

### Health Check Metrics
- **Cross-system consistency ratio**: (Valid sessions / Total sessions)
- **Orphaned file count**: Files without cross-references
- **Corruption rate**: Failed resumption attempts per day
- **Recovery success rate**: Automated recovery effectiveness

### Alert Triggers
- Active processes detected during migration
- Cross-system timestamp drift > 1 hour
- Session UUID found in only one system
- Failed session resumption attempts

---

*This analysis is based on operational experience with Claude session corruption and successful recovery procedures. These patterns must be addressed for production-ready session management.*