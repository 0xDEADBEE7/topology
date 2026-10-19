# Roadmap: React support first, then one tree-sitter walker

Status (October 2026): M0–M8 done; released as 0.2.0.

- The stash/fig scripts and their Make targets live in the untracked `.local/scripts/`
  (included by the Makefile when present).
- M1 verify: no corpus aborts; partially parsed files 0/5/5/5 (bulletproof-react/
  excalidraw/zustand/docusaurus) plus 2 deliberately broken docusaurus JS fixtures;
  symbol lookup on excalidraw's 11k-line App.tsx 0.03 s; tree on 10,000 files 0.20 s;
  0 empty descriptions; binary 4.85 MB. The corpus diff was not limited to previously
  failing files: tree-sitter-typescript 0.23 parses `import.meta` differently (Halstead
  moved in 25 files), and 2 docusaurus imports with import attributes lost a resolution
  that only worked by accident (their sources are mangled until M6). Relative resolution
  already fixes excalidraw's 31 wrong-file imports ahead of M6.
- M2 verify: NMI, Halstead and CC are gone from code, output and JSONL; function records
  gain `band`; every function's lines and cognitive value are unchanged on the corpus.
  topo scores all green on itself, so the yellow ceiling starts at 0. Red shares on the
  React repos: bulletproof-react 17%, docusaurus 39%, zustand 51%, excalidraw 55%. Of
  those red lines 36–94% are `<anonymous>`, and 82–99% of the anonymous ones are test
  `describe(...)` callbacks (M3's test-block units and decision 7 address them).
- M3 verify: `<anonymous>` is 0–0.2% of corpus functions (was 61–87%); duplicate ids 0
  (was 3–29 per repo); `find Button` and `form.tsx::FormItem` work on bulletproof-react;
  excalidraw's history.test.tsx yields one unit per `it` (63 of 63), so the test-block rule
  is in. TypeScript/JavaScript cognitive complexity follows SonarSource 1.7 (the
  whitepaper's `sumOfPrimes` scores 7). Red shares fell to bulletproof-react 17%,
  docusaurus 13%, zustand 14%, excalidraw 37% (its largest red functions are genuinely
  long: an 849-line pointer handler with cognitive 226). Rust and Python output is
  unchanged. Beyond the plan: a function's lines are the lines it owns (named nested
  functions count separately), so a `describe` block is not charged for its `it` units.
  M3 added about 170 lines of product code, above its +80 to +110 estimate.
- M4 verify: `src/lang.rs` holds the `Language` enum, the only extension list, and the
  TS/TSX/JS syntax table; `src/walk.rs` is the table-driven walker and the one `analyse`
  entry point; the `Adapter` trait and `src/adapters/` are gone (the Python heuristic
  moved to `src/python.rs`, the Rust entry point into `src/visit.rs`, both for M5 to
  delete). Extract, score, tree and outline output is byte-identical to M3 on every
  corpus repository and on topo itself. Unit tests left `src` for integration tests in
  `tests/` (not a separate `tests/languages.rs`). M4 removed only 10 lines, not the
  planned 80–120: the syntax table and `Language` methods cost about what the adapter
  layer saved, so the reduction now rests on M5 deleting syn and the Python heuristic.
- M3 review fixes (landed with M5): a wrapper call names only a callback with no function
  or call before it (4 excalidraw `memo` comparators had taken their component's name and
  turned red), callee names elide arguments (`describe.each(…)("adds %i")`, 68 multi-line
  names down to 4), parameter defaults are measured, `else` before a comment still reads
  as `else if`, computed keys fold, `path::symbol` prefers an exact name, and outline depth
  ignores equal ranges. eslint-plugin-sonarjs 4.2.2 cross-check: 80.4% exact and 97.9% same
  band on 1,135 sample functions (88.1% / 98.7% on 15,364 corpus functions); every
  difference comes from sonarjs scoring callbacks separately, ignoring `||` runs and
  exempting JSX `&&`, where topo follows the whitepaper.
- M5 verify: one tree-sitter walker measures Rust, Python, TS/TSX and JS from tables in
  `src/lang.rs`; syn, proc-macro2, the Rust visitors and the Python line heuristic are
  gone. requests vs Python's `ast`: 711/711 names, 709/711 exact spans (the 2 others end
  at trailing comments), 711/711 docstrings. ripgrep: 2,977 functions (was 2,914), Rust
  methods named `Type::method`; topo's own function list matches M4 once the prefix is
  stripped. Lines are code lines everywhere (bands 40/75). TS/JS cognitive scores and
  names are unchanged by the walker rewrite. Binary 5.18 MB; clean release build 10.5 s
  (was 13.3 s). M5 removed 135 physical lines net (176 before the M3 fixes), short of
  the planned 260–320: the Rust and Python tables and code-line counting cost about
  100 lines of what syn and the heuristic freed.
