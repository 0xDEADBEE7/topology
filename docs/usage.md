# Usage

```text
topo [<path> | <file>::<symbol>]   tree of a directory, outline of a file, or one symbol
topo find <name> [<dir>]           every declaration with that name
topo score [--detail] [--guide] [--tests] [<path>...]   function bands per file
topo extract [<path>]              one JSON record per file, import and declaration
topo --help | --version
```

Paths default to the working directory. Colour is used only on a terminal and never when
`NO_COLOR` is set.

## Read a repository progressively

Use the smallest view that answers the question, and pass what it prints to the next
command:

```text
topo .                       repository shape with each file's description
topo src/walk.rs             the file's description and its declarations with line ranges
topo src/walk.rs::increment  one declaration's source
topo find increment          where a name is declared, as path::name (Lstart–end)
```

- Outlines are not source dumps: pick one or two declarations, then read only those.
- `find` matches the full name (`Walker::increment`), its last segment (`increment`), or
  `path::name` with or without the extension (`src/walk::Walker::increment`). Search is
  exact, so try compound variants (`import`, `imports`, `resolve_import`) or browse the
  file outline when a guess finds nothing.
- `file::symbol` looks only in that file and shows every declaration named exactly
  `symbol` (two `fmt` impls, say). With no exact match it shows the one declaration whose
  last segment matches, and when several or none do it lists the candidates instead.
- Use `score` to prioritize reading and refactoring, not to orient yourself: a yellow or
  red function is a candidate, not a defect. See [metrics](./metrics.md).
- Use `extract` when another tool needs the data, for example
  `topo extract . | jq -c 'select(.type == "import" and .path == "src/main.rs")'`.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | The command succeeded; for `find` and `file::symbol`, something matched. |
| 1 | Nothing matched: `find` found no declaration, or `file::symbol` found none or several candidates (listed under an `N matches for` line). |
| 2 | A usage error (unknown option, extra or missing argument, or an argument that is not Unicode) or a failure such as an unreadable named path, an unsupported file to inspect, or no supported files to score. |

Help (`--help` or `-h`, anywhere) and the version print to stdout and exit 0. Results and
match counts go to stdout; warnings and errors go to stderr, prefixed `warning:` or
`error:`. A closed pipe (`topo extract . | head`) ends topo quietly.

## What topo reads

Supported files are `.rs`, `.py`, `.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`
and `.cjs`. TypeScript and TSX use their own grammars and JavaScript uses the TSX grammar;
nothing from the project is installed or run.

A directory scan takes the files git tracks or would track there
(`git ls-files -co --exclude-standard`), or every file a walk finds outside a repository.
Either way it leaves out hidden entries, symbolic links, dependency and install folders
(`node_modules`, `vendor`, `venv`, `site-packages`, `__pycache__`), build output
(`target`, `dist`), `coverage`, TypeScript declarations (`*.d.ts`) and minified scripts
(`*.min.js`); `bin` and `build` are scanned, since they often hold sources. `score` also
leaves out tests, mocks and stories unless `--tests` is given: `test`, `tests`,
`__tests__`, `__mocks__` and `e2e` folders, `*.test.*`, `*.spec.*`, `*.stories.*`,
`test_*.py`, `*_test.py` and `conftest.py`.

A path named on the command line is always read: a directory even when its name is
excluded, ignored or a symbolic link, and a file whatever its name (score, find and extract
warn and skip one in an unsupported language; inspecting it is an error). A file reached
through several spellings is read once, under the first.

A file with syntax errors is measured from tree-sitter's recovered tree, with a warning
naming it; the score table marks it `~` and extract records `parse_errors`. A file or
directory that cannot be read is skipped with a warning (the tree leaves it out).

### Descriptions

A file's description is its module docstring or the `//!`, `/*!` and `/** */` blocks
before its first code, past shebangs, inner attributes and directives such as
`'use client'`. A declaration's is the doc comment above it, past attributes, decorators
and plain comments, or the docstring opening its body (never an f-string or bytes
literal). A description is the first paragraph of prose up to its first `@` tag; plain
comments, tag-only JSDoc, license blocks and reStructuredText titles describe nothing.
A directory's description is the first line of its `.docstring` file. The score's Doc
column marks exactly what has a description.

### Imports

Imports come from the syntax tree: `import`, `export … from`, `import x = require()`, and
`import()` or `require()` of a quoted string in TypeScript and JavaScript; Python `import`
and `from … import` with their leading dots, one record per name (not `from __future__`);
and Rust `use`, one record per expanded path. Relative imports resolve to existing files:
script specifiers to `.ts`, `.tsx` and JavaScript files and directory indexes, Python
modules by dot level, a package before a same-named module. Package imports, tsconfig
aliases, Rust paths and absolute Python imports stay unresolved.

## Extract records

`topo extract` prints one JSON object per line, with sorted keys: each file, then its
imports, functions and classes. Ids are unique within a file. The shape is internal and
pinned by `tests/fixtures/typescript/basic.jsonl`.

| `type` | Fields |
|---|---|
| `file` | `id`, `path`, `description`, `metrics` (`language`, `loc`, `parse_errors`) |
| `import` | `id`, `path`, `line`, `source` as written, `imported_name` (Python names), `resolved_path` or null |
| `function`, `class`, `struct` | `id`, `path`, `name`, `language`, `lines` [start, end], `signature` (up to the body), `description`, `exported`, `parent`, and for functions `metrics` (`loc`, `cognitive`, `band`) |

`exported` is true for `export` (or a top-level name a local `export { }` or
`export default` names), any Rust `pub`, and public top-level Python names. `parent` is
the class, impl or trait whose name qualifies the declaration.

## Add a language

1. Add the grammar crate to Cargo.toml. The budget of 5 dependencies is already full, so
   a new language means agreeing to raise it (more languages are a non-goal of the
   [roadmap](./roadmap.md)).
2. In `src/lang.rs`, add a `Language` variant, its `EXTENSIONS` rows and its grammar, and
   write a `Spec` naming the grammar's node kinds: functions, closures, classes, scopes,
   bindings, structures and the other tables documented on `Spec`.
3. Check the kinds `walk.rs` and `syntax.rs` name themselves (calls, labels, let chains,
   strings, doc comments, exports and imports) and handle the language's equivalents.
4. Add a fixture with the whitepaper's examples and hand-computed scores, names and docs
   next to `tests/fixtures/rust/semantics.rs`, and run the corpus harness.
