# Handoff: finish `src/extract.rs` decomposition

## Objective

Finish the structural refactor requested in the extraction complexity issue. The target is a small barrel module with single-responsibility submodules and green Assay metrics for every resulting file.

## Current state

Commits already completed:

- `bea074e` — documented extraction helpers;
- `f09545b` — moved serialized record models to `src/extract/models.rs`;
- `580555f` — documented the extraction module boundary.

Working tree is clean. Tests pass.

Current command:

```text
assay score ./src/extract.rs --detail --all-metrics --no-colour
```

Current `src/extract.rs` score:

| Metric | Current |
|---|---:|
| LoC | 654 |
| Cyclomatic complexity | 128 |
| Cognitive complexity | 130 |
| Halstead volume | 6518 |
| NMI | 0.0 |

The model extraction reduced the file from 724 to 654 LoC, but the file remains well above every structural threshold. The goal is not merely to reduce the line count: move responsibilities into modules so Assay evaluates each file independently.

## Remaining boundaries

### 1. `src/extract/records.rs`

Move the `records` function and its record assembly helpers here. This module should own:

- reading and analysing one source file;
- constructing `FileRecord`, `Export`, and `SymbolRecord` values;
- converting records to `serde_json::Value`;
- import enrichment orchestration.

Keep its public surface narrow, preferably one `pub(super) fn records(...)` used by the barrel. Split record construction into small helpers if the file still exceeds the function/file thresholds. `records` is currently 139 LoC, CC 13, HV 1313 and is the largest immediate complexity hotspot.

### 2. `src/extract/imports.rs`

Move `import_records` and the import parsing logic here. Preserve the existing language behavior and tests, especially:

- grouped Rust imports;
- Python `from` and plain `import` forms;
- TypeScript/JavaScript imports;
- local import resolution and resolved symbol fields.

The current function is 90 LoC, CC 17, cognitive 28, HV 600. Do not simplify parsing by dropping fields or changing JSON output. Use focused helpers per language if needed.

### 3. `src/extract/metadata.rs`

Move metadata utilities here:

- `quoted_doc`;
- `language_name`;
- `module_path`;
- `qualified_name`;
- `class_kind`;
- `visibility`;
- `signature`;
- `is_exported`;
- `docstring`;
- `resolve_import`.

`docstring` is currently 61 LoC, CC 16, cognitive 30, HV 506. Separate Python docstrings, file-level docs, and preceding comment extraction into helpers while keeping one stable `docstring` entry point.

### 4. `src/extract/repository.rs` or barrel-owned API

Consider moving repository traversal and public query orchestration:

- `walk`;
- `supported`;
- `file_outline`;
- `file_descriptions`;
- `find`;
- `run`;
- `symbol_info`.

The final `src/extract.rs` should ideally only declare submodules, define `SymbolInfo`, and re-export or delegate the public API. Avoid leaving the 139-LoC `records` function or 90-LoC import parser in the barrel.

## Suggested implementation order

1. Create `src/extract/models.rs` is already complete; retain it.
2. Create `metadata.rs`; move helpers and update imports.
3. Create `imports.rs`; move `ImportRecord` usage and parsing tests.
4. Create `records.rs`; move record assembly and add a narrow `pub(super)` entry point.
5. Move traversal/public API into `repository.rs` if the barrel remains above 300 LoC.
6. Make `src/extract.rs` a documented barrel with minimal delegation.
7. Move tests alongside the behavior they verify, or keep only integration-facing tests in the barrel.

Use `fs_edit` for source changes. Do not discard unrelated work or reset the repository.

## Compatibility requirements

Preserve:

- `crate::extract::run`;
- `crate::extract::find`;
- `crate::extract::file_outline`;
- `crate::extract::file_descriptions`;
- `SymbolInfo` fields and visibility;
- serialized record shapes, field names, IDs, line ranges, import resolution values, and output order;
- existing CLI behavior for `extract` and `find`.

The current adapters and metrics APIs should remain unchanged.

## Documentation requirements

Follow `docs/code-style.md`:

- every module gets a `//!` responsibility description;
- every public item gets a concise `///` description;
- document non-obvious private helpers;
- describe responsibility and behavior, not implementation mechanics;
- keep descriptions short and stable.

Avoid duplicate prose between the barrel and submodules. The barrel should link responsibility through topology rather than restating implementation details.

## Verification checklist

Run after each cohesive boundary extraction:

```bash
cargo fmt --all
cargo test --quiet
cargo check --quiet
assay score ./src/extract.rs --detail --all-metrics --no-colour
assay score ./src/extract/* --detail --all-metrics --no-colour
```

Also run a behavior smoke test:

```bash
cargo run --quiet -- extract . >/dev/null
cargo run --quiet -- find records
```

Use `cargo clippy --all-targets --all-features -- -D warnings` only as supplemental feedback. The repository currently has unrelated pre-existing Clippy failures in `src/table.rs` and `src/main.rs`; do not broaden this task to fix them unless explicitly requested.

## Completion criteria

- No extraction source file is in the red LoC, CC, cognitive, or HV bands.
- No extraction function exceeds 60 LoC; no function has red complexity.
- All extraction modules have module documentation and public items are documented.
- `cargo test --quiet`, `cargo check --quiet`, formatting, and extraction smoke tests pass.
- Assay output and serialized extraction behavior remain compatible.
- Commit the completed refactor as one or more concise cohesive commits.
