//! Parses imports and enriches them with local resolution metadata.

use super::import_syntax::{javascript_import, python_from, python_import, rust_entries};
use super::metadata::{language_name, module_path, resolve_import};
use serde_json::{json, Value};
use std::path::Path;

/// Builds import records enriched with repository-local targets.
pub(super) fn imports(
    root: &Path,
    repo: &str,
    relative: &str,
    language: &str,
    source: &str,
) -> Vec<Value> {
    let lines = source.lines().enumerate();
    let entries = lines.flat_map(|(index, line)| {
        parse(language, line)
            .into_iter()
            .map(move |e| (index + 1, e))
    });
    entries
        .enumerate()
        .map(|(ordinal, (line, (source, local_name, imported_name)))| {
            let resolved = resolve_import(root, relative, language, &source);
            let symbol = resolved.as_ref().and_then(|path| {
                (imported_name.as_ref()).map(|name| format!("{}::{name}", module_path(path)))
            });
            json!({
                "id": format!("{relative}#import:{line}:{ordinal}"),
                "type": "import",
                "repo": repo,
                "path": relative,
                "language": language_name(language),
                "line": line,
                "source": source,
                "local_name": local_name,
                "imported_name": imported_name,
                "resolution": if resolved.is_some() { "local" } else { "unresolved" },
                "resolved_path": resolved,
                "resolved_symbol": symbol,
            })
        })
        .collect()
}

/// Converts a supported import statement into source, local name and imported name.
pub(super) fn parse(language: &str, line: &str) -> Vec<(String, Option<String>, Option<String>)> {
    let text = line.trim();
    if language == "rust" && text.starts_with("use ") {
        rust_entries(text)
    } else if language == "python" && text.starts_with("from ") {
        python_from(text)
    } else if language == "python" && text.starts_with("import ") {
        python_import(text)
    } else if (language == "typescript" || language == "javascript") && text.starts_with("import ")
    {
        javascript_import(text)
    } else {
        Vec::new()
    }
}
