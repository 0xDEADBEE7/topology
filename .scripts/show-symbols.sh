#!/usr/bin/env bash
set -euo pipefail

stash_dir=${STASH_DIR:-.stash}
query=${1:-}
command -v stash >/dev/null 2>&1 || { echo "stash is required; run make extract-store first" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "jq is required; run make extract-store first" >&2; exit 1; }

records=$(mktemp)
trap 'rm -f "$records"' EXIT
STASH_DIR="$stash_dir" stash get type=function >"$records"
STASH_DIR="$stash_dir" stash get type=class >>"$records"

jq -sr --arg query "$query" '
  [.[] | select(.name and .path and .lines)
   | select($query == "" or .name == $query or .qualified_name == $query or .path == $query)]
  | sort_by([.path, .lines[0], .name])
  | group_by([.path, .name])
  | map(max_by([(.lines[1] // 0), ((.kind // .type) == "struct"), ((.kind // .type) == "class")]))
  | .[]
  | [.name, (.kind // .type), (.path + ":" + (.lines[0] | tostring) + "-" + (.lines[1] | tostring)), (.signature // ""), (.description // "")]
  | @tsv
' "$records" | while IFS=$'\t' read -r symbol kind location signature description; do
    printf '%s  %s  %s\n' "$symbol" "$kind" "$location"
    printf '  %s\n' "$signature"
    [ -z "$description" ] || printf '  — %s\n' "$description"
  done
