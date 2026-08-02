#!/usr/bin/env bash
set -euo pipefail

cargo test
smoke_output=$(cargo run --quiet -- tests/fixtures/typescript/basic.ts)
grep -F 'basic.ts [typescript]' <<< "$smoke_output"
grep -F 'classify (L1–7)' <<< "$smoke_output"
grep -F 'render (L9–11)' <<< "$smoke_output"
