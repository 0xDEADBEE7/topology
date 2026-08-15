# Inspect imports and local dependencies

Import records show source text, location, and whether the extractor resolved a local target:

```bash
file=src/main.rs
STASH_DIR=/path/to/repo/.stash stash get type=import \
  | jq -sr --arg file "$file" '
      .[]
      | select(.path == $file)
      | (.path + ":" + (.line | tostring)
         + "  " + (.source // "")
         + "  [" + (.resolution // "unresolved") + "]"
         + (if .resolved_path then " -> " + .resolved_path else "" end))
    '
```

Search all local links into a file:

```bash
STASH_DIR=/path/to/repo/.stash stash get type=import \
  | jq -sr --arg target "src/metrics.rs" '
      .[]
      | select(.resolved_path == $target)
      | [.path, .line, .source, (.resolved_symbol // "")]
      | @tsv
    '
```

Use this after [overview](./overview.md) to understand module boundaries. Use `assay find` to locate a symbol and pass the resulting path to direct inspection.

Resolution is adapter-dependent. Unresolved imports still remain useful as an external-dependency inventory.
