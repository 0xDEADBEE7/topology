#!/usr/bin/env bash
set -euo pipefail

stash_dir=${STASH_DIR:-.stash}
command -v stash >/dev/null 2>&1 || { echo "stash is required; run make extract-store first" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "jq is required; run make extract-store first" >&2; exit 1; }

jq -s -r '
  flatten
  | map(select(.path and .type))
  | sort_by([.type, .path, (.name // "")])
  | group_by([.type, .path, (.name // "")])
  | map(max_by((.lines[1] // .metrics.loc // 0))) as $records
  | ($records | map(select(.type == "file"))) as $files
  | ($records | map(select(.type == "function" or .type == "class"))) as $symbols
  | "Repository orientation",
    "======================",
    ((($files | length) | tostring) + " files · " + (($symbols | length) | tostring) + " indexed symbols"),
    "",
    "Inspection priorities (low NMI or high complexity first)",
    "--------------------------------------------------------",
    ($files | sort_by([(.metrics.nmi // 100), -(.metrics.cc // 0), -(.metrics.loc // 0)]) | .[] |
      ((if (.metrics.nmi // 100) < 20 then "RED" elif (.metrics.nmi // 100) < 30 then "YELLOW" else "    " end)
       + " " + .path
       + "  " + ((.metrics.loc // 0) | tostring) + "L"
       + "  CC" + ((.metrics.cc // 0) | tostring)
       + "  NMI" + ((.metrics.nmi // 0) | floor | tostring)
       + (if .description then "  — " + .description else "" end))),
    "",
    "Public surface",
    "--------------",
    ($files | map(select((.exports // []) | length > 0)) | .[] |
      (.path + "  " + ((.exports | map(.signature) | join("; "))))),
    "",
    "Read these ranges first",
    "-----------------------",
    ($files | sort_by([-(.metrics.cc // 0), -(.metrics.loc // 0)]) | .[0:5] |
      .[] as $file | ($file.path) as $path |
      ($symbols | map(select(.path == $path and .metrics))
        | sort_by([-(.metrics.cognitive // 0), -(.metrics.cc // 0), -(.metrics.loc // 0)])
        | .[0:3]
        | map((.name + " L" + (.lines[0] | tostring) + "-" + (.lines[1] | tostring)))
        | join(", ")) as $ranges
      | ($path + ": " + $ranges))
' <(STASH_DIR="$stash_dir" stash get type=file) <(STASH_DIR="$stash_dir" stash get type=function)
