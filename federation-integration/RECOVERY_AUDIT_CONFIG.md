# Recovery Audit Trail Configuration

Configuration guide for the governance integration layer.

## Configuration File

Create a `recovery-audit.json` file with the following structure:

```json
{
  "lokiUrl": "http://localhost:3100",
  "linearTeamId": "your-linear-team-id",
  "enableLinearIssues": true,
  "severityThreshold": 50,
  "enableMocks": false
}
```

## Configuration Options

### `lokiUrl` (required)
- **Type**: `string`
- **Default**: `http://localhost:3100`
- **Description**: URL of the Loki instance for log ingestion
- **Example**: `http://wsl:3100` or `https://loki.example.com`

### `linearTeamId` (required)
- **Type**: `string`
- **Description**: Linear team ID for issue creation
- **How to find**: Visit Linear workspace settings > Teams > Copy team ID
- **Example**: `a1b2c3d4-e5f6-7890-abcd-ef1234567890`

### `enableLinearIssues` (required)
- **Type**: `boolean`
- **Default**: `false`
- **Description**: Enable automatic Linear issue creation for critical failures
- **When to enable**: Production environments with Linear integration
- **When to disable**: Development, testing, or when Linear is not available

### `severityThreshold` (required)
- **Type**: `number` (0-100)
- **Default**: `50`
- **Description**: Health score below which Linear issues are created
- **Examples**:
  - `30`: Only create issues for severe corruption
  - `50`: Balanced threshold (recommended)
  - `70`: Create issues for any degradation

### `enableMocks` (optional)
- **Type**: `boolean`
- **Default**: `false`
- **Description**: Enable mock mode for testing (skips actual Loki/Linear calls)
- **Use cases**: Unit testing, development without dependencies

## Environment Variables

Configuration can also be provided via environment variables (overrides config file):

```bash
export RECOVERY_AUDIT_CONFIG="/path/to/config.json"
export LOKI_URL="http://localhost:3100"
export LINEAR_TEAM_ID="your-team-id"
export ENABLE_LINEAR_ISSUES="true"
export SEVERITY_THRESHOLD="50"
```

## Bash Integration

Source the governance library in your recovery scripts:

```bash
#!/usr/bin/env bash
source "$(dirname "$0")/lib/governance.sh"

# Log diagnostic
log_diagnostic_event "$session_id" "success" "75" "missing-blocks"

# Log repair with automatic timing
with_recovery_logging "repair" "$session_id" \
    ./repair-command "$session_id"

# Generate report
generate_audit_report "$session_id"
```

## CLI Usage

### Log an operation
```bash
node dist/recovery/cli.js log \
  --operation diagnose \
  --session "abc123" \
  --outcome success \
  --health-score 75 \
  --corruption-patterns "missing-blocks,corrupted-metadata"
```

### Generate audit report
```bash
# Text format
node dist/recovery/cli.js report --session "abc123"

# JSON format
node dist/recovery/cli.js report --session "abc123" --format json
```

### Query history
```bash
# All events
node dist/recovery/cli.js query

# Filter by session
node dist/recovery/cli.js query --session "abc123"

# Filter by operation and outcome
node dist/recovery/cli.js query \
  --operation repair \
  --outcome failure

# Filter by health score
node dist/recovery/cli.js query \
  --min-health 0 \
  --max-health 50
```

### Get metrics
```bash
# Text format
node dist/recovery/cli.js metrics

# JSON format
node dist/recovery/cli.js metrics --format json
```

## Integration Examples

### Example 1: Diagnostic Workflow
```bash
#!/usr/bin/env bash
source lib/governance.sh

session_id="$1"

# Run diagnostic
if ./diagnose-session.sh "$session_id"; then
    health=$(get_health_score "$session_id")
    patterns=$(get_corruption_patterns "$session_id")
    log_diagnostic_event "$session_id" "success" "$health" "$patterns"
else
    log_diagnostic_event "$session_id" "failure" "0"
fi
```

### Example 2: Repair with Backup
```bash
#!/usr/bin/env bash
source lib/governance.sh

session_id="$1"
backup_location="/tmp/backup-$(date +%s).json"

# Create backup
cp "$session_file" "$backup_location"

# Repair with logging
start=$(date +%s%3N)
if ./repair-session.sh "$session_id"; then
    end=$(date +%s%3N)
    duration=$((end - start))
    health=$(get_health_score "$session_id")
    log_repair_event "$session_id" "success" "$health" "$backup_location" "$duration"
else
    end=$(date +%s%3N)
    duration=$((end - start))
    log_repair_event "$session_id" "failure" "0" "$backup_location" "$duration"
fi
```

### Example 3: Rollback on Failure
```bash
#!/usr/bin/env bash
source lib/governance.sh

session_id="$1"
backup_location="$2"

if [[ ! -f "$backup_location" ]]; then
    log_rollback_event "$session_id" "failure" "$backup_location" "" "Backup file not found"
    exit 1
fi

if cp "$backup_location" "$session_file"; then
    log_rollback_event "$session_id" "success" "$backup_location"
else
    log_rollback_event "$session_id" "failure" "$backup_location" "" "Copy failed"
fi
```

## Loki Query Examples

Query recovery events in Grafana/Loki:

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
```

## Linear Issue Format

When `enableLinearIssues` is true and severity threshold is met, issues are created with:

**Title**: `Session Recovery {outcome}: {session_id}`

**Description**:
```markdown
**Operation**: diagnose
**Outcome**: failure
**Session ID**: abc123def456
**Health Score**: 35%
**Duration**: 1250ms
**Backup**: /tmp/backup.json

**Corruption Patterns**:
- missing-blocks
- corrupted-metadata

**Error**:
```
Validation failed: block sequence mismatch
```
```

**Priority**:
- Failure: Priority 1 (Urgent)
- Health <30: Priority 2 (High)
- Health <50: Priority 3 (Medium)

**Labels**: `session-recovery`, `{operation}`, `{outcome}`

## Troubleshooting

### Loki connection fails
```bash
# Test connectivity
curl -v http://localhost:3100/ready

# Check Loki logs
docker logs loki

# Verify payload format
node dist/recovery/cli.js log --operation diagnose --session test --outcome success --health-score 75
```

### Linear issues not created
1. Verify `enableLinearIssues: true` in config
2. Check Linear team ID is correct
3. Ensure health score below severity threshold
4. Check Linear API credentials/permissions

### CLI not found
```bash
# Build CLI
cd federation-integration
bun build src/recovery/cli.ts --outdir ./dist/recovery --target node
chmod +x dist/recovery/cli.js
```

## Best Practices

1. **Always log operations**: Use audit trail for all recovery operations
2. **Include context**: Add corruption patterns and backup locations
3. **Set appropriate thresholds**: Tune severity threshold to avoid alert fatigue
4. **Monitor metrics**: Regularly check recovery metrics for trends
5. **Generate reports**: Create audit reports for post-mortems
6. **Test with mocks**: Use `enableMocks: true` in development

## Security Considerations

1. **Sensitive data**: Audit logs may contain session metadata - secure Loki instance
2. **Access control**: Restrict access to recovery CLI and audit trail
3. **Backup locations**: Ensure backup paths are secure and validated
4. **Linear integration**: Use scoped API tokens with minimal permissions

## Performance

- **Async logging**: All Loki/Linear calls are non-blocking
- **Batch operations**: Consider batching for bulk recovery operations
- **Error handling**: Failed logging doesn't block recovery operations
- **Metrics overhead**: Minimal - calculated from in-memory events
