# Locate a symbol definition

Use the indexed symbol records to find a function, class, or struct by name or qualified name:

```bash
query=compute
STASH_DIR=/path/to/repo/.stash stash get type=function type=class \
  | jq -sr --arg query "$query" '
      .[]
      | select(.name == $query or .qualified_name == $query)
      | [.name, (.kind // .type), .path,
         ((.lines[0] | tostring) + "-" + (.lines[1] | tostring)),
         (.signature // "")]
      | @tsv
    '
```

A qualified name is less ambiguous when a repository defines the same name more than once:

```bash
query=src/metrics::compute
STASH_DIR=/path/to/repo/.stash stash get type=function type=class \
  | jq -sr --arg query "$query" '
      .[]
      | select(.qualified_name == $query)
      | "\(.path):\(.lines[0])-\(.lines[1])  \(.signature)"
    '
```

To print the source range after locating it, keep the source checkout as the current directory:

```bash
STASH_DIR=/path/to/repo/.stash stash get type=function type=class \
  | jq -sr --arg query "$query" '
      .[] | select(.name == $query or .qualified_name == $query)
      | [.path, .lines[0], .lines[1]] | @tsv
    ' \
  | while IFS=$'\t' read -r path start end; do
      printf '%s:%s-%s\n' "$path" "$start" "$end"
      sed -n "${start},${end}p" "$path"
    done
```

The stored range is metadata from the extraction snapshot. Refresh the store after edits; otherwise the range can be stale. See [exports](./exports.md) for file-level public declarations and [imports](./imports.md) for dependencies.
