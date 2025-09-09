# Claude Manager Production Safety Protocols

## Overview

This document defines the mandatory safety protocols for production-ready Claude session migration. These protocols are based on reverse-engineering analysis and operational experience with session state corruption.

## Pre-Migration Safety Checks

### Phase 1: Environment Analysis

#### 1.1 Process Detection Protocol
```bash
#!/bin/bash
detect_active_processes() {
    local claude_processes=$(ps aux | grep -E '(claude|node.*claude)' | grep -v grep)
    local process_count=$(echo "$claude_processes" | wc -l)
    
    if [[ $process_count -gt 0 ]]; then
        echo "⚠️  CRITICAL: $process_count active Claude processes detected"
        echo "$claude_processes"
        echo ""
        echo "RECOMMENDATION: Close all Claude sessions before migration"
        return 1
    fi
    
    echo "✅ No active Claude processes detected"
    return 0
}
```

#### 1.2 Session State Inventory
```bash
inventory_session_state() {
    local session_uuid="$1"
    local inventory_file="/tmp/claude-migration-inventory-$(date +%s).json"
    
    echo "📊 Creating session state inventory for: $session_uuid"
    
    cat > "$inventory_file" << EOF
{
    "session_uuid": "$session_uuid",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "systems": {
        "projects": {
            "files": $(find ~/.claude/projects -name "${session_uuid}.jsonl" | jq -R . | jq -s .),
            "count": $(find ~/.claude/projects -name "${session_uuid}.jsonl" | wc -l),
            "total_size": $(find ~/.claude/projects -name "${session_uuid}.jsonl" -exec stat -c%s {} + | awk '{sum+=$1} END {print sum}')
        },
        "todos": {
            "files": $(ls ~/.claude/todos/${session_uuid}-agent-*.json 2>/dev/null | jq -R . | jq -s .),
            "count": $(ls ~/.claude/todos/${session_uuid}-agent-*.json 2>/dev/null | wc -l),
            "last_modified": "$(stat -c %y ~/.claude/todos/${session_uuid}-agent-*.json 2>/dev/null | head -1 || echo 'N/A')"
        },
        "shell_snapshots": {
            "count": $(ls ~/.claude/shell-snapshots/snapshot-*-*.sh 2>/dev/null | wc -l),
            "latest": "$(ls -t ~/.claude/shell-snapshots/snapshot-*-*.sh 2>/dev/null | head -1 || echo 'N/A')"
        },
        "statsig": {
            "files": $(ls ~/.claude/statsig/ 2>/dev/null | jq -R . | jq -s .),
            "active_session": "$(cat ~/.claude/statsig/statsig.session_id.* 2>/dev/null || echo 'N/A')"
        }
    }
}
EOF
    
    echo "📋 Inventory saved to: $inventory_file"
    jq . "$inventory_file"
    echo "$inventory_file"
}
```

#### 1.3 Cross-System Validation
```bash
validate_cross_system_consistency() {
    local session_uuid="$1"
    local errors=0
    
    echo "🔍 Validating cross-system consistency for: $session_uuid"
    
    # Check project existence
    local project_files=$(find ~/.claude/projects -name "${session_uuid}.jsonl" | wc -l)
    if [[ $project_files -eq 0 ]]; then
        echo "❌ ERROR: No project files found for session $session_uuid"
        ((errors++))
    elif [[ $project_files -gt 1 ]]; then
        echo "⚠️  WARNING: Multiple project files found for session $session_uuid"
        find ~/.claude/projects -name "${session_uuid}.jsonl"
    else
        echo "✅ Project file: OK"
    fi
    
    # Check todos existence
    local todo_files=$(ls ~/.claude/todos/${session_uuid}-agent-*.json 2>/dev/null | wc -l)
    if [[ $todo_files -eq 0 ]]; then
        echo "⚠️  WARNING: No todo files found for session $session_uuid (may be archived)"
    else
        echo "✅ Todo files: $todo_files found"
    fi
    
    # Check timestamp consistency
    if [[ $project_files -gt 0 && $todo_files -gt 0 ]]; then
        local project_file=$(find ~/.claude/projects -name "${session_uuid}.jsonl" | head -1)
        local latest_todo=$(ls -t ~/.claude/todos/${session_uuid}-agent-*.json 2>/dev/null | head -1)
        
        local project_mtime=$(stat -c %Y "$project_file" 2>/dev/null || stat -f %m "$project_file")
        local todo_mtime=$(stat -c %Y "$latest_todo" 2>/dev/null || stat -f %m "$latest_todo")
        
        local time_diff=$((project_mtime - todo_mtime))
        local abs_diff=${time_diff#-}
        
        if [[ $abs_diff -gt 3600 ]]; then
            echo "⚠️  WARNING: Timestamp drift detected (${abs_diff}s difference)"
            echo "   Project: $(date -d @$project_mtime 2>/dev/null || date -r $project_mtime)"
            echo "   Todo:    $(date -d @$todo_mtime 2>/dev/null || date -r $todo_mtime)"
        else
            echo "✅ Timestamp consistency: OK"
        fi
    fi
    
    return $errors
}
```

