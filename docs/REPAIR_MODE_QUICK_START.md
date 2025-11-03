# REPAIR Mode Quick Start Guide

**CM-302: Safe Session Duplication with Corruption Isolation**

---

## What is REPAIR Mode?

REPAIR mode safely duplicates a corrupted Claude session, creating a new session with a clean UUID while preserving all conversation history. The original session is archived for forensic analysis.

### When to Use REPAIR Mode

- Session fails to load or crashes Claude
- Session displays corruption warnings
- Todo state is out of sync
- Cross-system references are broken
- After session diagnostics indicate corruption

---

## Quick Start

### 1. Basic Usage

```bash
cd /Users/tryk/nabia/tools/claude-manager
./claude-manager.sh repair <session-uuid>
```

### 2. Find Your Session UUID

```bash
# List all sessions
./claude-manager.sh list

# Or find in project directories
find ~/.claude/projects -name "*.jsonl" | head -5
```

### 3. Run REPAIR

```bash
# Example UUID
./claude-manager.sh repair 550e8400-e29b-41d4-a716-446655440000
```

---

## What REPAIR Does

### The 6-Phase Workflow

1. **Pre-flight Checks** (5-10 seconds)
   - Validates session exists
   - Checks disk space (need 500MB)
   - Verifies JSONL integrity
   - Detects active Claude processes

2. **Backup Creation** (10-30 seconds)
   - Backs up all 4 state systems:
     - Projects (JSONL files)
     - Todos (agent state)
     - Statsig (telemetry)
     - Shell snapshots (recent)
   - Location: `~/.local/state/nabi/repairs/`

3. **Safe Duplication** (5-15 seconds)
   - Generates new UUID
   - Copies JSONL content (source of truth)
   - Rebuilds clean todo state
   - Creates fresh cross-references

4. **Corruption Isolation** (5-10 seconds)
   - Moves original to `~/.claude/.archive/`
   - Timestamps archive directory
   - Creates forensic manifest

5. **State Restoration** (5-10 seconds)
   - Validates JSONL integrity
   - Updates cross-system references
   - Synchronizes todo state

6. **Verification** (5-10 seconds)
   - Checks new session exists
   - Validates JSONL integrity
   - Calculates health score (must be >70)
   - Verifies backup preservation

**Total Time**: Under 2 minutes

---

## Output Example

```bash
$ ./claude-manager.sh repair 550e8400-e29b-41d4-a716-446655440000

[INFO] === REPAIR Mode: Session Duplication with Corruption Isolation ===
[INFO] Session ID: 550e8400-e29b-41d4-a716-446655440000
[INFO] Timestamp: 20241102_153045

[INFO] Phase 1/6: Pre-flight checks...
[SUCCESS] ✓ Session file found
[SUCCESS] ✓ No active Claude processes
[SUCCESS] ✓ Sufficient disk space: 5000MB available
[SUCCESS] ✓ Projects directory writable
[SUCCESS] ✓ JSONL integrity OK (125 lines)
[SUCCESS] Pre-flight checks passed

[INFO] Phase 2/6: Creating safety backup...
[INFO] Creating backup at: ~/.local/state/nabi/repairs/repair-550e8400-20241102_153045
[INFO]   ✓ Backed up 1 project file(s)
[INFO]   ✓ Backed up 2 todo file(s)
[INFO]   ✓ Backed up 0 statsig file(s)
[INFO]   ✓ Backed up 5 shell snapshot(s)
[SUCCESS] Backup manifest created

[INFO] Phase 3/6: Safe duplication...
[INFO] Generated new UUID: a1b2c3d4-e5f6-7890-abcd-ef1234567890
[INFO] Duplicating JSONL content...
[SUCCESS] ✓ Updated sessionId in new file
[INFO] Rebuilding todo state from JSONL...
[INFO]   ✓ Created clean todo state
[INFO] Creating fresh cross-references...
[SUCCESS] Safe duplication completed

[INFO] Phase 4/6: Isolating corruption...
[INFO] Archiving original corrupted session...
[SUCCESS] ✓ Archived 1 file(s) to: ~/.claude/.archive/sessions-20241102_153045

[INFO] Phase 5/6: Restoring valid state...
[INFO] Restoring valid state from JSONL source of truth...
[INFO] Updating cross-system references...
[SUCCESS]   ✓ Todo state synchronized
[SUCCESS] State restoration completed

[INFO] Phase 6/6: Post-repair verification...
[INFO] Verifying repair success...
[SUCCESS] ✓ New session file exists
[SUCCESS] ✓ JSONL integrity OK (125 lines)
[SUCCESS] ✓ Original session archived
[SUCCESS] ✓ Backup directory preserved
[SUCCESS] ✓ Line count matches (125 lines)
[SUCCESS] Post-repair verification passed (health score: 100/100)

[SUCCESS] === REPAIR Mode: Session repair completed successfully ===
[INFO] New session UUID: a1b2c3d4-e5f6-7890-abcd-ef1234567890
[INFO] Backup location: ~/.local/state/nabi/repairs/repair-550e8400-20241102_153045
[INFO] Original session archived at: ~/.claude/.archive/
```

---

## What Happens After REPAIR

### New Session Created

- **Location**: Same project directory as original
- **UUID**: New UUID (different from original)
- **Content**: Identical conversation history
- **State**: Clean, no corruption

