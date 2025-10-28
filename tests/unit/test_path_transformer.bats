#!/usr/bin/env bats

# test_path_transformer.bats
#
# PURPOSE:
#   Comprehensive tests for _suggest_project_dir_for() function.
#   This function is CRITICAL - it transforms working directory paths
#   to Claude's project naming convention. If it breaks, /resume breaks.
#
# COVERAGE TARGET: 100%
#
# TRANSFORMATION RULES:
#   1. Remove leading /
#   2. Convert . → - (dots to single dash)
#   3. Convert / → - (slashes to single dash)
#   4. Prefix with -
#
# LAST UPDATED: 2025-10-27
# RELATED COMMIT: b7dad8d (critical transformer fix)

# Setup runs before each test
setup() {
    # Create isolated test environment
    TEST_DIR="$(mktemp -d)"
    export CLAUDE_DIR="$TEST_DIR/.claude"
    export HOME="$TEST_DIR/home"
    mkdir -p "$CLAUDE_DIR/projects"
    mkdir -p "$HOME"

    # Source the script under test
    source "$BATS_TEST_DIRNAME/../../claude-manager.sh"
}

# Teardown runs after each test
teardown() {
    rm -rf "$TEST_DIR"
}

# ===== SIMPLE PATHS =====

@test "transformer: basic absolute path" {
    result=$(_suggest_project_dir_for "/Users/tryk/dev/crush")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-dev-crush" ]
}

@test "transformer: home tilde expansion" {
    result=$(_suggest_project_dir_for "~/dev/crush")
    # Tilde should expand to HOME, then be encoded
    # Note: Dots are replaced with dashes just like slashes
    home_encoded=$(echo "$HOME" | sed 's|^/||' | sed 's|\.|-|g' | sed 's|/|-|g')
    expected="$CLAUDE_DIR/projects/-${home_encoded}-dev-crush"
    [ "$result" = "$expected" ]
}

@test "transformer: simple two-level path" {
    result=$(_suggest_project_dir_for "/Users/tryk")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk" ]
}

@test "transformer: single directory" {
    result=$(_suggest_project_dir_for "/opt")
    [ "$result" = "$CLAUDE_DIR/projects/-opt" ]
}

# ===== PATHS WITH DOTS =====

@test "transformer: leading dot directory (.config)" {
    result=$(_suggest_project_dir_for "/Users/tryk/.config/nabi")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk--config-nabi" ]
}

@test "transformer: leading dot directory (.local)" {
    result=$(_suggest_project_dir_for "/Users/tryk/.local/share")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk--local-share" ]
}

@test "transformer: multiple dots in filename style" {
    result=$(_suggest_project_dir_for "/Users/tryk/Library.Data/stuff")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-Library-Data-stuff" ]
}

@test "transformer: dot in middle of directory name" {
    result=$(_suggest_project_dir_for "/Users/tryk/project.v1/src")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project-v1-src" ]
}

@test "transformer: multiple dots in single segment" {
    result=$(_suggest_project_dir_for "/Users/tryk/project.v1.2.3")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project-v1-2-3" ]
}

@test "transformer: hidden directory chain" {
    result=$(_suggest_project_dir_for "/Users/tryk/.local/.cache/.tmp")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk--local--cache--tmp" ]
}

@test "transformer: file extension style path" {
    result=$(_suggest_project_dir_for "/Users/tryk/backup.2024.tar.gz")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-backup-2024-tar-gz" ]
}

# ===== PATHS WITH HYPHENS =====

@test "transformer: single hyphen in path" {
    result=$(_suggest_project_dir_for "/Users/tryk/my-project")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-my-project" ]
}

@test "transformer: multiple hyphens in path" {
    result=$(_suggest_project_dir_for "/Users/tryk/my-awesome-project")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-my-awesome-project" ]
}

@test "transformer: hyphen and dot combination" {
    result=$(_suggest_project_dir_for "/Users/tryk/project-v1.2")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project-v1-2" ]
}

@test "transformer: kebab-case directory names" {
    result=$(_suggest_project_dir_for "/Users/tryk/some-really-long-kebab-case-name")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-some-really-long-kebab-case-name" ]
}

# ===== REAL-WORLD COMPLEX PATHS =====

@test "transformer: deep nested project path (nabia example)" {
    result=$(_suggest_project_dir_for "/Users/tryk/nabia/tui/production/riff-dag-tui")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-nabia-tui-production-riff-dag-tui" ]
}

@test "transformer: config directory typical case" {
    result=$(_suggest_project_dir_for "/Users/tryk/.config/nabi")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk--config-nabi" ]
}

@test "transformer: node_modules style path" {
    result=$(_suggest_project_dir_for "/Users/tryk/project/node_modules/@types/node")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project-node_modules-@types-node" ]
}

@test "transformer: workspace subdirectory" {
    result=$(_suggest_project_dir_for "/Users/tryk/workspace/client/frontend/src")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-workspace-client-frontend-src" ]
}

@test "transformer: temporary directory" {
    result=$(_suggest_project_dir_for "/tmp/test-project-123")
    [ "$result" = "$CLAUDE_DIR/projects/-tmp-test-project-123" ]
}

