# Project assessment

Assess TypeScript and JavaScript projects, including React components, with:

```bash
cargo run --quiet -- score /path/to/project/src
cargo run --quiet -- /path/to/project/src/App.tsx
cargo run --quiet -- extract /path/to/project
```

Supported extensions are `.rs`, `.py`, `.ts`, `.tsx`, `.mts`, `.cts`, `.js`,
`.jsx`, `.mjs`, and `.cjs`. TypeScript and TSX use separate grammars; JavaScript
uses the TSX grammar. React components, hooks, callbacks,
and class methods participate in the shared complexity reports. Assessment
does not require installing or executing the target project's dependencies.

Scoring and extraction share deterministic source discovery. Hidden entries,
dependency folders (`node_modules`, `vendor`), build folders (`target`, `dist`,
`build`, `bin`), and `coverage` are excluded. Symbolic links are not followed.
Explicitly named directories are scanned even if their name is normally excluded.
TypeScript with syntax errors is measured from tree-sitter's recovered tree; a
warning names the file and its error count, and extraction records it as
`parse_errors`. Every command skips a file or directory it cannot read or parse (Rust that
`syn` rejects) with a warning naming it; scoring fails only when no file could
be analysed.

The tree reads descriptions without parsing: the first paragraph of a leading
`//!`, `/*!` or `/** */` comment or Python docstring, up to its first `@` tag. It
skips plain `/* */` comments, license headers, `@` pragmas, reStructuredText
titles and directives such as `'use client'`. TypeScript and JavaScript score the
Docs column with the same reader.

`topo score` rates functions, not files. A function takes the worse of two bands
(`topo score --guide <path>` prints them):

| Metric | Green | Yellow | Red |
|---|---|---|---|
| Lines | ≤ 60 | 61–70 | > 70 |
| Cognitive complexity | ≤ 10 | 11–15 | > 15 |

Each file row shows its lines and whether it has a module doc, followed by its
yellow and red functions (`--detail` lists every function). A summary line counts
the yellow and red functions and the share of function lines they hold, plus doc
coverage, and `~` marks partially parsed files. Extracted function records carry `loc`, `cognitive`
and `band`. Lines are physical lines until roadmap M5 counts code lines, and
cognitive complexity still charges each `match`/`switch` arm until M3 and M5
apply the SonarSource rules.

Relative imports resolve existing TypeScript and JavaScript files and directory
indexes. Package imports and tsconfig aliases remain unresolved. This is
a structural assessment, not TypeScript type checking or React hook-rule linting.
The Python fallback remains approximate.

# Development checks

`make check` runs formatting verification, Clippy with warnings denied for all
targets, unit tests, CLI integration tests, and the TypeScript smoke test.
The integration suite extracts this project's `src` directory and enforces:

- every file has a module description, at most 400 lines and no parse errors;
- no function is red, and the share of function lines in yellow functions stays
  within `MAX_YELLOW_PER_MILLE`;
- `src` stays within the project budget in `tests/cli.rs`: files, total lines
  and dependencies;
- `tests/fixtures/typescript/basic.jsonl` pins the extract record shapes.

Complexity limits apply to functions only. File totals mostly measure size, and
gating them rewarded splitting cohesive code into many small files. The budget
and the yellow share are lowered as the [roadmap](./roadmap.md) shrinks the code
and are never raised to make topo pass. Documentation follows
[the code style](./code-style.md): describe module responsibilities, public
interfaces, and non-obvious helpers. Tests and routine trait implementations can
rely on their names and contracts.

`.scripts/corpus.sh` checks behaviour on real repositories pinned to fixed
commits: `fetch` clones them into `.local/corpus`, `run BIN OUT` records extract,
score and tree output with a one-line summary per repository, and `diff A B`
lists the files whose records changed between two binaries. Set
`IGNORE='del(.metrics.parse_errors)'` (any jq filter) to drop fields a milestone
changes on purpose before comparing.
