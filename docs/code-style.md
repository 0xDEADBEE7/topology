# Writing topo-friendly code

Topo exposes topology through file and symbol descriptions. Clear documentation makes the first inspection useful without opening source files.

## Describe responsibility, not mechanics

A good docstring answers what the file or symbol is responsible for:

```rust
/// Walks one function body and accumulates raw complexity counts.
struct BlockVisitor { ... }

/// Resolve a local import relative to the current source file.
fn resolve_import(...) { ... }
```

Avoid comments that merely repeat the name or implementation:

```rust
/// Resolves an import.
/// Increments depth by one.
```

## Keep descriptions concise

Prefer one short sentence or two compact clauses. The description should remain useful when rendered beside a tree entry:

```text
src/adapters/          # Language-specific analysis implementations
src/extract.rs         # Emits file, import, and symbol records
```

A useful description is:

- specific about responsibility;
- written in present tense;
- free of implementation detail unless it defines behavior;
- small enough to scan in a tree;
- stable when the implementation changes.

## Document the right boundaries

Prioritize documentation for:

1. files and modules;
2. public types and traits;
3. exported functions;
4. non-obvious private helpers;
5. invariants, failure modes, and side effects.

Use signatures and tests for detail. Use docstrings to provide orientation and intent.

## Treat topology and prose as one interface

A good file name says where to look. A good docstring says why it exists. A good symbol name says what to search for. Together they let a reader move through the progressive workflow:

```text
topo ./src → topo ./src/extract.rs → topo ./src/extract.rs::records
```

Documentation should make each step more informative without requiring a source dump.