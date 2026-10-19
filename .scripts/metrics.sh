#!/usr/bin/env bash
set -euo pipefail

METRICS_BIN="${TOPO_BIN:-target/release/topo}"

echo "=== Binary Size ==="
printf "%s  %s bytes  (%s)\n" "$METRICS_BIN" "$(wc -c < "$METRICS_BIN" | tr -d ' ')" "$(du -sh "$METRICS_BIN" | cut -f1)"

echo ""
echo "=== Code Health ==="
"$METRICS_BIN" score src --detail
