#!/usr/bin/env bash
set -euo pipefail

stash_bin=${STASH_BIN:-stash}
stash_dir=${STASH_DIR:-.stash}
fig_bin=${FIG_BIN:-fig}
command -v jq >/dev/null 2>&1 || { echo "jq is required" >&2; exit 1; }
command -v "$stash_bin" >/dev/null 2>&1 || { echo "stash is required" >&2; exit 1; }
command -v "$fig_bin" >/dev/null 2>&1 || { echo "fig is required" >&2; exit 1; }

functions=$(STASH_DIR="$stash_dir" "$stash_bin" get type=function | jq -s '.')

figure=$(mktemp "${TMPDIR:-/tmp}/topo-hotspots.XXXXXX.json")
trap 'rm -f "$figure"' EXIT

jq -n --argjson functions "$functions" '
  ($functions
    | map(select(.name and .path and .metrics))
    | sort_by(-(.metrics.cognitive // 0), -(.metrics.cc // 0), -(.metrics.loc // 0))
    | .[0:20]
    | unique_by([.path, .name, .lines[0]])) as $rows
  | {
      type: "histogram",
      x_label: "function / file",
      y_label: "complexity",
      series: [
        {label: "cognitive"},
        {label: "cyclomatic"}
      ],
      buckets: ($rows | map({
        label: (.name + " — " + .path + ":" + ((.lines[0] // 0) | tostring)),
        values: {
          cognitive: (.metrics.cognitive // 0),
          cyclomatic: (.metrics.cc // 0)
        }
      }))
    }
' > "$figure"

"$fig_bin" "$figure" --interactive
