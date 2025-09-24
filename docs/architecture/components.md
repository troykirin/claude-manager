# Claude Manager - Technical Reference

## Overview

Claude Manager is a production-ready tool for safely migrating Claude sessions between project directories. Built from reverse-engineering Claude's internal session state architecture, it provides atomic operations and comprehensive safety protocols to prevent session corruption.

## Architecture

### Claude Session State System

Claude maintains session state across four interconnected systems:

```
~/.claude/
├── projects/           # Primary session content (JSONL)
├── todos/             # Active session state & coordination
├── statsig/           # Analytics & session tracking  
└── shell-snapshots/   # Environment state preservation
```

### Critical Dependencies

- **Session UUID**: Primary key linking all systems
- **Path Encoding**: Project directories named by startup path
- **Cross-References**: Same UUID appears across all four systems
- **Process Coordination**: Multiple Claude processes can affect same session

## Tool Capabilities

### Core Operations

1. **Path Migration (`migrate`)**: Update session paths within existing project
2. **Project Moving (`move`)**: Move sessions between project directories
3. **Full Migration (`full`)**: Combined path update + project move
4. **State Validation**: Cross-system consistency checks
5. **Atomic Operations**: Transaction-like migration safety

### Enhanced Safety Features

- **Process Detection**: Identifies active Claude sessions
- **Lock Detection**: Checks for file system locks
- **Cross-System Validation**: Ensures state consistency
- **Atomic Backups**: Complete state preservation
- **Rollback Procedures**: Emergency recovery capabilities

## Usage Patterns

### Safe Migration Process

```bash
# 1. Pre-flight safety checks
./claude-project-migrator.sh config
detect_active_processes
validate_cross_system_consistency "$SESSION_UUID"

# 2. Create backup
backup_dir=$(create_migration_backup "$SESSION_UUID")

# 3. Execute migration
./claude-project-migrator.sh full \
    "/Users/tryk/nabia/claude-project-migrator" \
    "/Users/tryk/nabia/claude-manager" \
    "/Users/tryk/.claude/projects/-Users-tryk-nabia-claude-project-migrator" \
    "/Users/tryk/.claude/projects/-Users-tryk-nabia-claude-manager"

# 4. Verify success
verify_migration_success "$SESSION_UUID" "/Users/tryk/nabia/claude-manager"
```

### Production Environment Variables

```bash
export CLAUDE_DIR="$HOME/.claude"                    # Claude directory
export CLAUDE_BACKUP_STRATEGY="file"                 # file|project  
export CLAUDE_INTERACTIVE="false"                    # Disable prompts
export CLAUDE_DRY_RUN="false"                       # Execute changes
export CLAUDE_SAFETY_CHECKS="true"                  # Enable safety protocols
```

## Command Reference

### Basic Commands

```bash
# List all projects and sessions
claude_manager list

# Show configuration
claude_manager config

# Interactive migration (with safety checks)
claude_manager migrate

# Non-interactive migration
CLAUDE_INTERACTIVE=false claude_manager migrate \
    "/old/path" "/new/path" "/Users/user/.claude/projects/old-project"
```

### Advanced Operations

```bash
# Dry run with full validation
CLAUDE_DRY_RUN=true claude_manager full \
    "/old/path" "/new/path" "/old/project" "/new/project"

# Project backup strategy
CLAUDE_BACKUP_STRATEGY=project claude_manager migrate \
    "/old/path" "/new/path"

# Force migration (bypass safety checks)
CLAUDE_SAFETY_CHECKS=false claude_manager full \
    "/old/path" "/new/path" "/old/project" "/new/project"
```

## Safety Protocols

### Mandatory Pre-Migration Checks

1. **Process Detection**: Scan for active Claude processes
2. **Resource Locks**: Check for file system locks
3. **State Consistency**: Validate cross-system references
4. **Path Analysis**: Verify migration paths exist and are valid

### Atomic Operation Guarantees

- **All-or-Nothing**: Either complete success or complete rollback
- **State Preservation**: All related files backed up before changes
- **Consistency Maintenance**: Cross-system references remain valid
- **Process Safety**: No concurrent process conflicts

### Emergency Procedures

```bash
# Emergency rollback
emergency_rollback "/tmp/claude-migration-backup-1234567890"

# State validation
validate_cross_system_consistency "$SESSION_UUID"

# Manual recovery (last resort)
# 1. Identify corrupted session
# 2. Restore from backup
# 3. Rebuild cross-references
# 4. Test resumability
```

