# Project assessment

Assess TypeScript and JavaScript projects, including React components, with:

```bash
cargo run --quiet -- score /path/to/project/src
cargo run --quiet -- /path/to/project/src/App.tsx
cargo run --quiet -- extract /path/to/project
```

Supported extensions are `.rs`, `.py`, `.ts`, `.tsx`, `.mts`, `.cts`, `.js`,
`.jsx`, `.mjs`, and `.cjs`. TypeScript and TSX use separate grammars; JavaScript
uses the TSX grammar. React components are named by their bindings (including
through the first argument of `forwardRef` and `memo`), class members read `Class.method`, a
callback passed after a string label becomes a unit named after its call, such as
`it("undoes")` or `app.get("/users")`, and other unnamed callbacks fold into the
function that contains them. Assessment
does not require installing or executing the target project's dependencies.

Scoring and extraction share deterministic source discovery. Hidden entries,
dependency folders (`node_modules`, `vendor`), build folders (`target`, `dist`,
`build`, `bin`), and `coverage` are excluded. Symbolic links are not followed.
Explicitly named directories are scanned even if their name is normally excluded.
Every language is parsed with tree-sitter. A file with syntax errors is measured
from the recovered tree; a warning names the file and its error count, and
extraction records it as `parse_errors`. Every command skips a file or directory
it cannot read or decode with a warning naming it (the tree leaves it out); scoring
fails only when no file could be analysed.

Descriptions come from the doc node the parser finds. A file's is its module
docstring or the `//!`, `/*!` and `/** */` blocks before its first code, past
shebangs, inner attributes and directives such as `'use client'`. A declaration's
is the doc comment above it, past attributes, decorators and plain comments, or
the docstring opening its body (never an f-string or bytes literal). A description
is the first paragraph of prose up to its first `@` tag: plain comments, tag-only
JSDoc, license blocks and reStructuredText titles describe nothing. The score's
Doc column marks exactly the files and functions that have one.

`topo score` rates functions, not files. A function takes the worse of two bands
(`topo score --guide <path>` prints them):

| Metric | Green | Yellow | Red |
|---|---|---|---|
| Code lines | ≤ 40 | 41–75 | > 75 |
| Cognitive complexity | ≤ 10 | 11–15 | > 15 |

Each file row shows its lines and whether it has a module doc, followed by its
yellow and red functions (`--detail` lists every function). A summary line counts
the yellow and red functions and the share of function lines they hold, plus doc
coverage, and `~` marks partially parsed files. Extracted function records carry `loc`, `cognitive`
and `band`, and every declaration record its `signature` (up to its body), `parent`
and `exported`: `export` or an `export { }`/`export default` naming it, any `pub`,
or a public top-level Python name. Code lines hold source text, string contents included, that is
neither a comment nor a Python string statement (docstrings and strings used as
comments). A function's range runs from its declaration (binding, export, wrapper
call, or test call with its table) to the end of its body, but a callback passed
to a method, as in `items.filter(...)`, starts at its arguments. Each code line
belongs to the smallest range holding it, so functions never share lines, and a
file's lines are its code lines. Rust methods read
`Type::method` with the implemented type's own name, and Python methods
`Class.method` (`Outer.Inner.method` when nested); closures and lambdas fold into
their function.

Cognitive complexity follows SonarSource 1.7 in every language: `if`, loops,
`switch`/`match`, `catch`/`except` and ternaries add 1 plus nesting (a `switch`
counts once), `else`/`else if`/`elif` add 1 without nesting, each run of like
`&&`/`||`/`and`/`or` adds 1 (Rust let chains included), `??`, `?.` and Rust `?` add nothing, labelled jumps add
1, and folded callbacks nest their bodies. Deviations: recursion is not counted,
named nested functions are their own units, JavaScript's "declarative outer
function" exemption is not applied, and Rust macro bodies are not measured.
eslint-plugin-sonarjs 4.2.2 agrees exactly on 88% of corpus functions and on the
band of 99%; it scores callbacks separately, ignores `||` runs and exempts JSX
`&&`, so its numbers are lower by design.

Imports are read from the syntax tree: `import`, `export … from`, `import x =
require()`, and `import()` or `require()` of a quoted string in TypeScript and
JavaScript; Python `import` and `from … import` with their leading dots, one record
per name (`imported_name`), but not `from __future__` directives; and Rust `use`
expanded to one path each. Relative imports resolve to existing files: script
specifiers to `.ts`, `.tsx` and JavaScript files and directory indexes (not `.d.ts`
declarations), Python modules by dot level, a package before a same-named module.
Package imports, tsconfig aliases, Rust paths and absolute Python imports remain
unresolved. This is
a structural assessment, not TypeScript type checking or React hook-rule linting.

# Development checks

`make check` runs formatting verification, Clippy with warnings denied for all
targets, unit tests, CLI integration tests, and the TypeScript smoke test.
The integration suite extracts this project's `src` directory and enforces:

- every file has a module description, at most 400 code lines and no parse errors;
- no function is red, and the share of function lines in yellow functions stays
  within `MAX_YELLOW_PER_MILLE`;
- `src` stays within the project budget in `tests/cli.rs`: files, total code
  lines and dependencies;
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
