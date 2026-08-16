# Create or refresh a repository store

Build the CLI, choose a store directory, then pipe JSONL extraction directly into `stash set`:

```bash
repo=/path/to/repo
store="$repo/.stash"

cargo build --release
mkdir -p "$store"

target/release/topo extract "$repo" \
  | STASH_DIR="$store" stash set
```

`topo extract` emits one JSON record per line. The extractor currently indexes Rust, Python, TypeScript, and TSX files while skipping hidden and generated directories.

Use the store explicitly for later queries:

```bash
export STASH_DIR="$repo/.stash"
```

Refresh the store after source changes by running the extraction pipeline again. See [overview](./overview.md), [exports](./exports.md), [definitions](./definitions.md), and [imports](./imports.md) for query patterns.

## Separate store

A store can live outside the repository:

```bash
repo=/path/to/repo
store="$HOME/.cache/topo/$(basename "$repo")"
mkdir -p "$store"
target/release/topo extract "$repo" \
  | STASH_DIR="$store" stash set
export STASH_DIR="$store"
```

Keep the store and source checkout aligned; stored records and line ranges are snapshots. For current exploration, prefer the live commands `topo PATH`, `topo PATH::SYMBOL`, and `topo find NAME`. Use the store for persistent or scriptable queries.
