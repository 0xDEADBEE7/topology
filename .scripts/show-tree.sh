#!/usr/bin/env bash
set -euo pipefail

scope=${1:-repo}
repo=${REPO:-.}
topo_bin=${TOPO_BIN:-target/release/topo}

if ! command -v jq >/dev/null 2>&1; then
    echo "jq is required to render the topo tree" >&2
    exit 1
fi
if ! command -v "$topo_bin" >/dev/null 2>&1 && [ ! -x "$topo_bin" ]; then
    echo "topo is required; build it or set TOPO_BIN" >&2
    exit 1
fi

case "$scope" in
    project|repo) ;;
    *) echo "usage: $0 [project|repo]" >&2; exit 2 ;;
esac

"$topo_bin" extract "$repo" | jq -sr --arg scope "$scope" '
  [.[] | select(.type == "file" and .path) | select($scope == "repo" or (.path | startswith("src/") or startswith("tests/") or (contains("/") | not))) | {path, desc: (.description // ""), parts: (.path | split("/"))}]
  | sort_by(.path)
  | reduce .[] as $file (
      {out: [], seen: {}};
      ($file.parts[0:-1]) as $dirs
      | ($dirs | length) as $depth
      | reduce range(0; $depth) as $i (
          .;
          ($dirs[0:($i + 1)] | join("/")) as $dir
          | if .seen[$dir] then . else .seen[$dir] = true | .out += [("  " * $i) + $dirs[$i] + "/"] end
        )
      | .out += [
          (("  " * $depth) + $file.parts[-1])
          + (if $file.desc == "" then "" else (" " * ([40 - (2 * $depth) - ($file.parts[-1] | length), 1] | max)) + "# " + $file.desc end)
        ]
    )
  | .out
  | join("\n")
'