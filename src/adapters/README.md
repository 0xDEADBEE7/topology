# Adapter architecture

- `base.rs`: language-neutral `Adapter` interface
- `common.rs`: shared text-to-metrics fallback helpers
- `rust/`: `syn`-based Rust adapter
- `python/`: Python adapter barrel
- `typescript/`: TypeScript, TSX and JavaScript assessment, with separate declaration,
  syntax-classification, and complexity-counting modules

Register each adapter and its extensions in `src/adapters/mod.rs`. Adapters return
shared `FileMetrics`, so scoring and extraction do not depend on a language.

TypeScript supports `.ts`, `.mts`, and `.cts`. React components use `.tsx` and
the TSX grammar, which also parses JavaScript (`.js`, `.jsx`, `.mjs`, `.cjs`,
reported as `javascript`). Files with syntax errors are measured from the
recovered tree and report `parse_errors`. A function is measured as its own unit
when it has a name: its own, or the binding it is assigned to through parentheses,
`as`, `satisfies`, `!` and wrapper calls (`forwardRef`, `memo`, `useCallback`,
`useMemo`, `lazy`, `observer`; any call at the top level). Class members read
`Class.method`; a callback passed after a string is a unit named like
`it("undoes")`; other unnamed callbacks fold into the enclosing unit. Units and
their bindings inherit the JSDoc above their declaration.

This is syntax-based assessment. It does not run TypeScript type checking,
ESLint, React Rules of Hooks, or resolve tsconfig aliases. Imports are currently parsed line by line;
multiline imports and complete imported-name binding are not yet modeled.
