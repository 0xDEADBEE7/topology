# Find a symbol definition

Use the live CLI when exploring the current checkout:

```bash
topo find function_band
```

It returns every matching function, class, or struct with a navigable path and line range. Inspect a result directly:

```bash
topo ./src/report.rs::function_band
```

Queries can be qualified:

```bash
topo find src/report::function_band
topo ./src/report.rs::function_band
```

`find` answers “where is it defined?” and returns coordinates. The default inspection command answers “what does it contain?” or “how is it implemented?” Keep search output compact.