## Common Issues and Solutions

### Issue 1: "Session in use by active process"

**Symptoms**: Migration fails with process lock error

**Solution**:
```bash
# Find active processes
ps aux | grep claude | grep -v grep

# Graceful shutdown
kill -TERM $(ps aux | grep claude | awk '{print $2}')

# Wait and verify
sleep 5
detect_active_processes
```

### Issue 2: "Cross-system state inconsistency"

**Symptoms**: Session UUID found in some systems but not others

**Solution**:
```bash
# Inventory session state
inventory_file=$(inventory_session_state "$SESSION_UUID")

# Check which systems have the session
validate_cross_system_consistency "$SESSION_UUID"

# Manual cleanup if needed
# Remove orphaned files from systems that shouldn't have them
```

### Issue 3: "Migration verification failed"

**Symptoms**: Migration completes but verification fails

**Solution**:
```bash
# Check file integrity
jq empty ~/.claude/projects/*/sessionfile.jsonl

# Verify path updates
grep '"cwd":' ~/.claude/projects/*/sessionfile.jsonl

# Test session structure
test_session_resumability "$SESSION_UUID" "/new/path"

# Emergency rollback if needed
emergency_rollback "$backup_dir"
```

## Monitoring and Alerting

### Health Metrics

- **Migration Success Rate**: Successful migrations / Total attempts
- **Rollback Frequency**: Emergency rollbacks / Total migrations  
- **State Consistency**: Valid sessions / Total sessions
- **Process Conflicts**: Active process detections during migration

### Alert Conditions

- Active Claude processes detected during migration
- Cross-system timestamp drift > 1 hour
- Session UUID orphaned in single system
- Migration verification failures
- Emergency rollback triggered

## Development and Testing

### Test Environment Setup

```bash
# Create test session
claude --project-dir "/tmp/test-project"

# Generate test data
# (conversation with multiple tool calls and state changes)

# Verify test session has cross-system state
validate_cross_system_consistency "$TEST_SESSION_UUID"
```

### Testing Checklist

- [ ] Basic path migration
- [ ] Project directory moving  
- [ ] Full migration workflow
- [ ] Dry run accuracy
- [ ] Backup and restore
- [ ] Emergency rollback
- [ ] Process conflict handling
- [ ] State validation accuracy

## Performance Considerations

### Scalability Factors

- **Session Count**: Linear growth with number of sessions
- **File Size**: JSONL files can be large (2MB+ observed)
- **Cross-System Checks**: Multiplicative cost with state complexity
- **Backup Size**: Full backups can be substantial

### Optimization Strategies

- **Incremental Validation**: Only check modified sessions
- **Parallel Processing**: Migrate multiple sessions concurrently
- **Selective Backup**: Backup only essential files
- **Lazy Loading**: Load session data on-demand

## Integration Points

### Federation Integration

- **Memory Chain**: Session state coordination
- **Loki Events**: Migration event logging
- **Agent Coordination**: Multi-agent session handling

### Hook Integration

- **Pre-Migration Hooks**: Custom validation logic
- **Post-Migration Hooks**: Custom verification steps
- **Rollback Hooks**: Custom recovery procedures

## Security Considerations

### Data Protection

- **Session Content**: Contains conversation history and tool outputs
- **Environment State**: Shell snapshots may contain sensitive data
- **Backup Security**: Temporary backups in /tmp may be world-readable

### Access Control

- **File Permissions**: Ensure proper ownership and permissions
- **Process Isolation**: Prevent concurrent access conflicts
- **Audit Trail**: Log all migration operations

## Troubleshooting Guide

### Diagnostic Commands

```bash
# Session inventory
inventory_session_state "$SESSION_UUID"

# Process analysis  
ps aux | grep claude | grep -v grep

# File lock detection
lsof ~/.claude/projects/*/*.jsonl

# Cross-system validation
validate_cross_system_consistency "$SESSION_UUID"

# Backup verification
ls -la /tmp/claude-migration-backup-*
```

### Recovery Procedures

1. **Identify Problem**: Use diagnostic commands to assess situation
2. **Stop Active Processes**: Prevent further state corruption
3. **Restore from Backup**: Use most recent valid backup
4. **Validate Recovery**: Ensure session can be resumed
5. **Document Incident**: Record for future prevention

---

*This technical reference is based on reverse-engineering analysis and operational experience. It represents the current understanding of Claude's session state architecture and provides production-ready migration capabilities.*