- M5 review (27 of 35 findings confirmed; all fixed here or moved to M6): Rust let
  chains count their `&&` run; Rust docs reach past attributes and plain comments, and
  `////` is not a doc; Python methods under `if`/`try` and in nested classes qualify
  (`Popen._execute_child`, `Outer.Inner.deep`); Rust impls use the type's own name
  (`S::begin`, `Error::from`, 38 ripgrep names); `memo(Base, compare)` no longer names
  the comparator; text in hidden grammar tokens counts (Python strings with escapes,
  Rust raw-string delimiters); each code row has one owner, so sibling callbacks,
  multi-declarator `const` and functions in `it.each` tables no longer double-count,
  and a method-chain callback starts at its arguments (excalidraw's i18n `.filter`
  callback was 83 rows of array data); multi-line receivers print as `…`; equal-range
  outline entries share a depth. Rust and Python fixtures now pin whitepaper scores,
  names, docs and lookups (before, a one-token table typo cut topo's own cognitive total
  from 220 to 126 with every test green). cognitive_complexity 1.3.0 (PyPI) on requests:
  96.9% exact, 99.9% same band; every difference is topo's documented nested-function
  deviation or the package's `elif` nesting bug. Kept: let-else adds nothing, as in
  SonarSource's own Rust analyzer. The corpus harness now flags requests' 70 malformed
  Python imports (it reported 0).
- M5 fix review (16 of 20 findings confirmed, 12 of them regressions; all fixed in M6
  except pre-existing parenthesized Python docstrings): a method-chain callback bound by
  an object key, class field or assignment starts at its binding; a comment is not a
  wrapper call's argument; functions in objects, decorators, heritage clauses and
  static blocks are not class members (16 corpus names lose a wrong
  `App.`/`WaypointClient.` prefix); Rust impls on arrays, tuples, pointers, fn types and
  trait objects keep their type's text (`[u8; 4]::size`, not `4]::size`); equal rows go
  to the unit with fewer bytes, so `withX = (C) => function WithX() {...}` charges the
  body to `WithX`; Unicode and zero-width spaces are blank; an outline entry inside two
  equal-range entries sits one level below them; f-strings and bytes are not docstrings.
- M6 verify: one `FileInfo` per file (units with doc, signature, `exported` and
  `parent`; imports; module doc) feeds score, extract, the tree, outlines and `find`.
  The 14-file `extract/`, its line scanners, models.rs and serde are gone, so syn,
  quote and proc-macro2 leave the build (`cargo tree -i syn` prints nothing; 5
  dependencies). Bogus import sources fell from 939/13/100/414/70/8 to 0 on all six
  repositories (the harness now accepts the valid specifiers `.` and `..`). The 31
  excalidraw wrong-file resolutions were already fixed in M1; now no import resolves to
  its own file and no earlier relative resolution changed, while resolved pairs rose
  (excalidraw 2,299 → 2,862, docusaurus 852 → 1,362, requests 0 → 79 relative Python
  imports; its 2 absolute `tests.testserver.server` resolutions are dropped by design).
  Outlining a 40,000-line TS file takes 0.09–0.18 s (2.4–14.5 s before, on three
  generated files). The Doc column equals extract's descriptions in every repository:
  described functions went from 541 to 643 in excalidraw (JSDoc above class fields and
  object pairs) and from 305 to 259 in docusaurus (tag-only JSDoc such as `/** @type */`
  describes nothing); plain `//` comments above TS functions no longer count. Function
  names, lines and metrics, file descriptions and the directory tree are otherwise
  unchanged. The tree now parses every file, so it takes 0.57 s on excalidraw (was
  0.02 s) and warns about partially parsed files; extract takes 0.64 s (was 1.08 s).
  M6 removed 360 code lines (1,845 → 1,485), within its −350 to −450 target, after
  about 35 lines for the M5 review fixes and `export { }` clauses.
