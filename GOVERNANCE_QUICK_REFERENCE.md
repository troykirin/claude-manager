# Governance Integration - Quick Reference

One-page reference for session recovery governance integration.

## Setup (One-Time)

```bash
# Build CLI
cd federation-integration
bun build src/recovery/cli.ts --outdir ./dist/recovery --target node

# Configure
export LOKI_URL="http://wsl:3100"
export LINEAR_TEAM_ID="your-team-id"
export ENABLE_LINEAR_ISSUES="true"
export SEVERITY_THRESHOLD="50"
```

## Bash Integration

```bash
# Source library
source lib/governance.sh

# Log diagnostic
log_diagnostic_event "$session_id" "success" "$health_score" "$corruption_patterns"

# Log repair
log_repair_event "$session_id" "success" "$health_score" "$backup_path" "$duration_ms"

# Log verify
log_verify_event "$session_id" "success" "$health_score"

# Log rollback
log_rollback_event "$session_id" "failure" "$backup_path" "$duration_ms" "$error_msg"

# Automatic timing
with_recovery_logging "repair" "$session_id" repair_command "$session_id"

# Generate report
generate_audit_report "$session_id"

# Query history
query_recovery_history --session "$session_id"
query_recovery_history --operation repair --outcome failure
query_recovery_history --max-health 50

# Get metrics
get_recovery_metrics
```

## CLI Commands

```bash
# Log operation
node dist/recovery/cli.js log \
  --operation diagnose \
  --session "abc123" \
  --outcome success \
  --health-score 75 \
  --corruption-patterns "missing-blocks,corrupted-metadata"

# Generate report (text)
node dist/recovery/cli.js report --session "abc123"

# Generate report (JSON)
node dist/recovery/cli.js report --session "abc123" --format json

# Query all events
node dist/recovery/cli.js query

# Query filtered
node dist/recovery/cli.js query \
  --session "abc123" \
  --operation repair \
  --outcome failure \
  --min-health 0 \
  --max-health 50

# Get metrics
node dist/recovery/cli.js metrics
node dist/recovery/cli.js metrics --format json
```

## Event Types

| Operation | When to Use | Required Fields |
|-----------|-------------|-----------------|
| `diagnose` | After running diagnostic | session_id, outcome, health_score |
| `repair` | After repair attempt | session_id, outcome, health_score, backup_location |
| `verify` | After verification | session_id, outcome, health_score |
| `rollback` | After rollback | session_id, outcome, backup_location |

## Outcome Values

- `success`: Operation completed successfully
- `failure`: Operation failed
- `partial`: Operation partially successful

## Health Score Guidelines

- **90-100**: Excellent health, no issues
- **70-89**: Good health, minor issues
- **50-69**: Moderate issues, monitor
- **30-49**: Significant corruption (triggers Linear issue by default)
- **0-29**: Critical corruption (triggers urgent Linear issue)

## Loki Queries

```logql
# All recovery operations
{job="session-recovery"}

# Failed operations
{job="session-recovery", outcome="failure"}

# Low health scores
{job="session-recovery"} | json | health_score < 50

# Specific session
{job="session-recovery", session_id="abc123"}

# Operations by type
{job="session-recovery", operation="repair"}

# Time range
{job="session-recovery"} |= "2025-01-15"
```

## Linear Issue Triggers

Issues are created when:
1. `outcome === 'failure'` (any operation)
2. `healthScore < severityThreshold` (default: 50)

Issue priority:
- **Priority 1** (Urgent): Failure
- **Priority 2** (High): Health <30
- **Priority 3** (Medium): Health 30-49

## Configuration Options

```json
{
  "lokiUrl": "http://localhost:3100",
  "linearTeamId": "team-id-here",
  "enableLinearIssues": true,
  "severityThreshold": 50,
  "enableMocks": false
}
```

## Environment Variables

```bash
RECOVERY_AUDIT_CONFIG="/path/to/config.json"
LOKI_URL="http://localhost:3100"
LINEAR_TEAM_ID="your-team-id"
ENABLE_LINEAR_ISSUES="true"
SEVERITY_THRESHOLD="50"
```

## Common Patterns

### Pattern 1: Diagnostic with Logging
```bash
session_id="$1"

if ./diagnose.sh "$session_id" > /tmp/diag.txt; then
    health=$(extract_health /tmp/diag.txt)
    patterns=$(extract_patterns /tmp/diag.txt)
    log_diagnostic_event "$session_id" "success" "$health" "$patterns"
else
    log_diagnostic_event "$session_id" "failure" "0"
fi
```

### Pattern 2: Repair with Backup
```bash
session_id="$1"
backup="/tmp/backup-$(date +%s).json"

# Backup
cp "$session_file" "$backup"

# Repair with timing
start=$(date +%s%3N)
if ./repair.sh "$session_id"; then
    duration=$(( $(date +%s%3N) - start ))
    health=$(get_health "$session_id")
    log_repair_event "$session_id" "success" "$health" "$backup" "$duration"
else
    duration=$(( $(date +%s%3N) - start ))
    log_repair_event "$session_id" "failure" "0" "$backup" "$duration"
fi
```

### Pattern 3: Rollback on Failure
```bash
session_id="$1"
backup="$2"

if [[ ! -f "$backup" ]]; then
    log_rollback_event "$session_id" "failure" "$backup" "" "Backup not found"
    exit 1
fi

if cp "$backup" "$session_file"; then
    log_rollback_event "$session_id" "success" "$backup"
else
    log_rollback_event "$session_id" "failure" "$backup" "" "Restore failed"
fi
```

## Troubleshooting

### Issue: CLI not found
```bash
cd federation-integration
bun build src/recovery/cli.ts --outdir ./dist/recovery --target node
chmod +x dist/recovery/cli.js
```

### Issue: Loki connection fails
```bash
# Test connectivity
curl -v http://localhost:3100/ready

# Check Loki container
docker ps | grep loki
docker logs loki
```

### Issue: Linear issues not created
1. Check `ENABLE_LINEAR_ISSUES="true"`
2. Verify `LINEAR_TEAM_ID` is correct
3. Ensure health score below threshold
4. Check Linear API credentials

### Issue: Events not appearing in queries
- Events are per-process (CLI state is ephemeral)
- For persistent queries, check Loki directly
- Use `query_recovery_history` only for in-process events

## Testing

```bash
# Run test suite
cd federation-integration
bun test test/recovery/audit.test.ts

# Test CLI
export ENABLE_LINEAR_ISSUES="false"
node dist/recovery/cli.js log \
  --operation diagnose \
  --session test-123 \
  --outcome success \
  --health-score 85

# Test bash integration
bash -c 'source lib/governance.sh && check_cli'
```

## Files

- **Audit Module**: `federation-integration/src/recovery/audit.ts`
- **CLI**: `federation-integration/src/recovery/cli.ts`
- **Bash Library**: `lib/governance.sh`
- **Config**: `recovery-audit.json` or environment variables
- **Tests**: `federation-integration/test/recovery/audit.test.ts`
- **Documentation**: `federation-integration/RECOVERY_AUDIT_CONFIG.md`

## Links

- Full Documentation: `federation-integration/RECOVERY_AUDIT_CONFIG.md`
- Implementation Summary: `CM-303_IMPLEMENTATION_SUMMARY.md`
- Test Suite: `federation-integration/test/recovery/audit.test.ts`
