#!/usr/bin/env bash
set -euo pipefail

stash_dir=${STASH_DIR:-.stash}
query=${1:-}
[ -n "$query" ] || { echo "usage: $0 PATH::NAME | NAME" >&2; exit 2; }
command -v stash >/dev/null 2>&1 || { echo "stash is required; run make extract-store first" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "jq is required; run make extract-store first" >&2; exit 1; }

records=$(mktemp)
trap 'rm -f "$records"' EXIT
STASH_DIR="$stash_dir" stash get type=function >"$records"
STASH_DIR="$stash_dir" stash get type=class >>"$records"

jq -sr --arg query "$query" '
  [.[] | select(.name and .path and .lines)
   | select(.qualified_name == $query or .name == $query or (.path + "::" + .name) == $query)]
  | sort_by([.path, .lines[0], .name])
  | group_by([.path, .name, (.kind // .type)])
  | map(max_by(.lines[1]))
  | .[]
  | [.path, (.lines[0] | tostring), (.lines[1] | tostring), .name,
     (.kind // .type), (.signature // ""), (.description // "")]
  | @tsv
' "$records" | while IFS=$'\t' read -r path start end name kind signature description; do
    printf '%s  %s:%s-%s\n' "$name" "$path" "$start" "$end"
    printf 'kind: %s\n' "$kind"
    printf 'signature: %s\n' "$signature"
    [ -z "$description" ] || printf 'documentation: %s\n' "$description"
    printf '\n'
    sed -n "${start},${end}p" "$path"
    printf '\n'
  done