- M6 review (30 of 31 findings confirmed; fixed here unless listed as a limit): an
  unterminated import string no longer panics every command; extract streams again
  (`extract | head` on excalidraw 0.54 → 0.01 s); `#[doc(alias …)]` and `include_str!`
  are not docs; prose that mentions copyright stays a description; leading `**`, `/`
  and `!` in doc text survive; a JSDoc block that is only tags ends the search and never
  lends a later paragraph; decorated and line-broken class fields, pairs and assignments
  start at their binding and keep their JSDoc and name (excalidraw: 29 parents restored,
  47 signatures lose container or sibling text); members of one-line objects, classes
  and impls no longer take the container's doc; `export { a }` marks only the top-level
  `a`; a Python package wins over a same-named module. Known limits, all rare in the
  corpus: doc search is quadratic in a run of thousands of comments above one
  declaration; a callback bound mid-chain keeps the chain in its signature; later
  declarators in `const a = …, b = …` take the statement's doc; Python public names under
  a module-level `if`/`try` are not exported; `#[doc = "…"]` escapes are not decoded; a
  comment before a test label loses the label.
- M7 verify: `topo` alone inspects `.`; `--help` prints the usage to stdout and exits 0;
  `--version` prints the version; extra arguments, unknown options and `find` without a
  name are usage errors (exit 2); `find <name> [dir]` and a missing `file::symbol` exit
  1, and an ambiguous `file::symbol` lists every candidate and exits 1; `file::Type::method`
  works. `--no-colour` (use `NO_COLOR`), `--full-docstring` and the `tree` alias are
  gone. Discovery now scans `bin` and `build` (docusaurus's `src/commands/build/*.ts`
  and three `bin` scripts are found, +29 functions), skips `*.d.ts` and `*.min.js` in
  scans (68 corpus files) and venv, `site-packages` and `__pycache__`, reads a named
  file whatever its name with a warning when unsupported, follows a named symbolic
  link, and analyses a file once however it is spelt. Records for every other file are
  unchanged, and 16 files remain partially parsed (5/5/0/6/0/0).
- M7 review (15 of 21 findings confirmed, all fixed): every exact `file::symbol` match is
  shown (two `fmt` impls were unreachable); options after `find` and `extract` and
  non-Unicode arguments are usage errors; a named FIFO no longer hangs; the first
  spelling of a file wins; unreadable entries warn again. A simplification pass then
  removed 87 lines with byte-identical output (walk.rs 382 → 345, syntax.rs 249 → 223).
- Decisions 5 and 7, approved by the owner: directory scans list files with
  `git ls-files -co --exclude-standard` and fall back to a walk outside a repository
  (corpus extract and tree output unchanged, since the checkouts are clean), and
  `score` leaves out tests, mocks and stories unless `--tests` is given. Scored files,
  before → after: excalidraw 695 → 539, docusaurus 1,141 → 810, bulletproof-react
  428 → 349, zustand 49 → 34, requests 37 → 22, ripgrep 110 → 92. Red shares rise
  (excalidraw 32.7% → 44.6%, zustand 12.8% → 25.9%, requests 10.2% → 21.6%) because
  mostly green test units no longer dilute the production code's.
- M8 verify: `src` is the end state's nine files (metrics.rs moved into walk.rs). README.md
  shows real output for every command, and `make check` fails when that output drifts;
  docs/usage.md (commands, exit codes, discovery, records, adding a language) and
  docs/metrics.md (definitions, deviations, cross-checks, pitfalls) replace six older
  docs. Corpus: no aborts, `<anonymous>` at most 0.23% (excalidraw 17 of 7,428), no
  bogus imports, no duplicate ids. Binary 5,179,088 bytes with one codegen unit (5.2 MB
  target). topo on itself: 93 functions, 0 yellow, 0 red. CI runs `make check` on Linux
  and macOS.
