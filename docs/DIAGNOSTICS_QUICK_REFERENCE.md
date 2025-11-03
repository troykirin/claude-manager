# Diagnostics Quick Reference

## Quick Start

```bash
# Diagnose current session
cm diagnose

# Diagnose specific session
cm diagnose <session-uuid>

# Get JSON output
DIAGNOSE_JSON=true cm diagnose <session-uuid>
```

## Corruption Patterns Detected

| Pattern | Description | Severity |
|---------|-------------|----------|
| Branch Collision | Session in multiple projects | -20 pts |
| Migration Race | Duplicate files or backups | -20 pts |
| Cross-System Issues | Orphaned files | -20 pts |
| Path Mismatch | Directory vs content mismatch | -20 pts |
| Orphaned Todos | Todos without project | -20 pts |
| Timestamp Drift | >1 hour time difference | -20 pts |

## Health Score Interpretation

| Score | Severity | Status | Action |
|-------|----------|--------|--------|
| 100-90 | HEALTHY | ✅ | No action needed |
| 89-70 | MINOR_ISSUES | ⚠ | Monitor |
| 69-50 | DEGRADED | ⚠ | Consider recovery |
| 49-30 | CORRUPTED | ❌ | Run REPAIR |
| 29-0 | CRITICAL | ❌ | Manual intervention |

## Common Scenarios

### Scenario 1: Healthy Session
```
✓ No branch collision
✓ No migration race
✓ Cross-system consistency OK
✓ Path consistency OK
✓ No orphaned todos
✓ Timestamps consistent

Health Score: 100/100 - HEALTHY ✅
```

### Scenario 2: Path Mismatch After Move
```
✓ No branch collision
✓ No migration race
✓ Cross-system consistency OK
✗ Path mismatch: Expected: /old/path, Found: /new/path
✓ No orphaned todos
✓ Timestamps consistent

Health Score: 80/100 - MINOR_ISSUES ⚠
```
**Fix**: Run `cm migrate /old/path /new/path`

### Scenario 3: Orphaned Todos
```
✓ No branch collision
✓ No migration race
✗ Cross-system issues: orphaned todos (3 files)
✓ Path consistency OK
✗ Orphaned todos: 3 orphaned todo files
✓ Timestamps consistent

Health Score: 60/100 - DEGRADED ⚠
```
**Fix**: Remove orphaned todos or restore project file

### Scenario 4: Branch Collision
```
✗ Branch collision: Found in 2 different project directories
✓ No migration race
⚠ Cross-system state: 2 project files
✓ Path consistency OK
✓ No orphaned todos
✓ Timestamps consistent

Health Score: 70/100 - MINOR_ISSUES ⚠
```
**Fix**: Consolidate to single project directory

## Validation Checks

| Check | Description | Warning Penalty |
|-------|-------------|----------------|
| Process Safety | Active Claude processes | -10 pts |
| Cross-System State | Missing or duplicate files | -10 pts |
| Path Consistency | Multiple paths in session | -10 pts |

## Troubleshooting

### "Cannot detect current session UUID"
**Cause**: No active statsig session file
**Fix**: Provide explicit session UUID: `cm diagnose <uuid>`

### "No project file found"
**Cause**: Session UUID doesn't exist
**Fix**: Verify UUID with `cm list`

### "Session doesn't match source directory"
**Cause**: Path mismatch
**Fix**: Run `cm verify <project-dir>` to see all paths, then migrate if needed

## Integration with Other Commands

```bash
# Workflow: Diagnose → Verify → Migrate
cm diagnose <session-uuid>           # Detect issues
cm verify <project-dir>               # Verify specific project
cm migrate /old/path /new/path        # Fix path issues

# Workflow: Health Check → Diagnose
cm health                             # Check system health
cm diagnose current                   # Check session health
```

## JSON Output Format

```json
{
  "session_uuid": "...",
  "health_score": 80,
  "severity": "MINOR_ISSUES",
  "corruption_patterns": [
    {
      "name": "Branch Collision",
      "detected": false,
      "details": ""
    },
    ...
  ],
  "validations": [
    {
      "name": "Process Safety",
      "status": "ok",
      "details": ""
    },
    ...
  ]
}
```

## Performance

- **Execution Time**: <2 seconds typical
- **I/O Operations**: Minimal (metadata only)
- **Safety**: Read-only, no modifications

## See Also

- Full documentation: `README.md` (Health Diagnostics section)
- Architecture docs: `docs/architecture/state-corruption.md`
- Safety protocols: `docs/architecture/safety-protocols.md`
- Implementation: `docs/CM-301_IMPLEMENTATION_SUMMARY.md`