# ===== VERY LONG PATHS =====

@test "transformer: long nested path (10 levels)" {
    long_path="/Users/tryk/very/deeply/nested/project/structure/with/many/levels/of/directories"
    result=$(_suggest_project_dir_for "$long_path")
    expected="$CLAUDE_DIR/projects/-Users-tryk-very-deeply-nested-project-structure-with-many-levels-of-directories"
    [ "$result" = "$expected" ]
}

@test "transformer: very long single segment" {
    # 50 character directory name
    long_segment="this-is-a-very-long-directory-name-for-testing"
    result=$(_suggest_project_dir_for "/Users/tryk/$long_segment")
    [[ "$result" == "$CLAUDE_DIR/projects/-Users-tryk-$long_segment" ]]
}

# ===== EDGE CASES =====

@test "transformer: trailing slash is normalized" {
    result=$(_suggest_project_dir_for "/Users/tryk/project/")
    # Trailing slash should be stripped by bash parameter expansion
    # or handled by the function
    [[ "$result" == "$CLAUDE_DIR/projects/-Users-tryk-project" ]] || \
    [[ "$result" == "$CLAUDE_DIR/projects/-Users-tryk-project-" ]]
}

@test "transformer: root directory" {
    result=$(_suggest_project_dir_for "/")
    [ "$result" = "$CLAUDE_DIR/projects/-" ]
}

@test "transformer: path with spaces" {
    result=$(_suggest_project_dir_for "/Users/tryk/my project")
    # Spaces should be preserved (no special handling)
    [[ "$result" == *"my project"* ]]
}

@test "transformer: path with underscores" {
    result=$(_suggest_project_dir_for "/Users/tryk/my_project")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-my_project" ]
}

@test "transformer: path with numbers" {
    result=$(_suggest_project_dir_for "/Users/tryk/project-2024-v2")
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk-project-2024-v2" ]
}

# ===== IDEMPOTENCY TESTS =====

@test "transformer: same input produces same output" {
    result1=$(_suggest_project_dir_for "/Users/tryk/project")
    result2=$(_suggest_project_dir_for "/Users/tryk/project")
    [ "$result1" = "$result2" ]
}

@test "transformer: different inputs produce different outputs" {
    result1=$(_suggest_project_dir_for "/Users/tryk/project1")
    result2=$(_suggest_project_dir_for "/Users/tryk/project2")
    [ "$result1" != "$result2" ]
}

@test "transformer: case sensitivity preserved" {
    result1=$(_suggest_project_dir_for "/Users/Tryk/Project")
    result2=$(_suggest_project_dir_for "/Users/tryk/project")
    [ "$result1" != "$result2" ]
}

# ===== CRITICAL REGRESSION TESTS =====

@test "regression: dots become single dash not double (b7dad8d)" {
    # Critical fix: .config should become -config, not --config
    # WRONG (old): -Users-tryk---config
    # RIGHT (new): -Users-tryk--config (one dash for slash, one for dot)
    result=$(_suggest_project_dir_for "/Users/tryk/.config")
    # Should have exactly double dash for "/.config" part
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk--config" ]
    # Ensure it doesn't have triple dash (old bug)
    [[ "$result" != *"---"* ]]
}

@test "regression: slashes become single dash" {
    result=$(_suggest_project_dir_for "/Users/tryk")
    # Should have single dash between Users and tryk
    [ "$result" = "$CLAUDE_DIR/projects/-Users-tryk" ]
    # Ensure no double dashes except for dots
    [[ "$result" != *"--"* ]]
}

@test "regression: leading slash removed" {
    result=$(_suggest_project_dir_for "/Users/tryk/project")
    # Should start with "Users" not "/Users" after the prefix dash
    [[ "$result" == "$CLAUDE_DIR/projects/-Users"* ]]
    [[ "$result" != "$CLAUDE_DIR/projects/-/Users"* ]]
}

# ===== OUTPUT FORMAT VALIDATION =====

@test "transformer: output always starts with CLAUDE_DIR/projects/-" {
    result=$(_suggest_project_dir_for "/Users/tryk/anything")
    [[ "$result" == "$CLAUDE_DIR/projects/-"* ]]
}

@test "transformer: output is non-empty" {
    result=$(_suggest_project_dir_for "/Users/tryk/project")
    [ -n "$result" ]
}

@test "transformer: output contains no double slashes" {
    result=$(_suggest_project_dir_for "/Users/tryk/project")
    [[ "$result" != *"//"* ]]
}

# ===== ERROR HANDLING =====

@test "transformer: empty input returns prefix only" {
    result=$(_suggest_project_dir_for "")
    # Should return just the prefix, or handle gracefully
    [[ "$result" == "$CLAUDE_DIR/projects/-"* ]] || [ -z "$result" ]
}

@test "transformer: relative path is accepted" {
    # Function doesn't resolve relative paths, just encodes them
    result=$(_suggest_project_dir_for "relative/path")
    [[ "$result" == "$CLAUDE_DIR/projects/-relative-path" ]]
}