- M8 review (27 of 32 findings confirmed; fixed unless listed): git runs with the
  repository's file monitor hook turned off, so a scan executes nothing the checkout
  configures; `file::symbol` on a FIFO errors instead of blocking; Rust `#[doc = ""]`
  separators, `\n` escapes and quotes in generated docs read as rustdoc does, and doc
  comments are collected across plain comments and attributes; TS/JS `//!` loses its
  marker; score explains that tests need `--tests` when nothing else is left; the label
  column is sized in characters; make targets and test.sh honour `CARGO_TARGET_DIR` and
  any working directory, and the git-sensitive tests no longer depend on where TMPDIR
  is. Documented instead: a file nested tens of thousands of levels deep (only
  generators write one) overflows the stack and aborts the run; Markdown headings such
  as `# Safety` stay descriptions; git skips unreadable directories silently.
- Size: `src` is 9 files and 1,424 code lines, all green. Budget 9 files and 1,425 code
  lines (the release size rounded up to the next 25); yellow share 0‰.

## 1. Recommendation

Make React and plain JavaScript work first (M1), then shrink the codebase by moving every
language onto one table-driven tree-sitter walker. Each milestone is one to three commits,
keeps `make check` green, and is checked against a pinned corpus of real repositories
(`.scripts/corpus.sh`). Pure moves are kept separate from behaviour changes, and the
TypeScript adapter becomes the generic walker in a refactor whose output must not change.

The self-quality gate changes before any code does: complexity limits apply to functions,
and a project-wide size budget makes splitting files gain nothing.

**End state:** about 1,600 physical lines in 9 files (from 2,538 in 45), one parser for
Rust, Python, TS/TSX and JS/JSX, and 5 dependencies.

## 2. Evidence behind the plan

Measured in October 2026 against topo itself and a corpus of excalidraw, zustand,
bulletproof-react, docusaurus, requests and ripgrep:

- Only 21 of about 2,200 TS/TSX corpus files fail to parse, but one failure aborted
  `score`, the tree and `find` for the whole repository, so 3 of 4 React repositories
  produced no report. Upgrading tree-sitter fixes 6 of the 21; the rest have 1–3 ERROR
  nodes and score near-identically when errors are tolerated.
- JS/JSX/MJS/CJS parse 208 of 210 corpus files with the existing TSX grammar.
- 61–87% of TS functions were reported as `<anonymous>` (forwardRef/memo components,
  inline callbacks).
- `tests/cli.rs` applied function-sized limits (CC < 15, cognitive < 15, Halstead < 1000)
  to whole files. That, not NMI, drove the split from 18 to 45 files (+299 lines).
- Per-file NMI is 70–95% explained by ln(LoC): halving a file raises each half by about
  8.7 points. "Halstead volume" used total counts as distinct counts in every adapter.
  Cognitive complexity departed from the SonarSource spec (a 10-case `switch` scored 18;
  the spec gives 1).
- A single tree-sitter walker with per-language node-kind tables was prototyped three
  times. It matched `syn` on all 128 of topo's functions, matched Python's `ast` on
  711/711 functions in requests, found all 2,977 ripgrep functions (topo found 2,914),
  and removes `syn` and `proc-macro2`.
- Research: size confounds most code metrics; per-function complexity is the part that
  adds information; churn × complexity hotspots are the cheapest high-signal addition;
  import and type resolution belong to language servers, not topo.

## 3. Metric policy

**Dropped in M2:**

| Metric | Why |
|---|---|
| NMI | 70–95% of it is explained by ln(LoC), and halving a file raises it by about 8.7 points. |
| Halstead volume | Distinct counts equal total counts. A corrected version duplicates LoC (ρ 0.82–0.93) and turns 16–29 of topo's own files red. |
| Cyclomatic complexity (CC) | Keep only one complexity metric. |
| File-level CC, cognitive and Halstead sums | File totals mostly measure size again. |
| Docs as a quality band | No evidence that docs predict maintainability. |

The flags `--metrics`, `--all-metrics` and `--functions` are removed too.

**Kept, rated per function, with one definition for every language:**

| Metric | Green | Yellow | Red |
|---|---|---|---|
| Code lines: rows holding source text other than comments and Python string statements; each row counts for the smallest function range holding it. Used from M5; before that, physical lines with today's 60/70 limits. | ≤40 | 41–75 | >75 |
| Cognitive complexity (Sonar spec 1.7) | ≤10 | 11–15 | >15 |

