# User patterns for fast code navigation

Use Topo to reduce the search space before reading source:

```text
repository → directory → file outline → symbol → implementation
```

## Start with the repository shape

```bash
topo .
topo ./src
```

Use `topo .` for the broad map and `topo ./src` for production code.

## Narrow to a file

```bash
topo ./src/extract
topo ./src/extract/metadata.rs
```

The file view shows its description, symbols, and line ranges without dumping the implementation. Pick one or two likely symbols, then inspect only those.

## Find a definition

```bash
topo find resolve_import
topo find FileMetrics
topo find analyse
```

Search returns the path and line range for functions, structs, and classes. For an exact path-qualified lookup, copy the path shown by the result:

```bash
topo find src/metrics::FileMetrics
topo ./src/metrics.rs::FileMetrics
```

## Inspect one implementation

```bash
topo ./src/extract/resolution.rs::resolve_import
topo ./tests/fixtures/typescript/basic.ts::classify
```

This prints the signature, source range, and implementation. Use the range as the edit target; inspect a small surrounding window only when callers or nearby helpers matter.

## Follow a concept through naming variants

Search is exact-name oriented, so try likely compound names:

```bash
topo find import
topo find imports
topo find resolve_import
topo find parse
```

A broad concept may not match `resolve_import`. If the name is unknown, inspect the containing directory or file outline and choose from its symbols.

## Check public boundaries

Inspect the module that registers entry points, then the relevant symbol:

```bash
topo ./src/adapters/mod.rs
topo ./src/adapters/mod.rs::for_path
```

This distinguishes exported entry points from private implementation details.

## Check cross-file dependencies

```bash
topo extract . | jq -c 'select(.type == "import" and .path == "src/cli/mod.rs")'
```

Each import record shows its source, any locally resolved path, and whether it resolved.

## Use complexity to prioritize reading

```bash
topo score --detail --all-metrics ./src
topo score --detail --all-metrics ./src/extract/resolution.rs
```

A hotspot is a prioritization hint, not proof of a defect. It is a useful first read for control-flow bugs and refactors.

## Task recipes

| Task | Fast route |
|---|---|
| Find a feature | `topo find NAME`, then `topo PATH::NAME` |
| Understand a subsystem | `topo ./src/SUBSYSTEM`, then inspect one file |
| Fix a parser or resolver | `topo score --detail ./src`, then inspect the relevant ranges |
| Trace a public API | file outline → exported symbol |
| Understand a fixture | `topo ./tests`, then `topo FILE::TEST_FUNCTION` |
| Find refactor targets | `topo score --detail --all-metrics ./src`, then inspect the ranges |

## Keep output high-signal

- Prefer outlines over whole-file dumps.
- Run one narrow command at a time and use its path/range as the next command.
- Unsupported files such as Markdown can appear in repository context, but direct inspection is for supported source languages.
- If a symbol is not found, check spelling, try compound-name variants, or inspect the containing file outline.
