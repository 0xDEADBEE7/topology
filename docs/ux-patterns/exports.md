# Inspect a file's exported surface

File records contain an `exports` array for declarations recognized as public by the language adapter:

```bash
file=src/metrics.rs
STASH_DIR=/path/to/repo/.stash stash get type=file \
  | jq -sr --arg file "$file" '
      [.[] | select(.path == $file)]
      | .[0].exports // []
      | .[]
      | (.signature + "  L" + (.lines[0] | tostring) + "-" + (.lines[1] | tostring))
    '
```

To inspect every exported file and declaration:

```bash
STASH_DIR=/path/to/repo/.stash stash get type=file \
  | jq -sr '
      .[]
      | select((.exports // []) | length > 0)
      | .path as $path
      | .exports[]
      | ($path + ":" + (.lines[0] | tostring) + "-" + (.lines[1] | tostring) + "  " + .signature)
    '
```

Use the symbol index when you need searchable names, qualified names, documentation, or metrics rather than only a file's public surface. For live exploration, prefer the direct CLI:

```bash
assay ./src/metrics.rs
assay find compute
```

For persistent queries, the extracted symbol records remain available through `stash`:

```bash
STASH_DIR=/path/to/repo/.stash stash get type=function type=class \
  | jq -sr --arg file "$file" '
      .[] | select(.path == $file)
      | [.name, (.kind // .type), (.qualified_name // ""), (.signature // "")]
      | @tsv
    '
```

The current extractor indexes functions and classes/structs. Variable-level exports are not yet emitted as symbols; treat the file `exports` array as adapter-defined rather than a complete language API model. See [definitions](./definitions.md) for locating any indexed declaration.
