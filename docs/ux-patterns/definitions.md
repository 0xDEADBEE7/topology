# Find a symbol definition

Use the live CLI when exploring the current checkout:

```bash
topo find function_band
```

It returns every matching function, class, or struct with a navigable path and line range. Inspect a result directly:

```bash
topo ./src/metrics.rs::file
```

Queries can be qualified:

```bash
topo find src/metrics::file
topo ./src/metrics.rs::file
```

`find` answers “where is it defined?” and returns coordinates. The default inspection command answers “what does it contain?” or “how is it implemented?” Keep search output compact.
