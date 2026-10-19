#!/usr/bin/env bash
# Runs topo against pinned real-world repositories and compares runs.
#   corpus.sh fetch          clone the pinned corpus into $CORPUS_DIR (.local/corpus)
#   corpus.sh run BIN OUT    record extract, score and tree output plus a summary in OUT
#   corpus.sh diff A B       compare two runs; IGNORE='del(.field)' drops fields first
set -euo pipefail

corpus=${CORPUS_DIR:-.local/corpus}
repos=(
    "excalidraw https://github.com/excalidraw/excalidraw e1b7aacb356a492aa1e0fc6eb57b8896124027f0"
    "zustand https://github.com/pmndrs/zustand d7a5583cffd80af515f7dfb69583c95cbdc9e2ce"
    "bulletproof-react https://github.com/alan2207/bulletproof-react 9506629ed003a561c6627735480cce4994244bb4"
    "docusaurus https://github.com/facebook/docusaurus 7b6826b8476167008640cfb71105b9bb6c4f99f9"
    "requests https://github.com/psf/requests 611c6162cbc4ac2020a2f91c7cfa4f3abf9bbb60"
    "ripgrep https://github.com/BurntSushi/ripgrep 3fce3b5bb0236da2df6d99672afb8a719642eca7"
)

fetch() {
    for entry in "${repos[@]}"; do
        read -r name url commit <<< "$entry"
        [[ $(git -C "$corpus/$name" rev-parse -q --verify HEAD 2>/dev/null) == "$commit" ]] && continue
        git init -q "$corpus/$name"
        git -C "$corpus/$name" fetch -q --depth 1 "$url" "$commit"
        git -C "$corpus/$name" checkout -q FETCH_HEAD
    done
}

# Runs one topo command, saving stdout, stderr, seconds and "ok"/"abort".
timed() {
    local out=$1; shift
    local TIMEFORMAT=%R status=ok
    { time "$@" > "$out.out" 2> "$out.err" || status=abort; } 2> "$out.seconds"
    echo "$status"
}

stats() {
    jq -rs '
      (map(select(.type == "function"))) as $fns
      | ((map(.id) | length) - (map(.id) | unique | length)) as $dups
      | "files=\(map(select(.type == "file")) | length)"
        + " partial=\(map(select(.type == "file" and (.metrics.parse_errors // 0) > 0)) | length)"
        + " functions=\($fns | length)"
        + " anonymous=\($fns | map(select(.name == "<anonymous>")) | length)"
        + " bogus_imports=\(map(select(.type == "import" and ([.source, .imported_name]
            | map(. // "" | test("[{}(),\\s]|[.:]$")) | any))) | length)"
        + " duplicate_ids=\($dups)"
        + " empty_descriptions=\(map(select(.type == "file" and .description == "")) | length)"' "$1"
}

run() {
    local bin=$1 out=$2
    mkdir -p "$out"
    echo "binary_bytes=$(wc -c < "$bin" | tr -d ' ')" > "$out/summary.txt"
    for entry in "${repos[@]}"; do
        read -r name _ <<< "$entry"
        local dir="$corpus/$name" base="$out/$name"
        local extract score tree
        extract=$(timed "$base.extract" "$bin" extract "$dir")
        score=$(timed "$base.score" "$bin" score "$dir")
        tree=$(timed "$base.tree" "$bin" "$dir")
        printf '%s extract=%s score=%s tree=%s stderr_lines=%s %s seconds=%s/%s/%s\n' \
            "$name" "$extract" "$score" "$tree" "$(wc -l < "$base.extract.err" | tr -d ' ')" \
            "$(stats "$base.extract.out")" "$(cat "$base.extract.seconds")" \
            "$(cat "$base.score.seconds")" "$(cat "$base.tree.seconds")" >> "$out/summary.txt"
    done
    cat "$out/summary.txt"
}

compare() {
    local a=$1 b=$2
    diff "$a/summary.txt" "$b/summary.txt" || true
    for entry in "${repos[@]}"; do
        read -r name _ <<< "$entry"
        diff <(jq -c "${IGNORE:-.}" "$a/$name.extract.out" | sort) \
            <(jq -c "${IGNORE:-.}" "$b/$name.extract.out" | sort) \
            | sed -n 's/^[<>] //p' | jq -r '.path' | sort -u > "$b/$name.changed" || true
        echo "$name: $(wc -l < "$b/$name.changed" | tr -d ' ') files with changed records ($b/$name.changed)"
    done
}

case ${1:-} in
    fetch) fetch ;;
    run) run "$2" "$3" ;;
    diff) compare "$2" "$3" ;;
    *) sed -n '2,5p' "$0" >&2; exit 2 ;;
esac
