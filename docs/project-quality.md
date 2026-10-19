# Project assessment

Assess TypeScript projects, including React components, with:

```bash
cargo run --quiet -- score /path/to/project/src --detail --all-metrics --no-colour
cargo run --quiet -- /path/to/project/src/App.tsx
cargo run --quiet -- extract /path/to/project
```

Supported extensions are `.rs`, `.py`, `.ts`, `.tsx`, `.mts`, and `.cts`.
TypeScript and TSX use separate grammars. React components, hooks, callbacks,
and class methods participate in the shared complexity reports. Assessment
does not require installing or executing the target project's dependencies.

Scoring and extraction share deterministic source discovery. Hidden entries,
dependency folders (`node_modules`, `vendor`), build folders (`target`, `dist`,
`build`, `bin`), and `coverage` are excluded. Symbolic links are not followed.
Explicitly named directories are scanned even if their name is normally excluded.
Scoring fails on unreadable or invalid supported source and on empty inputs.
Extraction retains its existing best-effort behavior and reports parse errors
on stderr while emitting records for the other files.

Relative imports resolve existing TypeScript files and directory indexes,
including TSX. Package imports and tsconfig aliases remain unresolved. This is
a structural assessment, not TypeScript type checking or React hook-rule linting.
The Python fallback and Halstead token counts remain approximate.

# Development checks

`make check` runs formatting verification, Clippy with warnings denied for all
targets, unit tests, CLI integration tests, and the TypeScript smoke test.
The integration suite extracts this project's `src` directory and enforces:

- module descriptions are present;
- file LoC stays below 300 and function LoC stays at or below 60;
- file NMI stays at or above 20;
- file and function CC and cognitive complexity stay below the red boundary of 15;
- file and function Halstead volume stays below the red boundary of 1000.

Yellow complexity bands remain advisory. These checks use the existing report
thresholds without relaxing them. Documentation follows [the code style](./code-style.md):
describe module responsibilities, public interfaces, and non-obvious helpers.
Tests and routine trait implementations can rely on their names and contracts.

The quality review separated CLI parsing, repository discovery, source metadata,
import resolution, record assembly, report ratings, and tree rendering. It also
fixed TSX grammar selection, nested TypeScript function double counting,
short-circuit decision counting, exported JSDoc recognition, and the off-by-one
error that hid preceding symbol descriptions.
