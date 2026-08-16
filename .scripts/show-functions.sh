#!/usr/bin/env bash
set -euo pipefail

stash_dir=${STASH_DIR:-.stash}
command -v stash >/dev/null 2>&1 || { echo "stash is required; run make extract-store first" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "jq is required to render the function index" >&2; exit 1; }

STASH_DIR="$stash_dir" stash get type=function | jq -sr '
  [.[] | select(.name and .path and .lines)]
  | sort_by([.path, .lines[0], .name])
  | group_by([.path, .name])
  | map(max_by(.lines[1]))
  | .[]
  | (.name + "  " + .path + ":" + (.lines[0] | tostring) + "-" + (.lines[1] | tostring))
  , ("  " + .signature)
  , (if .description then "  — " + .description else empty end)
'