### Phase 2: Migration Readiness Assessment

#### 2.1 Path Analysis
```bash
analyze_migration_paths() {
    local old_path="$1"
    local new_path="$2"
    local session_uuid="$3"
    
    echo "📂 Analyzing migration paths:"
    echo "   From: $old_path"
    echo "   To:   $new_path"
    
    # Validate paths exist
    if [[ ! -d "$old_path" ]]; then
        echo "❌ ERROR: Source path does not exist: $old_path"
        return 1
    fi
    
    if [[ ! -d "$new_path" ]]; then
        echo "❌ ERROR: Destination path does not exist: $new_path"
        return 1
    fi
    
    # Check for JSONL path consistency
    local project_file=$(find ~/.claude/projects -name "${session_uuid}.jsonl" | head -1)
    if [[ -n "$project_file" ]]; then
        local stored_paths=$(grep -o '"cwd":"[^"]*"' "$project_file" | cut -d'"' -f4 | sort -u)
        echo "📄 Stored paths in JSONL:"
        echo "$stored_paths" | sed 's/^/   /'
        
        local path_match=$(echo "$stored_paths" | grep -c "^$old_path$")
        if [[ $path_match -eq 0 ]]; then
            echo "⚠️  WARNING: Old path not found in JSONL. Migration may not be necessary."
        else
            echo "✅ Path consistency: OK"
        fi
    fi
    
    return 0
}
```

#### 2.2 Resource Lock Detection
```bash
detect_resource_locks() {
    local session_uuid="$1"
    
    echo "🔒 Checking for resource locks:"
    
    # Check file locks using lsof
    local locked_files=$(lsof ~/.claude/projects/*/${session_uuid}.jsonl 2>/dev/null)
    if [[ -n "$locked_files" ]]; then
        echo "❌ ERROR: Session files are locked by active processes"
        echo "$locked_files"
        return 1
    fi
    
    # Check todo file locks
    local locked_todos=$(lsof ~/.claude/todos/${session_uuid}-agent-*.json 2>/dev/null)
    if [[ -n "$locked_todos" ]]; then
        echo "❌ ERROR: Todo files are locked by active processes"
        echo "$locked_todos"
        return 1
    fi
    
    echo "✅ No resource locks detected"
    return 0
}
```

## Safe Migration Protocol

### Phase 3: Atomic Migration Execution

#### 3.1 Backup Creation
```bash
create_migration_backup() {
    local session_uuid="$1"
    local backup_dir="/tmp/claude-migration-backup-${session_uuid}-$(date +%s)"
    
    echo "💾 Creating migration backup: $backup_dir"
    mkdir -p "$backup_dir"/{projects,todos,shell-snapshots,statsig}
    
    # Backup project files
    find ~/.claude/projects -name "${session_uuid}.jsonl" -exec cp {} "$backup_dir/projects/" \;
    
    # Backup todo files
    cp ~/.claude/todos/${session_uuid}-agent-*.json "$backup_dir/todos/" 2>/dev/null
    
    # Backup recent shell snapshots (last 10)
    ls -t ~/.claude/shell-snapshots/snapshot-*-*.sh 2>/dev/null | head -10 | \
        xargs -I {} cp {} "$backup_dir/shell-snapshots/" 2>/dev/null
    
    # Backup statsig files
    cp ~/.claude/statsig/* "$backup_dir/statsig/" 2>/dev/null
    
    # Create manifest
    cat > "$backup_dir/manifest.json" << EOF
{
    "session_uuid": "$session_uuid",
    "backup_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "backup_dir": "$backup_dir",
    "files_backed_up": {
        "projects": $(find "$backup_dir/projects" -type f | wc -l),
        "todos": $(find "$backup_dir/todos" -type f | wc -l),
        "shell_snapshots": $(find "$backup_dir/shell-snapshots" -type f | wc -l),
        "statsig": $(find "$backup_dir/statsig" -type f | wc -l)
    }
}
EOF
    
    echo "✅ Backup created: $backup_dir"
    echo "$backup_dir"
}
```

