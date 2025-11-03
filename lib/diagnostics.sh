#!/usr/bin/env bash
# Claude Manager Health Diagnostics Engine
# Detects session state corruption patterns and generates health scores

# Corruption pattern detection functions

# Pattern 1: Session Branch/Split Collision
detect_branch_collision() {
    local session_uuid="$1"
    local detected=false
    local details=""

    # Find all project directories containing this session
    local projects=()
    while IFS= read -r project_file; do
        if [[ -n "$project_file" ]]; then
            projects+=("$(dirname "$project_file")")
        fi
    done < <(find "$CLAUDE_DIR/projects" -name "${session_uuid}.jsonl" 2>/dev/null)

    # Multiple projects = potential branch collision
    if [[ ${#projects[@]} -gt 1 ]]; then
        detected=true
        details="Found in ${#projects[@]} different project directories"
    fi

    echo "$detected|$details"
}

# Pattern 2: Migration During Active Sessions
detect_migration_race() {
    local session_uuid="$1"
    local detected=false
    local details=""

    # Check for duplicate session files (indicates interrupted migration)
    local session_files=()
    while IFS= read -r file; do
        if [[ -n "$file" ]]; then
            session_files+=("$file")
        fi
    done < <(find "$CLAUDE_DIR/projects" -name "${session_uuid}.jsonl" 2>/dev/null)

    if [[ ${#session_files[@]} -gt 1 ]]; then
        detected=true
        details="Found ${#session_files[@]} copies of session file"

        # Check for backup files (indicates interrupted operation)
        local backups
        backups=$(find "$CLAUDE_DIR/projects" -name "${session_uuid}.jsonl.bak" -o -name "${session_uuid}.jsonl.pre-move-backup" 2>/dev/null | wc -l)
        if [[ "$backups" -gt 0 ]]; then
            details="$details, ${backups} backup files found"
        fi
    fi

    echo "$detected|$details"
}

# Pattern 3: Cross-System Inconsistency
detect_cross_system_inconsistency() {
    local session_uuid="$1"
    local detected=false
    local details=""
    local issues=()

    # Check project existence
    local project_count
    project_count=$(find "$CLAUDE_DIR/projects" -name "${session_uuid}.jsonl" 2>/dev/null | wc -l)

    # Check todos existence
    local todo_count
    todo_count=$(ls "$CLAUDE_DIR/todos/${session_uuid}"-agent-*.json 2>/dev/null | wc -l)

    # Check shell snapshots
    local snapshot_count
    snapshot_count=$(ls "$CLAUDE_DIR/shell-snapshots/snapshot-${session_uuid}"-*.sh 2>/dev/null | wc -l)

    # Validate consistency
    if [[ "$project_count" -eq 0 && "$todo_count" -gt 0 ]]; then
        detected=true
        issues+=("orphaned todos ($todo_count files)")
    fi

    if [[ "$project_count" -gt 0 && "$todo_count" -eq 0 ]]; then
        # Not necessarily corruption - session might be completed/archived
        issues+=("no todos (possibly archived)")
    fi

    if [[ "$project_count" -eq 0 && "$snapshot_count" -gt 0 ]]; then
        detected=true
        issues+=("orphaned snapshots ($snapshot_count files)")
    fi

    if [[ ${#issues[@]} -gt 0 ]]; then
        details=$(IFS=", "; echo "${issues[*]}")
    fi

    echo "$detected|$details"
}

# Pattern 4: Directory Name Mismatch
detect_path_mismatch() {
    local session_uuid="$1"
    local detected=false
    local details=""

    # Find project directory
    local project_file
    project_file=$(find "$CLAUDE_DIR/projects" -name "${session_uuid}.jsonl" 2>/dev/null | head -1)

    if [[ -z "$project_file" ]]; then
        echo "false|no project file found"
        return
    fi

    local project_dir
    project_dir=$(dirname "$project_file")

    # Extract encoded path from directory name
    local dir_name
    dir_name=$(basename "$project_dir")

    if [[ "$dir_name" == -* ]]; then
        # Decode expected path
        local expected_path
        expected_path=$(echo "${dir_name#-}" | sed 's|-|/|g')
        expected_path="/$expected_path"

        # Extract actual paths from session
        local actual_paths
        actual_paths=$(grep -o '"cwd":"[^"]*"' "$project_file" 2>/dev/null | sed 's/"cwd":"\([^"]*\)"/\1/' | sort -u)

        # Check if any actual path matches expected
        local match_found=false
        while IFS= read -r path; do
            if [[ "$path" == "$expected_path" ]]; then
                match_found=true
                break
            fi
        done <<< "$actual_paths"

        if [[ "$match_found" == false ]] && [[ -n "$actual_paths" ]]; then
            detected=true
            local first_path
            first_path=$(echo "$actual_paths" | head -1)
            details="Expected: $expected_path, Found: $first_path"
        fi
    fi

    echo "$detected|$details"
}

# Pattern 5: Orphaned Todos
detect_orphaned_todos() {
    local session_uuid="$1"
    local detected=false
    local details=""

    # Check if todo files exist without corresponding project
    local todo_count
    todo_count=$(ls "$CLAUDE_DIR/todos/${session_uuid}"-agent-*.json 2>/dev/null | wc -l)

    if [[ "$todo_count" -gt 0 ]]; then
        local project_exists
        project_exists=$(find "$CLAUDE_DIR/projects" -name "${session_uuid}.jsonl" 2>/dev/null | wc -l)

        if [[ "$project_exists" -eq 0 ]]; then
            detected=true
            details="$todo_count orphaned todo files"
        fi
    fi

    echo "$detected|$details"
}

# Pattern 6: Timestamp Drift
detect_timestamp_drift() {
    local session_uuid="$1"
    local detected=false
    local details=""

    # Find project file
    local project_file
    project_file=$(find "$CLAUDE_DIR/projects" -name "${session_uuid}.jsonl" 2>/dev/null | head -1)

    if [[ -z "$project_file" ]]; then
        echo "false|no project file"
        return
    fi

    # Find latest todo
    local latest_todo
    latest_todo=$(ls -t "$CLAUDE_DIR/todos/${session_uuid}"-agent-*.json 2>/dev/null | head -1)

    if [[ -n "$latest_todo" ]]; then
        # Get modification times
        local project_mtime todo_mtime
        if [[ "$OSTYPE" == "darwin"* ]]; then
            project_mtime=$(stat -f %m "$project_file")
            todo_mtime=$(stat -f %m "$latest_todo")
        else
            project_mtime=$(stat -c %Y "$project_file")
            todo_mtime=$(stat -c %Y "$latest_todo")
        fi

        # Calculate drift
        local time_diff=$((project_mtime - todo_mtime))
        local abs_diff=${time_diff#-}

        # Drift > 1 hour is suspicious
        if [[ $abs_diff -gt 3600 ]]; then
            detected=true
            local hours=$((abs_diff / 3600))
            details="Timestamp drift: ${hours}h"
        fi
    fi

    echo "$detected|$details"
}

# Validation functions from safety protocols

validate_process_safety() {
    local warnings=()

    # Check for active Claude processes
    local claude_procs
    claude_procs=$(pgrep -f "[Cc]laude" 2>/dev/null | wc -l) || claude_procs=0

    if [[ "$claude_procs" -gt 0 ]]; then
        warnings+=("$claude_procs active Claude processes detected")
    fi

    # Check for file locks
    local locked_files
    locked_files=$(lsof "$CLAUDE_DIR/projects"/*/*.jsonl 2>/dev/null | wc -l) || locked_files=0

    if [[ "$locked_files" -gt 0 ]]; then
        warnings+=("$locked_files session files locked by processes")
    fi

    if [[ ${#warnings[@]} -gt 0 ]]; then
        echo "warning|$(IFS="; "; echo "${warnings[*]}")"
    else
        echo "ok|"
    fi
}

validate_cross_system_state() {
    local session_uuid="$1"
    local errors=0
    local warnings=0
    local details=()

    # Project file check
    local project_count
    project_count=$(find "$CLAUDE_DIR/projects" -name "${session_uuid}.jsonl" 2>/dev/null | wc -l)

    if [[ "$project_count" -eq 0 ]]; then
        ((errors++))
        details+=("no project file")
    elif [[ "$project_count" -gt 1 ]]; then
        ((warnings++))
        details+=("$project_count project files")
    fi

    # Todo files check
    local todo_count
    todo_count=$(ls "$CLAUDE_DIR/todos/${session_uuid}"-agent-*.json 2>/dev/null | wc -l)

    if [[ "$project_count" -gt 0 && "$todo_count" -eq 0 ]]; then
        # Could be normal (archived session)
        details+=("no todos")
    fi

    # Construct result
    local status="ok"
    if [[ $errors -gt 0 ]]; then
        status="error"
    elif [[ $warnings -gt 0 ]]; then
        status="warning"
    fi

    echo "$status|$(IFS=", "; echo "${details[*]}")"
}

validate_path_consistency() {
    local session_uuid="$1"

    # Find project file
    local project_file
    project_file=$(find "$CLAUDE_DIR/projects" -name "${session_uuid}.jsonl" 2>/dev/null | head -1)

    if [[ -z "$project_file" ]]; then
        echo "error|no project file"
        return
    fi

    # Extract unique paths
    local unique_paths
    unique_paths=$(grep -o '"cwd":"[^"]*"' "$project_file" 2>/dev/null | sed 's/"cwd":"\([^"]*\)"/\1/' | sort -u | wc -l)

    if [[ "$unique_paths" -eq 1 ]]; then
        echo "ok|"
    elif [[ "$unique_paths" -gt 1 ]]; then
        echo "warning|$unique_paths different paths"
    else
        echo "ok|no paths found"
    fi
}

# Health score calculation
calculate_health_score() {
    local corruption_patterns="$1"
    local validation_warnings="$2"

    # Start at perfect score
    local score=100

    # Deduct points for each corruption pattern (weight: 20 points each)
    local pattern_count
    pattern_count=$(echo "$corruption_patterns" | grep -c "^true")
    score=$((score - (pattern_count * 20)))

    # Deduct points for validation warnings (weight: 10 points each)
    local warning_count
    warning_count=$(echo "$validation_warnings" | grep -c "^warning")
    score=$((score - (warning_count * 10)))

    # Ensure score doesn't go below 0
    if [[ $score -lt 0 ]]; then
        score=0
    fi

    echo "$score"
}

# Get severity classification
get_severity() {
    local score="$1"

    if [[ $score -ge 90 ]]; then
        echo "HEALTHY"
    elif [[ $score -ge 70 ]]; then
        echo "MINOR_ISSUES"
    elif [[ $score -ge 50 ]]; then
        echo "DEGRADED"
    elif [[ $score -ge 30 ]]; then
        echo "CORRUPTED"
    else
        echo "CRITICAL"
    fi
}

# Main diagnostic function
diagnose_session_health() {
    local session_uuid="${1:-current}"

    # Resolve "current" to actual session UUID if needed
    if [[ "$session_uuid" == "current" ]]; then
        # Try to detect current session from statsig
        session_uuid=$(cat "$CLAUDE_DIR/statsig/statsig.session_id."* 2>/dev/null | head -1)

        if [[ -z "$session_uuid" ]]; then
            _log_error "Cannot detect current session UUID"
            _log_info "Usage: cm diagnose <session-uuid>"
            return 1
        fi
    fi

    _log_info "=== Session Health Diagnosis ==="
    _log_info "Session UUID: $session_uuid"
    echo ""

    # Run all corruption pattern detections
    _log_info "Scanning for corruption patterns..."

    local patterns=()
    local pattern_results=""

    # Pattern 1: Branch collision
    local result
    result=$(detect_branch_collision "$session_uuid")
    IFS='|' read -r detected details <<< "$result"
    patterns+=("Branch Collision|$detected|$details")
    pattern_results="$pattern_results$detected\n"
    if [[ "$detected" == "true" ]]; then
        _log_warn "  ✗ Branch collision detected: $details"
    else
        _log_success "  ✓ No branch collision"
    fi

    # Pattern 2: Migration race
    result=$(detect_migration_race "$session_uuid")
    IFS='|' read -r detected details <<< "$result"
    patterns+=("Migration Race|$detected|$details")
    pattern_results="$pattern_results$detected\n"
    if [[ "$detected" == "true" ]]; then
        _log_warn "  ✗ Migration race detected: $details"
    else
        _log_success "  ✓ No migration race"
    fi

    # Pattern 3: Cross-system inconsistency
    result=$(detect_cross_system_inconsistency "$session_uuid")
    IFS='|' read -r detected details <<< "$result"
    patterns+=("Cross-System Inconsistency|$detected|$details")
    pattern_results="$pattern_results$detected\n"
    if [[ "$detected" == "true" ]]; then
        _log_warn "  ✗ Cross-system issues: $details"
    else
        _log_success "  ✓ Cross-system consistency OK"
    fi

    # Pattern 4: Path mismatch
    result=$(detect_path_mismatch "$session_uuid")
    IFS='|' read -r detected details <<< "$result"
    patterns+=("Path Mismatch|$detected|$details")
    pattern_results="$pattern_results$detected\n"
    if [[ "$detected" == "true" ]]; then
        _log_warn "  ✗ Path mismatch: $details"
    else
        _log_success "  ✓ Path consistency OK"
    fi

    # Pattern 5: Orphaned todos
    result=$(detect_orphaned_todos "$session_uuid")
    IFS='|' read -r detected details <<< "$result"
    patterns+=("Orphaned Todos|$detected|$details")
    pattern_results="$pattern_results$detected\n"
    if [[ "$detected" == "true" ]]; then
        _log_warn "  ✗ Orphaned todos: $details"
    else
        _log_success "  ✓ No orphaned todos"
    fi

    # Pattern 6: Timestamp drift
    result=$(detect_timestamp_drift "$session_uuid")
    IFS='|' read -r detected details <<< "$result"
    patterns+=("Timestamp Drift|$detected|$details")
    pattern_results="$pattern_results$detected\n"
    if [[ "$detected" == "true" ]]; then
        _log_warn "  ✗ Timestamp drift: $details"
    else
        _log_success "  ✓ Timestamps consistent"
    fi

    echo ""
    _log_info "Running validation checks..."

    local validations=()
    local validation_results=""

    # Validation 1: Process safety
    result=$(validate_process_safety)
    IFS='|' read -r status details <<< "$result"
    validations+=("Process Safety|$status|$details")
    validation_results="$validation_results$status\n"
    if [[ "$status" == "warning" ]]; then
        _log_warn "  ⚠ Process safety: $details"
    else
        _log_success "  ✓ Process safety OK"
    fi

    # Validation 2: Cross-system state
    result=$(validate_cross_system_state "$session_uuid")
    IFS='|' read -r status details <<< "$result"
    validations+=("Cross-System State|$status|$details")
    validation_results="$validation_results$status\n"
    if [[ "$status" == "error" ]]; then
        _log_error "  ✗ Cross-system state: $details"
    elif [[ "$status" == "warning" ]]; then
        _log_warn "  ⚠ Cross-system state: $details"
    else
        _log_success "  ✓ Cross-system state OK"
    fi

    # Validation 3: Path consistency
    result=$(validate_path_consistency "$session_uuid")
    IFS='|' read -r status details <<< "$result"
    validations+=("Path Consistency|$status|$details")
    validation_results="$validation_results$status\n"
    if [[ "$status" == "warning" ]]; then
        _log_warn "  ⚠ Path consistency: $details"
    elif [[ "$status" == "error" ]]; then
        _log_error "  ✗ Path consistency: $details"
    else
        _log_success "  ✓ Path consistency OK"
    fi

    echo ""

    # Calculate health score
    local health_score
    health_score=$(calculate_health_score "$pattern_results" "$validation_results")

    local severity
    severity=$(get_severity "$health_score")

    # Display results
    _log_info "=== Health Assessment ==="
    case "$severity" in
        "HEALTHY")
            _log_success "Health Score: ${health_score}/100 - HEALTHY ✅"
            _log_info "Session is in good condition"
            ;;
        "MINOR_ISSUES")
            _log_warn "Health Score: ${health_score}/100 - MINOR ISSUES ⚠"
            _log_info "Session has minor issues that should be monitored"
            ;;
        "DEGRADED")
            _log_warn "Health Score: ${health_score}/100 - DEGRADED ⚠"
            _log_info "Session has multiple issues that may affect functionality"
            _log_info "Recommendation: Consider running recovery procedures"
            ;;
        "CORRUPTED")
            _log_error "Health Score: ${health_score}/100 - CORRUPTED ❌"
            _log_info "Session has significant corruption"
            _log_info "Recommendation: Run REPAIR mode (when available)"
            ;;
        "CRITICAL")
            _log_error "Health Score: ${health_score}/100 - CRITICAL ❌"
            _log_info "Session is severely corrupted"
            _log_info "Recommendation: Manual intervention required"
            ;;
    esac

    echo ""

    # JSON output option
    if [[ "${DIAGNOSE_JSON:-false}" == "true" ]]; then
        echo ""
        _log_info "=== JSON Output ==="
        cat <<EOF
{
  "session_uuid": "$session_uuid",
  "health_score": $health_score,
  "severity": "$severity",
  "corruption_patterns": [
EOF
        local first=true
        for pattern in "${patterns[@]}"; do
            IFS='|' read -r name detected details <<< "$pattern"
            if [[ "$first" == true ]]; then
                first=false
            else
                echo ","
            fi
            echo -n "    {\"name\": \"$name\", \"detected\": $detected, \"details\": \"$details\"}"
        done
        cat <<EOF

  ],
  "validations": [
EOF
        first=true
        for validation in "${validations[@]}"; do
            IFS='|' read -r name status details <<< "$validation"
            if [[ "$first" == true ]]; then
                first=false
            else
                echo ","
            fi
            echo -n "    {\"name\": \"$name\", \"status\": \"$status\", \"details\": \"$details\"}"
        done
        cat <<EOF

  ]
}
EOF
    fi

    return 0
}
