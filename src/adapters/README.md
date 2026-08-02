# Adapter architecture

- `base.rs`: language-neutral `Adapter` interface
- `common.rs`: shared text-to-metrics fallback helpers
- `rust/`: `syn`-based Rust adapter
- `python/`: Python adapter barrel
- `typescript/`: TypeScript adapter barrel

Register each adapter and its extensions in `src/adapters/mod.rs`. Adapters return
shared `FileMetrics`, so aggregation and reporting do not depend on a language.
