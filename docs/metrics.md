# Metrics

`topo score` rates functions, not files. Each function gets two numbers and takes the
worse of their bands; `topo score --guide` prints the table.

| Metric | Green | Yellow | Red |
|---|---|---|---|
| Code lines | ≤ 40 | 41–75 | > 75 |
| Cognitive complexity | ≤ 10 | 11–15 | > 15 |

Files show their code lines and whether they have a module doc, but are not rated, and
neither are docs. The summary line counts yellow and red functions, the share of function
lines they hold, and doc coverage.

## Code lines

A code line is a row holding source text, string contents included, that is neither a
comment nor a Python string statement (a docstring or a string used as a comment). Blank
rows, including rows of Unicode or zero-width spaces, never count.

A function's range runs from its declaration to the end of its body. The declaration is
its binding (`const`, a class field, an object key, an assignment), its `export`, the
wrapper call that names it (`memo(...)`, `forwardRef(...)`), or the test call that labels
it, together with a `.each` table. A callback passed to a method, as in
`items.filter(...)`, starts at its arguments unless the call itself is bound. Each code
line belongs to the smallest range holding it, in rows and then bytes, so functions never
share lines: a `describe` block is not charged for its `it` units, and a component
returned by a higher-order function owns its own body. A file's lines are all its code
lines.

## Cognitive complexity

topo follows the SonarSource specification (version 1.7) in every language:

- `if`, ternaries, loops and `catch`/`except` add 1 plus the nesting depth.
- `switch`/`match` adds 1 plus nesting once; its cases add nothing.
- `else`, `else if` and `elif` add 1 and do not deepen nesting.
- Each run of like boolean operators (`&&`, `||`, `and`, `or`) adds 1; a Rust let chain
  is a run of `&&`.
- `??`, `?.` and Rust's `?` add nothing.
- A labelled `break` or `continue` adds 1.
- Unnamed callbacks, lambdas and closures fold into the function around them and nest
  their bodies.

Deviations from the specification: recursion adds nothing, named nested functions are
units of their own, JavaScript's exemption for a "declarative" outer function is not
applied, Rust `let … else` adds nothing (as in SonarSource's own Rust analyzer), and code
inside macro bodies is not measured.

## Units and names

A unit is a named function, a method, or a callback that a binding, wrapper call or label
names; other callbacks fold into their parent. Names read:

- `Class.method` and `Outer.Inner.method` for class members (Python methods under
  `if` or `try` included);
- `Type::method` for Rust, with the implemented type's own name (`Error::from`,
  `[u8; 4]::size`);
- the binding for components and handlers, including through `memo` and `forwardRef`;
- `callee("label")` for a callback passed after a string, such as `it("undoes")` or
  `app.get("/users")`, with call arguments elided (`describe.each(…)("adds %i")`);
- `callee(…)` for an unlabelled top-level callback, and `default` for an anonymous
  default export.

Functions in objects, decorators, heritage clauses and static blocks are not class
members.

## Cross-checks

The walker is checked against independent tools on a pinned corpus (excalidraw, zustand,
bulletproof-react, docusaurus, requests and ripgrep; see `.scripts/corpus.sh`):

- Python's `ast` on requests: 711 of 711 function names, 709 exact spans (the other two
  end at trailing comments), 711 of 711 docstrings.
- The `cognitive_complexity` package (1.3.0) on requests: 96.9% exact, 99.9% in the same
  band. Every difference is topo's nested-function deviation or that package's `elif`
  nesting bug.
- eslint-plugin-sonarjs (4.2.2) on the TypeScript repositories: 88% exact and 99% in the
  same band. sonarjs scores callbacks separately, ignores `||` runs and exempts JSX `&&`,
  so its numbers are lower by design.
- The whitepaper's examples are pinned in `tests/fixtures` for each language:
  `sumOfPrimes` scores 7 as published (6 in Python, which has no labelled `continue`).

## Pitfalls

- Size confounds most code metrics: a file-level score mostly measures length, and
  splitting a file improves it without improving the code. topo rates functions and
  budgets the project's total size instead (see El Emam et al., "The Confounding Effect
  of Class Size on the Validity of Object-Oriented Metrics", IEEE TSE, 2001).
- A band is a reading priority, not a verdict. Long test suites, generated tables and
  reducers are often red for good reasons.
- `score` leaves tests and stories out by default, since long test suites would dominate
  the bands; `--tests` includes them.
- A partially parsed file (`~` in the table) is measured from tree-sitter's recovered
  tree, so its numbers are approximate.

## Dropped metrics

The maintainability index, Halstead volume and cyclomatic complexity are not computed.
The maintainability index is 70–95% explained by the logarithm of file length, Halstead
volume duplicates length, and one complexity metric is enough. Docs are shown but not
rated: there is no evidence that they predict maintainability.

## Sources

- G. Ann Campbell, *Cognitive Complexity: A new way of measuring understandability*,
  SonarSource white paper, version 1.7.
- K. El Emam, S. Benlarbi, N. Goel and S. N. Rai, "The Confounding Effect of Class Size on
  the Validity of Object-Oriented Metrics", *IEEE Transactions on Software Engineering*
  27(7), 2001.
