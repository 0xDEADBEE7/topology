# Assay UX patterns

Assay is designed for progressive disclosure: start with topology, then move to a file, symbol, or source range only when it becomes relevant.

## Workflows

1. [Use progressive inspection](./overview.md)
2. [Find definitions](./definitions.md)
3. [Inspect a file's exported surface](./exports.md)
4. [Inspect imports and local dependencies](./imports.md)
5. [Create or refresh a repository store](./store.md)
6. [Write assay-friendly code](../code-style.md)

```bash
assay PATH                    # directory tree or file outline
assay PATH::SYMBOL            # symbol signature and implementation
assay find NAME               # all matching definitions
assay score PATH              # code-health metrics
assay extract PATH            # machine-readable JSONL
```

The optional `stash` workflows are useful for persistent, scriptable indexes.
