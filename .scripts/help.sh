#!/usr/bin/env bash
set -euo pipefail

printf "%-10s %s\n" "Target"  "Description"
printf "%-10s %s\n" "------"  "-----------"
printf "%-10s %s\n" "build"   "cargo build --release"
printf "%-10s %s\n" "test"    "cargo test"
printf "%-10s %s\n" "lint"    "cargo clippy -- -D warnings"
printf "%-10s %s\n" "check"   "lint + test (CI gate)"
printf "%-10s %s\n" "metrics" "code health report"
printf "%-10s %s\n" "clean"   "cargo clean"
