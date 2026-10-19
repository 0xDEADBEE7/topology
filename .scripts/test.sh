#!/usr/bin/env bash
set -euo pipefail

cargo test

# Every `$ topo …` line in a README console block must print exactly the lines below it.
topo() { cargo run --quiet -- "$@"; }
check() {
    local actual
    actual=$(eval "$1" 2>/dev/null) || { echo "README command failed: $1" >&2; exit 1; }
    if [[ "$actual" != "$2" ]]; then
        echo "README output differs for: $1" >&2
        diff <(printf '%s\n' "$2") <(printf '%s\n' "$actual") >&2
        exit 1
    fi
}
command="" expected=""
while IFS= read -r line; do
    case "$line" in
        '$ '*) [[ -z $command ]] || check "$command" "$expected"
               command=${line#'$ '} expected="" ;;
        '```') [[ -z $command ]] || check "$command" "$expected"
               command="" ;;
        *) expected+="${expected:+$'\n'}$line" ;;
    esac
done < <(awk '/^```console$/ {inside = 1; next} inside {print} /^```$/ {inside = 0}' README.md)
