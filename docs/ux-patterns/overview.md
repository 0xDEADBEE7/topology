# Repository overview

Start with the file tree, including file-level descriptions extracted from doc comments or docstrings:

```bash
STASH_DIR=/path/to/repo/.stash stash get type=file \
  | jq -sr '
      map(select(.path))
      | sort_by(.path)
      | .[]
      | (.path + (if .description then "  — " + .description else "" end))
    '
```

A compact tree view with directory indentation:

```bash
STASH_DIR=/path/to/repo/.stash stash get type=file \
  | jq -sr '
      map(select(.path) | {path, description: (.description // "")})
      | sort_by(.path)
      | .[]
      | (.path + (if .description == "" then "" else "  # " + .description end))
    '
```

The second form is intentionally simple and works well as a first orientation. For the project-specific richer report, use the repository's `show-overview.sh` pattern; it combines files, symbols, metrics, public surface, and suggested reading ranges.

Continue with [exports](./exports.md) to inspect a file's API or [definitions](./definitions.md) to locate a symbol.
