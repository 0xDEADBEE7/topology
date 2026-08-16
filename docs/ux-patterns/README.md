# Topo UX patterns

Topo is designed for progressive disclosure: start with topology, then move to a file, symbol, or source range only when it becomes relevant.

## Workflows

1. [Use progressive inspection](./overview.md)
2. [Fast user patterns](./user-patterns.md)
3. [Find definitions](./definitions.md)
4. [Write topo-friendly code](../code-style.md)

```bash
topo PATH                    # directory tree or file outline
topo PATH::SYMBOL            # symbol signature and implementation
topo find NAME               # all matching definitions
topo score PATH              # code-health metrics
topo extract PATH            # machine-readable JSONL
```