**Cognitive complexity rules:**
- `if`, ternary, loops and `catch`/`except` add +1 plus the current nesting depth.
- `switch`/`match` adds +1 plus nesting once; individual cases add nothing.
- `else`, `else if` and `elif` add a flat +1 and do not deepen nesting.
- Each run of the same boolean operator (`&&`, `||`, `and`, `or`) adds +1.
- `??`, `?.` and Rust `?` add nothing.
- A labelled `break` or `continue` adds +1.
- Unnamed callbacks, lambdas and closures folded into their parent add nesting only.
- Documented deviations from Sonar: no increment for recursion, named nested functions
  are their own units, JavaScript's "declarative outer function" exemption is not
  applied, and code inside macro bodies is not measured.

**Shown but never rated:**
- Per file: lines, module doc yes/no, and a `~` marker for partially parsed files, followed
  by every yellow and red function with its lines and cognitive complexity coloured
  (`--detail` lists all functions). This replaces the planned function count and single
  worst-function column: it names every risky function without widening the table.
- Summary line: counts of yellow and red functions and the share of function lines they
  hold, plus doc coverage.
- One `BANDS` constant in report.rs drives the colours, the guide text and a `band` field
  in the JSONL output.

**Rule:** never raise a band or a budget to make topo pass. If topo fails, simplify the
code. A constant changes only with the owner's approval and a reason in the commit message.

**Final self-gate** (in `tests/cli.rs`, reading `topo extract src` and Cargo.toml):
1. No function has `band == "red"`.
2. Lines in yellow functions are at most `MAX_YELLOW_PER_MILLE` of all function lines. It
   starts at the measured value and ratchets down (0 since M2).
3. Every file has a non-empty module doc, `parse_errors == 0` and `lines ≤ 400`, and
   stderr is empty.
4. Total code lines ≤ `MAX_LINES`, with a hard ceiling of 1,500, and files ≤ `MAX_FILES`
   (1,425 and 9 at 0.2.0).
5. Cargo.toml has at most 5 `[dependencies]`.
6. Hand-computed fixtures in `tests/fixtures`, checked by `tests/cli.rs`, keep a walker bug
   from passing the gate by under-counting: the Sonar whitepaper examples in Rust, TS and
   Python, among others.

**Budget ratchet:** after each milestone, set the budget to the measured size plus the next
milestone's planned growth plus 25 lines; a milestone that plans to shrink the code counts
as 0 growth until its deletions land. It never goes up. The budget counts physical lines
until M5 and code lines from then on.

## 4. Milestones

### M0: checkpoint, cleanup, harness, gate (no CLI change)
- Commit the uncommitted 45-file working tree as a checkpoint.
- `git rm` primary.pf (4 MB of zero bytes) and data.jsonl.lock.
- Move the stash/fig scripts and their Make targets out of the repo; fix help.sh,
  metrics.sh and `.PHONY`.
- Add `.scripts/corpus.sh`: fetches each corpus repo at a pinned commit, handles paths
  with spaces, and has a `diff A B` mode. Per repo it records aborts and partially parsed
  files, the share of `<anonymous>` functions, bogus import sources, duplicate ids, empty
  descriptions, timings and binary size.
- Gate: apply the existing caps (CC < 15, cognitive < 15, Halstead < 1000, LoC ≤ 60) to
  functions only; drop the file-level CC, cognitive, Halstead and NMI asserts; cap files
  at 400 lines; allow at most 45 files and 2,650 lines in total.
- Delete the old extraction handoff doc and fix documented commands that fail.
- **Verify:** the baseline reproduces the audit numbers (752 of 1,234 bulletproof-react
  functions are `<anonymous>`; 3 corpus repos abort).

### M1: React works and the read paths are robust (first milestone where React projects work)
- Move to tree-sitter 0.25.10 and tree-sitter-typescript 0.23.2; declare `rust-version`.
- A file with syntax errors is measured anyway: it gets a `parse_errors` count and a
  warning that names it.
- Parse `.js`, `.jsx`, `.mjs` and `.cjs` with the TSX grammar, labelled `javascript`.
- tree, find and symbol lookup skip unreadable or non-UTF-8 files and name them.
- `file::symbol` looks only in that file instead of re-analysing the working directory.
- Reset SIGPIPE so piping into `head` exits cleanly; colour output only when stdout is a
  terminal and `NO_COLOR` is unset.
