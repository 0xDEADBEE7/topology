#!/usr/bin/env bash
set -euo pipefail

stash_dir=${STASH_DIR:-.stash}
command -v stash >/dev/null 2>&1 || { echo "stash is required; run make extract-store first" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "jq is required; run make extract-store first" >&2; exit 1; }

STASH_DIR="$stash_dir" stash get type=file type=function type=class | jq -sr '
  map(select(.path and .type))
  | sort_by([.type, .path, (.name // "")])
  | group_by([.type, .path, (.name // "")])
  | map(max_by((.lines[1] // .metrics.loc // 0)))
  | map(select((.description == null) or (.description == "")))
  | sort_by([-(.metrics.cognitive // 0), -(.metrics.cc // 0), -(.metrics.loc // 0), .path])
  | .[]
  | ((if .type == "file" then "FILE " else (.type | ascii_upcase) + " " end)
     + .path
     + (if .name then "::" + .name else "" end)
     + (if .lines then " L" + (.lines[0] | tostring) + "-" + (.lines[1] | tostring) else "" end)
     + (if .metrics then "  " + (.metrics.loc | tostring) + "L CC" + (.metrics.cc | tostring) else "" end))
'