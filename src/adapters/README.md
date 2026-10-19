# Adapter architecture

- `base.rs`: language-neutral `Adapter` interface
- `common.rs`: shared text-to-metrics fallback helpers
- `rust/`: `syn`-based Rust adapter
- `python/`: Python adapter barrel
- `typescript/`: TypeScript and React TSX assessment, with separate declaration,
  syntax-classification, and complexity-counting modules

Register each adapter and its extensions in `src/adapters/mod.rs`. Adapters return
shared `FileMetrics`, so aggregation and reporting do not depend on a language.

TypeScript supports `.ts`, `.mts`, and `.cts`. React components use `.tsx` and
the TSX grammar. Functions, arrows, hooks, callbacks, and class methods receive
the same metrics; nested callbacks are measured independently. JSX conditionals
and short-circuit expressions contribute to complexity. Exported functions and
component bindings inherit their preceding JSDoc.

This is syntax-based assessment. It does not run TypeScript type checking,
ESLint, React Rules of Hooks, or resolve tsconfig aliases. JavaScript `.js` and
`.jsx` adapters are not registered. Imports are currently parsed line by line;
multiline imports and complete imported-name binding are not yet modeled.