- Merge `tree/*` into tree.rs as a pure move, then key tree entries by path components.
  This fixes `x.rs` next to a sibling `x/` rendering in the wrong place.
- The tree no longer parses files. One module-doc reader skips license headers, accepts
  `/*!`, and treats an empty result as no description.
- Cap the alignment width so one long name no longer blanks every description, and list
  Rust structs in outlines.
- Tests: update the tests that expected invalid source to fail; add a fixture for each
  kind of syntax that still fails to parse; add a golden tree test for `x.rs` next to
  `x/`; check that `extract . | head -1` does not panic.
- **Verify:** no aborts across the corpus; partially parsed TS files 0/5/5/5
  (bulletproof-react/excalidraw/zustand/docusaurus); 208 of 210 JS files parse cleanly;
  symbol lookup ≤ 0.05 s (was 2.03 s); empty description rows in docusaurus go from 996
  to 0; the corpus diff changes only files that previously failed.

### M2: honest metric set and one report
- Delete Halstead, NMI and CC everywhere. Merge `report/*` into one report.rs driven by
  `BANDS`, and merge `cli/*` into main.rs.
- The JSONL output drops `nmi`, `halstead` and `cc` and gains `band`. Add a golden test
  that pins the JSONL output for a fixture.
- Gate: no red functions, and `YELLOW_MAX` set to the measured value.
- **LoC:** −170 to −230. **Verify:** topo scores all green on itself; pure-move commits
  leave corpus output unchanged.

### M3: React semantics in the TS adapter
- Name each function after what it is assigned to, seeing through parentheses, `as`,
  `satisfies`, non-null wrappers, and forwardRef, memo, useCallback, useMemo, lazy and
  observer calls. Methods are named `Class.method`. Remaining unnamed top-level functions
  fall back to `callee(…)` or `default`.
- Fold unnamed callbacks into their parent function, shipping the Sonar cognitive rules
  for TS/JS in the same step.
- Sort outlines by line, nest entries inside the entry that contains them, and give each
  id a per-file ordinal so ids are unique.
- Optionally, a callback passed to a call whose first argument is a string becomes its own
  unit (`it("…")`), only if a prototype on excalidraw's history.test.tsx gives one unit
  per `it`.
- **LoC:** +80 to +110. **Verify:** `<anonymous>` ≤ 1% across the corpus; `find Button`
  and `form.tsx::FormItem` work; duplicate ids go to 0; cross-check cognitive scores
  against eslint-plugin-sonarjs.

### M4: turn the TS adapter into `walk.rs` and `lang.rs` (pure refactor)
- An `enum Language` becomes the only list of file extensions; delete `Box<dyn Adapter>`.
- TS, TSX and JS share one set of node-kind tables. Move the TS unit tests to
  `tests/languages.rs`.
- **LoC:** −80 to −120. **Verify:** extract JSONL for every TS/TSX/JS corpus file is
  byte-identical to M3.

### M5: Python and Rust onto the walker
- Add tree-sitter-python and tree-sitter-rust; remove syn and proc-macro2. Rust names are
  `Type::method`, `#[doc]` counts as documentation, macro bodies are skipped, trait
  default methods are measured.
- Delete the syn adapter and the Python line heuristic; switch every language, and the
  budget, to code lines.
- **LoC:** −260 to −320. **Verify:** requests vs Python `ast`: 711/711 names, ≥709/711
  exact spans, 711/711 docstrings; ripgrep: 2,977 functions; topo's own function list
  matches M4 once the `Type::` prefix is stripped.

### M6: one data model read from the syntax tree
- One typed `FileInfo` record carries doc text, signature, `exported` and `parent`, and
  every command uses it. Descriptions come from the doc node the walker finds, so the
  Doc column and `extract` agree (today the line reader misses single-quoted and
  prefixed Python docstrings, and the walker misses JSDoc above class-field arrows,
  object pairs and assignments).
- Imports come from syntax-tree nodes: re-exports, `import()`, `require()`, Python relative
  imports with dot levels, Rust grouped `use` expanded per name. Resolve relative imports
  only.
- Delete the line scanners in extract/, models.rs and the serde derive dependency.
- **LoC:** −350 to −450. **Verify:** bogus import sources go to 0; excalidraw's 31
  wrong-file resolutions go to 0; outlining a 40,000-line TS file takes < 0.5 s (was
  10.8 s); `cargo tree` shows no syn.

