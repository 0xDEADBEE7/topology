# Topo UX patterns

Topo is designed for progressive disclosure: start with topology, then move to a file, symbol, or source range only when it becomes relevant.

## Workflows

1. [Use progressive inspection](./overview.md)
2. [Find definitions](./definitions.md)
3. [Inspect a file's exported surface](./exports.md)
4. [Inspect imports and local dependencies](./imports.md)
5. [Create or refresh a repository store](./store.md)
6. [Write topo-friendly code](../code-style.md)

```bash
topo PATH                    # directory tree or file outline
topo PATH::SYMBOL            # symbol signature and implementation
topo find NAME               # all matching definitions
topo score PATH              # code-health metrics
topo extract PATH            # machine-readable JSONL
```

The optional `stash` workflows are useful for persistent, scriptable indexes.
