//! Builds import records from the imports the walker read, resolving relative ones.

use super::metadata::resolve_import;
use crate::metrics::FileMetrics;
use serde_json::{json, Value};
use std::path::Path;

/// Builds a file's import records, each with the file it names under `root` when relative.
pub(super) fn imports(root: &Path, repo: &str, file: &FileMetrics) -> Vec<Value> {
    let imports = file.imports.iter().enumerate();
    imports
        .map(|(ordinal, import)| {
            json!({
                "id": format!("{}#import:{}:{ordinal}", file.path, import.line),
                "type": "import",
                "repo": repo,
                "path": file.path,
                "line": import.line,
                "source": import.source,
                "imported_name": import.name,
                "resolved_path": resolve_import(root, &file.path, import),
            })
        })
        .collect()
}
