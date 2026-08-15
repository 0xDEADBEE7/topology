#!/usr/bin/env bash
set -euo pipefail

stash_dir=${STASH_DIR:-.stash}
path=${1:-}
command -v stash >/dev/null 2>&1 || { echo "stash is required; run make extract-store first" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "jq is required; run make extract-store first" >&2; exit 1; }

STASH_DIR="$stash_dir" stash get type=import | jq -sr --arg path "$path" '
  [.[] | select(.path and (.path == $path or $path == ""))]
  | sort_by([.path, .line, .local_name])
  | group_by([.path, .line, .local_name, .source])
  | map(max_by(.id))
  | .[]
  | [.path, (.line | tostring), (.local_name // ""), (.source // ""), (.resolved_symbol // ""), (.resolution // "unresolved")]
  | @tsv
' | while IFS=$'\t' read -r file line local source target resolution; do
    printf '%s:%s  %s  %s\n' "$file" "$line" "${local:-—}" "$source"
    [ -z "$target" ] || printf '  -> %s\n' "$target"
    printf '  [%s]\n' "$resolution"
  done
