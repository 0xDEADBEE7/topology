# User patterns for fast code navigation

Use Topo to reduce the search space before reading source:

```text
repository → directory → file outline → symbol → implementation
```

## Start with the repository shape

```bash
topo .
topo ./src
make overview
```

Use `topo .` for the broad map and `topo ./src` for production code. Use `make overview` when you want prioritized files and the **Read these ranges first** section.

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
topo ./src/extract/metadata.rs::resolve_import
topo ./tests/fixtures/typescript/basic.ts::classify
```

This prints the signature, source range, and implementation. Use the range as the edit target; inspect a small surrounding window only when callers or nearby helpers matter.

## Follow a concept through naming variants

Search is exact-name oriented, so try likely compound names:

```bash
topo find import
topo find imports
topo find resolve_import
topo find parse_line
```

A broad concept may not match `resolve_import`. If the name is unknown, inspect the containing directory or file outline and choose from its symbols.

## Check public boundaries

Use the **Public surface** section from `make overview`, then inspect the relevant file and symbol:

```bash
topo ./src/adapters/mod.rs
topo ./src/adapters/mod.rs::for_path
```

This distinguishes exported entry points from private implementation details.

## Check cross-file dependencies

With a stored repository index:

```bash
make imports FILE=src/extract/metadata.rs
make imports FILE=src/main.rs
```

Use this to see local and unresolved dependencies. Refresh the store after source edits; live `topo PATH` and `topo find NAME` provide current ranges.

## Use complexity to prioritize reading

```bash
make hotspots-report
topo score --detail ./src/extract/metadata.rs
```

A hotspot is a prioritization hint, not proof of a defect. It is a useful first read for control-flow bugs and refactors.

## Task recipes

| Task | Fast route |
|---|---|
| Find a feature | `topo find NAME`, then `topo PATH::NAME` |
| Understand a subsystem | `topo ./src/SUBSYSTEM`, then inspect one file |
| Fix a parser or resolver | `make overview`, then inspect the relevant ranges |
| Trace a public API | **Public surface** → file outline → exported symbol |
| Understand a fixture | `topo ./tests`, then `topo FILE::TEST_FUNCTION` |
| Find refactor targets | `make hotspots-report`, then inspect the ranges |

## Keep output high-signal

- Prefer outlines over whole-file dumps.
- Run one narrow command at a time and use its path/range as the next command.
- Unsupported files such as Markdown can appear in repository context, but direct inspection is for supported source languages.
- If a symbol is not found, check spelling, try compound-name variants, or inspect the containing file outline.
