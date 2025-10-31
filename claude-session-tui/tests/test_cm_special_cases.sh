#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
CLAUDE_SCRIPT="${ROOT_DIR}/claude-manager.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

export HOME="${TMP_DIR}/home"
mkdir -p "${HOME}"

export CLAUDE_DIR="${HOME}/.claude"
export CLAUDE_INTERACTIVE="false"
export CLAUDE_DRY_RUN="false"
export FORCE="true"

mkdir -p "${CLAUDE_DIR}/projects"

encode_path() {
    local path="$1"
    local resolved
    resolved=$(python3 - <<'PY'
import os, sys
print(os.path.realpath(sys.argv[1]))
PY
"${path}")
    local encoded
    encoded=$(echo "${resolved/#\~/$HOME}" | sed 's|^/||' | sed 's|\.|-|g' | sed 's|/|-|g')
    printf '%s\n' "${CLAUDE_DIR}/projects/-${encoded}"
}

assert_contains() {
    local file="$1" needle="$2"
    if ! grep -Fq "$needle" "$file"; then
        echo "Expected '$needle' in $file" >&2
        exit 1
    fi
}

assert_not_contains() {
    local file="$1" needle="$2"
    if grep -Fq "$needle" "$file"; then
        echo "Did not expect '$needle' in $file" >&2
        exit 1
    fi
}

test_migrate_special_chars() {
    local old_path="${TMP_DIR}/source dir[1]"
    local new_path="${TMP_DIR}/new dir[1]"
    mkdir -p "${old_path}"

    local project_dir
    project_dir=$(encode_path "${old_path}")
    mkdir -p "${project_dir}"

    cat >"${project_dir}/session.jsonl" <<EOF
{"cwd":"${old_path}","thread_id":"test"}
{"cwd" : "${old_path}"}
EOF

    bash "${CLAUDE_SCRIPT}" migrate "${old_path}" "${new_path}" "${project_dir}" >/dev/null

    local migrated_project_dir
    migrated_project_dir="${CLAUDE_DIR}/projects/$(basename "${new_path}")"

    assert_contains "${migrated_project_dir}/session.jsonl" "\"cwd\":\"${new_path}\""
    assert_not_contains "${migrated_project_dir}/session.jsonl" "${old_path}"
}

test_move_and_undo_with_spaces() {
    local old_path="${TMP_DIR}/spaces dir"
    local new_path="${TMP_DIR}/dest dir"
    mkdir -p "${old_path}"

    local project_dir
    project_dir=$(encode_path "${old_path}")
    mkdir -p "${project_dir}"

    cat >"${project_dir}/session.jsonl" <<EOF
{"cwd":"${old_path}"}
EOF

    bash "${CLAUDE_SCRIPT}" move "${old_path}" "${new_path}" >/dev/null

    local moved_project_dir
    moved_project_dir=$(encode_path "${new_path}")

    [[ -d "${new_path}" ]] || { echo "Move failed" >&2; exit 1; }
    assert_contains "${moved_project_dir}/session.jsonl" "\"cwd\":\"${new_path}\""

    bash "${CLAUDE_SCRIPT}" undo >/dev/null

    local restored_project_dir
    restored_project_dir=$(encode_path "${old_path}")

    [[ -d "${old_path}" ]] || { echo "Undo did not restore source" >&2; exit 1; }
    assert_contains "${restored_project_dir}/session.jsonl" "\"cwd\":\"${old_path}\""
    assert_not_contains "${restored_project_dir}/session.jsonl" "${new_path}"
}

main() {
    test_migrate_special_chars
    test_move_and_undo_with_spaces
    echo "All cm special case tests passed"
}

main "$@"
