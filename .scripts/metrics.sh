#!/usr/bin/env bash
set -euo pipefail

METRICS_BIN="target/release/assay"
RED='\033[0;31m'; YELLOW='\033[1;33m'; GREEN='\033[0;32m'; RESET='\033[0m'

# ── Section 1: LoC breakdown ──────────────────────────────────────────────────
echo "=== Lines of Code ==="
if command -v cloc &>/dev/null; then
    echo "--- production ---"
    cloc src/ --quiet --exclude-dir=tests 2>/dev/null || true
    echo "--- tests ---"
    cloc src/ --quiet --include-lang=Rust --match-f='.*test.*' 2>/dev/null || true
else
    echo "cloc not found — skipping LoC breakdown"
fi

# ── Section 2: per-file size warnings ────────────────────────────────────────
echo ""
echo "=== File Size ==="
red_count=0
while IFS= read -r -d '' f; do
    lines=$(wc -l < "$f")
    if (( lines >= 300 )); then
        printf "${RED}[RED]${RESET}    %4d lines  %s\n" "$lines" "$f"
        (( red_count++ )) || true
    elif (( lines >= 200 )); then
        printf "${YELLOW}[YELLOW]${RESET} %4d lines  %s\n" "$lines" "$f"
    fi
done < <(find src -name '*.rs' -print0 2>/dev/null)

if (( red_count > 0 )); then
    printf "${RED}%d file(s) exceed 300 lines — refactor before adding code.${RESET}\n" "$red_count"
else
    printf "${GREEN}All files within threshold.${RESET}\n"
fi

# ── Section 3: binary size ────────────────────────────────────────────────────
echo ""
echo "=== Binary Size ==="
if [[ -f "$METRICS_BIN" ]]; then
    size=$(wc -c < "$METRICS_BIN")
    printf "%s  %s bytes  (%s)\n" "$METRICS_BIN" "$size" "$(du -sh "$METRICS_BIN" | cut -f1)"
else
    echo "No release binary found — run 'make build' first."
fi

# ── Section 4: AST complexity ─────────────────────────────────────────────────
echo ""
echo "=== Complexity ==="
if [[ ! -f "$METRICS_BIN" ]]; then
    echo "Building assay..."
    cargo build --release -q
fi

# Pass all source files in one invocation so assay renders a single table
src_files=()
while IFS= read -r -d '' f; do
    src_files+=("$f")
done < <(find src -name '*.rs' -print0 2>/dev/null)
if (( ${#src_files[@]} > 0 )); then
    "$METRICS_BIN" "${src_files[@]}"
fi
