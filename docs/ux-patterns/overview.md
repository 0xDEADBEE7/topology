# Progressive repository inspection

Use the smallest view that answers the current question. Topo's default invocation is structural inspection.

## 1. Orient yourself

```bash
topo ./src
```

This shows directories, files, and concise file documentation.

## 2. Inspect a file

```bash
topo ./src/visitor.rs
```

This shows the file description and its symbols with line ranges. It is an outline, not a source dump; use it to choose what deserves attention.

## 3. Inspect an implementation

```bash
topo ./src/visitor.rs::visit_expr_while
```

This shows the signature and source range for one symbol.

## 4. Find an unknown location

```bash
topo find visit_expr_while
```

The result lists every matching definition. Pass any result back to `topo` to inspect it.

## 5. Score only when evaluating quality

```bash
topo score ./src --detail --all-metrics
```

Use `score` for measurement and threshold-oriented review, not orientation. Use `topo extract` when another tool needs structured records.

```text
directory → file outline → symbol definition → relevant source
```
