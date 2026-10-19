#!/usr/bin/env bash
set -euo pipefail

printf "%-16s %s\n" "Target"    "Description"
printf "%-16s %s\n" "------"    "-----------"
printf "%-16s %s\n" "build"     "cargo build --release"
printf "%-16s %s\n" "test"      "cargo test plus every README command"
printf "%-16s %s\n" "lint"      "cargo fmt --check and clippy -D warnings"
printf "%-16s %s\n" "check"     "lint + test (CI gate)"
printf "%-16s %s\n" "extract"   "extract metadata as raw JSONL (REPO=dir)"
printf "%-16s %s\n" "metrics"   "binary size and code health report"
printf "%-16s %s\n" "tree"      "show topo-extracted project file tree with descriptions"
printf "%-16s %s\n" "repo-tree" "show the tree for the whole repository"
printf "%-16s %s\n" "clean"     "cargo clean"

if [[ -x .local/scripts/help.sh ]]; then
    .local/scripts/help.sh
fi
