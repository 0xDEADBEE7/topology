#!/usr/bin/env bash
set -euo pipefail

stash_dir=${STASH_DIR:-.stash}
command -v stash >/dev/null 2>&1 || { echo "stash is required; run make extract-store first" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "jq is required; run make extract-store first" >&2; exit 1; }

STASH_DIR="$stash_dir" stash get type=function | jq -sr '
  map(select(.type == "function" and .metrics != null))
  | sort_by([.path, .name, (.lines[0] // 0)])
  | group_by([.path, .name])
  | map(max_by(.lines[0] // 0))
  | sort_by([-(.metrics.cognitive // 0), -(.metrics.cc // 0), -(.metrics.loc // 0)])
  | .[0:12][]
  | ((if (.metrics.cognitive // 0) >= 10 or (.metrics.cc // 0) >= 8 then "HIGH" else "    " end)
     + "  " + .name
     + "  " + .path + ":" + (.lines[0] | tostring) + "-" + (.lines[1] | tostring)
     + "  " + ((.metrics.loc // 0) | tostring) + "L"
     + "  CC" + ((.metrics.cc // 0) | tostring)
     + "  Cog" + ((.metrics.cognitive // 0) | tostring)
     + (if .description then "  — " + .description else "" end))
'