# Project assessment

Assess TypeScript projects, including React components, with:

```bash
cargo run --quiet -- score /path/to/project/src --detail --all-metrics --no-colour
cargo run --quiet -- /path/to/project/src/App.tsx
cargo run --quiet -- extract /path/to/project
```

Supported extensions are `.rs`, `.py`, `.ts`, `.tsx`, `.mts`, and `.cts`.
TypeScript and TSX use separate grammars. React components, hooks, callbacks,
and class methods participate in the shared complexity reports. Assessment
does not require installing or executing the target project's dependencies.

Scoring and extraction share deterministic source discovery. Hidden entries,
dependency folders (`node_modules`, `vendor`), build folders (`target`, `dist`,
`build`, `bin`), and `coverage` are excluded. Symbolic links are not followed.
Explicitly named directories are scanned even if their name is normally excluded.
Scoring fails on unreadable or invalid supported source and on empty inputs.
Extraction retains its existing best-effort behavior and reports parse errors
on stderr while emitting records for the other files.

Relative imports resolve existing TypeScript files and directory indexes,
including TSX. Package imports and tsconfig aliases remain unresolved. This is
a structural assessment, not TypeScript type checking or React hook-rule linting.
The Python fallback and Halstead token counts remain approximate.

# Development checks

`make check` runs formatting verification, Clippy with warnings denied for all
targets, unit tests, CLI integration tests, and the TypeScript smoke test.
The integration suite extracts this project's `src` directory and enforces:

- every file has a module description and at most 400 lines;
- every function stays at or below 60 lines and below the red boundaries for CC
  and cognitive complexity (15) and Halstead volume (1000);
- `src` stays within the project budget in `tests/cli.rs`: files, total lines
  and dependencies.

Complexity limits apply to functions only. File totals mostly measure size, and
gating them rewarded splitting cohesive code into many small files. The budget
is lowered as the [roadmap](./roadmap.md) shrinks the code and is never raised to
make topo pass. Yellow bands remain advisory. Documentation follows
[the code style](./code-style.md): describe module responsibilities, public
interfaces, and non-obvious helpers. Tests and routine trait implementations can
rely on their names and contracts.

`.scripts/corpus.sh` checks behaviour on real repositories pinned to fixed
commits: `fetch` clones them into `.local/corpus`, `run BIN OUT` records extract,
score and tree output with a one-line summary per repository, and `diff A B`
lists the files whose records changed between two binaries.
