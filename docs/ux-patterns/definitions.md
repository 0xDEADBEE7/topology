# Find a symbol definition

Use the live CLI when exploring the current checkout:

```bash
topo find compute
```

It returns every matching function, class, or struct with a navigable path and line range. Inspect a result directly:

```bash
topo ./src/metrics.rs::compute
```

Queries can be qualified:

```bash
topo find src/metrics::compute
topo ./src/metrics.rs::compute
```

`find` answers “where is it defined?” and returns coordinates. The default inspection command answers “what does it contain?” or “how is it implemented?” Keep search output compact.

For persistent or scriptable indexes, use `topo extract` with `stash` as described in [store](./store.md). Refresh an index after source edits because stored line ranges can become stale.