### M7: CLI, file discovery and agent usability
- Help prints to stdout and exits 0; add `--version`; extra arguments are errors; bare
  `topo` means `topo .`.
- `find <name> [dir]` exits 1 when nothing matches; `file::Type::method` lookups work;
  ambiguous names list every candidate.
- Discovery: stop excluding `bin` and `build` at any depth; exclude venv, `__pycache__`
  and site-packages; skip `*.d.ts` and `*.min.js` in directory scans; warn when a named
  file is unsupported.
- Remove `--no-colour`, `--full-docstring` and the `tree` alias.
- **LoC:** +20 to +60.

### M8: lock in and release
- Consolidate to the end-state layout; README with real command output; `docs/metrics.md`
  (definitions, sources, deviations, pitfalls); `docs/usage.md` replaces the four
  ux-patterns docs and documents the exit-code contract and how to add a language.
- test.sh runs every command shown in the README. Set the final budget constants (measured
  size rounded up to the next 25, at most 1,500) and tag v0.2.0.
- **Verify:** no aborts, ≤ 1% `<anonymous>`, no bogus imports; binary ≤ 5.2 MB; 0 red
  functions and ≤ 10% yellow lines on topo itself.

## 5. End state

| File | ≈Lines | Contents |
|---|---|---|
| main.rs | 150 | arguments, help, dispatch, SIGPIPE reset |
| paths.rs | 70 | file discovery and filters |
| lang.rs | 200 | `Language` extension registry plus node-kind tables for Rust, Python and JS/TS |
| walk.rs | 300 | parsing, function units, naming and folding, cognitive complexity, code lines |
| syntax.rs | 170 | docs, signatures, exports, imports |
| report.rs | 170 | `BANDS`, score rows, summary line |
| table.rs | 90 | table layout |
| tree.rs | 250 | tree, outline, symbol lookup, find |
| extract.rs | 140 | JSONL output, ids, import resolution |

Dependencies: tree-sitter, tree-sitter-typescript, tree-sitter-python, tree-sitter-rust
and serde_json. Tests are about 500 lines and do not count toward the budget.

## 6. Non-goals

- Import resolution beyond relative paths (tsconfig `paths`, workspaces, Node exports,
  Rust module trees, Python import roots).
- Cross-file analysis: references, call graphs, a persistent index, a daemon, an MCP
  server or LSP.
- tree-sitter-javascript, query engines, runtime grammar loading, or more languages.
- Halstead, maintainability index, CC, OO metrics, duplication detection, COCOMO, HTML
  reports.
- clap, ignore and unicode-width; `--strict`, `--fail-on` and `.topoignore`.
- Interim patches to the Python line heuristic or the syn adapter (both deleted in M5).
- Deferred: hotspots and change coupling (about 80 lines; must fit under the budget).

## 7. Open decisions (recommended default first)

1. Drop NMI, Halstead and CC — yes (alternative: keep CC as a JSONL-only field).
2. Fold unnamed callbacks into their parent — yes (reverses
   `nested_functions_have_independent_complexity`).
3. Treat the extract JSONL as internal and change it freely under a contract test — yes.
4. Import resolution: relative imports only (alternative: raw specifiers only).
5. Discovery via `git ls-files -co --exclude-standard` with the directory walk as fallback
   (applied in M7).
6. Describe undocumented files in the tree by their exported names.
7. `score` skips tests and stories by default, with `--tests` to include them (applied in
   M7).
8. tree-sitter 0.25.10, the mature release (0.27.1 is newer; both need Rust 1.90 because
   the shared grammar runtime, tree-sitter-language 0.1.9, does).
9. SIGPIPE: a small `unsafe` reset (alternative: the safe `writeln!` route).
10. Hotspots after the roadmap.

## 8. Risks

- Regression when moving to the walker: M4 must be byte-identical; M5 is checked against
  Python's `ast` and a function-list diff.
- topo measuring itself: hand-computed fixtures and external cross-checks keep a
  measurement bug from passing the gate.
- The budget turns out too tight: simplify the code; never raise the ceiling.
- The TypeScript grammar is no longer maintained upstream: tolerant parsing and the
  regression fixtures cover new syntax.
- Breaking CLI and JSONL changes: the contract test, `--version` and release notes.