#### 3.2 Atomic Path Update
```bash
update_session_paths_atomic() {
    local session_uuid="$1"
    local old_path="$2"
    local new_path="$3"
    local backup_dir="$4"
    
    echo "🔄 Executing atomic path update:"
    echo "   Session: $session_uuid"
    echo "   $old_path → $new_path"
    
    # Phase 1: Update JSONL files
    local project_files=$(find ~/.claude/projects -name "${session_uuid}.jsonl")
    local update_count=0
    local error_count=0
    
    while IFS= read -r project_file; do
        if [[ -z "$project_file" ]]; then continue; fi
        
        echo "   📝 Updating: $project_file"
        
        # Create temporary file for atomic update
        local temp_file="${project_file}.migrate.tmp"
        
        # Update paths using sed
        if sed "s|\"cwd\":\"$old_path\"|\"cwd\":\"$new_path\"|g" "$project_file" > "$temp_file"; then
            # Verify update was successful
            local old_count=$(grep -c "\"cwd\":\"$old_path\"" "$project_file" || echo 0)
            local new_count=$(grep -c "\"cwd\":\"$new_path\"" "$temp_file" || echo 0)
            
            if [[ $new_count -eq $old_count ]]; then
                # Atomic move
                mv "$temp_file" "$project_file"
                echo "   ✅ Updated: $new_count paths"
                ((update_count++))
            else
                echo "   ❌ ERROR: Path count mismatch (expected: $old_count, got: $new_count)"
                rm -f "$temp_file"
                ((error_count++))
            fi
        else
            echo "   ❌ ERROR: Failed to update $project_file"
            rm -f "$temp_file"
            ((error_count++))
        fi
    done <<< "$project_files"
    
    # Phase 2: Move to new project directory
    local old_project_dir=$(dirname "$(find ~/.claude/projects -name "${session_uuid}.jsonl" | head -1)")
    local new_project_dir="~/.claude/projects/$(echo "$new_path" | sed 's|/|-|g')"
    local expanded_new_dir="${HOME}/.claude/projects/$(echo "$new_path" | sed 's|/|-|g')"
    
    if [[ "$old_project_dir" != "$expanded_new_dir" ]]; then
        echo "   📂 Moving project directory:"
        echo "      From: $old_project_dir"
        echo "      To:   $expanded_new_dir"
        
        mkdir -p "$expanded_new_dir"
        if mv "$old_project_dir"/*.jsonl "$expanded_new_dir/" 2>/dev/null; then
            echo "   ✅ Project directory moved successfully"
            # Remove old directory if empty
            rmdir "$old_project_dir" 2>/dev/null
        else
            echo "   ❌ ERROR: Failed to move project directory"
            ((error_count++))
        fi
    fi
    
    if [[ $error_count -gt 0 ]]; then
        echo "❌ Migration failed with $error_count errors"
        return 1
    fi
    
    echo "✅ Atomic migration completed: $update_count files updated"
    return 0
}
```

### Phase 4: Post-Migration Validation

