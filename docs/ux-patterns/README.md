# Assay UX patterns

These patterns use `assay` and `stash` directly. The [`Makefile`](../../Makefile) and `.scripts/` directory provide equivalent project-specific examples.

## Workflows

1. [Create or refresh a repository store](./store.md)
2. [Orient yourself in a repository](./overview.md)
3. [Inspect a file's exported surface](./exports.md)
4. [Locate a symbol definition](./definitions.md)
5. [Inspect imports and local dependencies](./imports.md)

All queries operate on a stash directory selected with `STASH_DIR`. Keep the store beside the repository unless you need a separate index.