### Original Session Archived

- **Location**: `~/.claude/.archive/sessions-YYYYMMDD_HHMMSS/`
- **Contents**:
  - Original JSONL file
  - Original todo files
  - Archive manifest (ARCHIVE_REASON.txt)
- **Purpose**: Forensic analysis, recovery if needed

### Backup Preserved

- **Location**: `~/.local/state/nabi/repairs/repair-<uuid>-YYYYMMDD_HHMMSS/`
- **Contents**:
  - All 4 state systems
  - Backup manifest (manifest.json)
- **Purpose**: Emergency rollback, data recovery

---

## Using the Repaired Session

### 1. Resume Work in Claude

Open Claude and select the **new session** (not the original):
- Look for the new UUID in your session list
- Session will have same conversation history
- State will be clean and functional

### 2. Verify Everything Works

- Check that messages load correctly
- Verify todos are accessible
- Test file operations
- Ensure no crashes or errors

### 3. Clean Up (Optional)

After confirming the repair worked:

```bash
# Keep backup for 7 days, then remove
rm -rf ~/.local/state/nabi/repairs/repair-<uuid>-<timestamp>

# Archive can stay (minimal disk space)
# Or compress for long-term storage
```

---

## Rollback (If Needed)

If the repaired session has issues:

```bash
# The backup includes a rollback script (coming in CM-303)
# For now, manually restore from backup:

# Find your backup
ls -la ~/.local/state/nabi/repairs/

# Restore original
cp ~/.local/state/nabi/repairs/repair-<uuid>-<timestamp>/projects/*/*.jsonl \
   ~/.claude/projects/<project-dir>/

cp ~/.local/state/nabi/repairs/repair-<uuid>-<timestamp>/todos/*.json \
   ~/.claude/todos/
```

---

## Troubleshooting

### REPAIR Fails at Pre-flight

**Issue**: "Session not found"
- **Fix**: Verify UUID is correct
- **Check**: `find ~/.claude/projects -name "*.jsonl"`

**Issue**: "Insufficient disk space"
- **Fix**: Free up at least 500MB
- **Check**: `df -h ~/.claude`

**Issue**: "Claude processes running"
- **Fix**: Close all Claude windows
- **Check**: `pgrep -f Claude`

### REPAIR Fails During Duplication

**Issue**: "Failed to duplicate JSONL file"
- **Cause**: Permissions issue
- **Fix**: Check write permissions on project directory

**Issue**: "Invalid JSON"
- **Cause**: JSONL file is corrupted
- **Fix**: Restore from a recent backup first

### Health Score Below 70

**Issue**: "Health score below threshold"
- **Cause**: Some verification checks failed
- **Fix**: Review output for specific issues
- **Action**: Check backup is complete before using new session

---

## Advanced Usage

### Interactive Mode

```bash
# Prompts for session UUID
./claude-manager.sh repair
```

### With Verbose Logging

```bash
# Enable debug output
CLAUDE_DEBUG=1 ./claude-manager.sh repair <uuid>
```

### Check REPAIR Availability

```bash
# Verify REPAIR module is installed
./claude-manager.sh help | grep repair
```

---

## Safety Features

### Automatic Rollback

If REPAIR fails at any phase:
- Automatically rolls back to pre-repair state
- Restores from backup
- Removes partial duplicate
- Leaves session intact

### Data Loss Prevention

- **Backup-first approach**: Nothing changes until backup is complete
- **Atomic operations**: Each phase is self-contained
- **Verification**: Health checks at start and end
- **Preservation**: Original session always archived (never deleted)

### No Surprises

- Clear progress reporting at each phase
- Detailed logging with color-coded output
- Final summary with new UUID and locations
- Health score for quality assurance

---

## FAQ

**Q: Will I lose my conversation history?**
A: No, REPAIR preserves 100% of conversation history (JSONL is source of truth).

**Q: Can I continue using the original session?**
A: No, the original is archived. Use the new session instead.

**Q: What if REPAIR fails?**
A: Automatic rollback restores original session. No data loss.

**Q: How long does REPAIR take?**
A: Under 2 minutes for most sessions.

**Q: Can I undo REPAIR?**
A: Yes, backup includes original session. Manual restore currently required (CM-303 will add automatic restore).

**Q: Is REPAIR safe to run?**
A: Yes, backup-first approach ensures no data loss. Original always preserved.

---

## Getting Help

### Check Logs

```bash
# Recent REPAIR operations
ls -la ~/.local/state/nabi/repairs/

# Archive history
ls -la ~/.claude/.archive/

# Backup manifests
find ~/.local/state/nabi/repairs -name "manifest.json" -exec cat {} \;
```

### Run Diagnostics

```bash
# System health check
./claude-manager.sh health

# Session verification (after repair)
./claude-manager.sh verify ~/.claude/projects/<project-dir>
```

### Report Issues

If REPAIR fails:
1. Copy the full output
2. Note the session UUID
3. Check backup location
4. Run health check
5. Report with all details

---

## Next Steps

After successful REPAIR:
1. ✅ Resume work in new session
2. ✅ Verify functionality
3. ✅ Monitor for issues (first 24 hours)
4. ✅ Keep backup for 7 days
5. ✅ Clean up old backups

---

*Quick Start Guide for CM-302 REPAIR Mode*
*Part of NOS-678 Session Recovery Phase 1*