#### 4.1 Migration Verification
```bash
verify_migration_success() {
    local session_uuid="$1"
    local new_path="$2"
    
    echo "🔍 Verifying migration success:"
    
    # Check new project directory exists
    local new_project_dir="${HOME}/.claude/projects/$(echo "$new_path" | sed 's|/|-|g')"
    local new_project_file="$new_project_dir/${session_uuid}.jsonl"
    
    if [[ ! -f "$new_project_file" ]]; then
        echo "❌ ERROR: Session file not found in new location: $new_project_file"
        return 1
    fi
    
    # Verify path updates
    local new_path_count=$(grep -c "\"cwd\":\"$new_path\"" "$new_project_file" 2>/dev/null || echo 0)
    if [[ $new_path_count -eq 0 ]]; then
        echo "❌ ERROR: No updated paths found in migrated file"
        return 1
    fi
    
    echo "✅ Found $new_path_count updated paths in migrated file"
    
    # Check file integrity
    if ! jq empty "$new_project_file" 2>/dev/null; then
        echo "❌ ERROR: Migrated file contains invalid JSON"
        return 1
    fi
    
    echo "✅ File integrity check passed"
    
    # Verify session can be loaded (basic check)
    local session_id=$(head -1 "$new_project_file" | jq -r '.sessionId' 2>/dev/null)
    if [[ "$session_id" != "$session_uuid" ]]; then
        echo "❌ ERROR: Session ID mismatch in migrated file"
        echo "   Expected: $session_uuid"
        echo "   Found:    $session_id"
        return 1
    fi
    
    echo "✅ Session ID consistency verified"
    echo "✅ Migration verification completed successfully"
    return 0
}
```

#### 4.2 Session Resume Test
```bash
test_session_resumability() {
    local session_uuid="$1"
    local new_path="$2"
    
    echo "🧪 Testing session resumability (dry-run):"
    
    # This would require integration with Claude's internal session loader
    # For now, we perform basic structural validation
    
    local project_file="${HOME}/.claude/projects/$(echo "$new_path" | sed 's|/|-|g')/${session_uuid}.jsonl"
    
    # Check required fields exist
    local required_fields=("sessionId" "cwd" "timestamp")
    local missing_fields=()
    
    for field in "${required_fields[@]}"; do
        if ! head -1 "$project_file" | jq -e ".$field" >/dev/null 2>&1; then
            missing_fields+=("$field")
        fi
    done
    
    if [[ ${#missing_fields[@]} -gt 0 ]]; then
        echo "❌ ERROR: Missing required fields: ${missing_fields[*]}"
        return 1
    fi
    
    echo "✅ Session structure validation passed"
    
    # Check message chronology
    local message_count=$(wc -l < "$project_file")
    if [[ $message_count -lt 1 ]]; then
        echo "❌ ERROR: Empty session file"
        return 1
    fi
    
    echo "✅ Session contains $message_count messages"
    echo "✅ Session resumability test completed"
    
    return 0
}
```

## Rollback Procedures

### Emergency Rollback
```bash
emergency_rollback() {
    local backup_dir="$1"
    
    echo "🚨 EMERGENCY ROLLBACK INITIATED"
    echo "   Backup: $backup_dir"
    
    if [[ ! -d "$backup_dir" ]]; then
        echo "❌ CRITICAL: Backup directory not found: $backup_dir"
        return 1
    fi
    
    # Restore project files
    if [[ -d "$backup_dir/projects" ]]; then
        echo "📁 Restoring project files..."
        find "$backup_dir/projects" -name "*.jsonl" -exec cp {} ~/.claude/projects/ \;
    fi
    
    # Restore todo files
    if [[ -d "$backup_dir/todos" ]]; then
        echo "📋 Restoring todo files..."
        cp "$backup_dir/todos"/*.json ~/.claude/todos/ 2>/dev/null
    fi
    
    # Restore statsig files
    if [[ -d "$backup_dir/statsig" ]]; then
        echo "📊 Restoring statsig files..."
        cp "$backup_dir/statsig"/* ~/.claude/statsig/ 2>/dev/null
    fi
    
    echo "✅ Emergency rollback completed"
    echo "⚠️  IMPORTANT: Verify session state before resuming work"
    
    return 0
}
```

## Production Checklist

### Pre-Migration (MANDATORY)
- [ ] All active Claude processes terminated
- [ ] Session state inventory completed
- [ ] Cross-system consistency validated
- [ ] Resource locks checked
- [ ] Backup created and verified

### During Migration (MANDATORY)
- [ ] Atomic path updates completed
- [ ] Project directory moved successfully
- [ ] No errors during file operations
- [ ] Temporary files cleaned up

### Post-Migration (MANDATORY)
- [ ] Migration verification passed
- [ ] Session resumability tested
- [ ] Cross-system consistency re-validated
- [ ] Backup retention policy applied

### Emergency Procedures (READY)
- [ ] Rollback procedure tested and ready
- [ ] Emergency contact information available
- [ ] Recovery tools accessible
- [ ] Backup integrity verified

---

*These protocols must be followed for all production Claude session migrations. Deviating from these procedures risks permanent session state corruption.*