//! Parses imports and enriches them with local resolution metadata.

use super::import_syntax::{javascript_import, python_from, python_import, rust_entries};
use super::metadata::{language_name, module_path, resolve_import};
use super::models::ImportRecord;
use std::path::Path;

/// Builds import records enriched with repository-local targets.
pub(super) fn imports(
    root: &Path,
    repo: &str,
    relative: &str,
    language: &str,
    source: &str,
) -> Vec<ImportRecord> {
    source
        .lines()
        .enumerate()
        .flat_map(|(index, line)| parse(language, line, index + 1))
        .enumerate()
        .map(|(ordinal, mut import)| {
            import.id = format!("{}#import:{}:{}", relative, import.line, ordinal);
            import.repo = repo.to_owned();
            import.path = relative.to_owned();
            import.language = language_name(language);
            import.resolved_path = resolve_import(root, relative, language, &import.source);
            import.resolved_symbol = import.resolved_path.as_ref().and_then(|path| {
                import
                    .imported_name
                    .as_ref()
                    .map(|name| format!("{}::{name}", module_path(path)))
            });
            import.resolution = if import.resolved_path.is_some() {
                "local"
            } else {
                "unresolved"
            };
            import
        })
        .collect()
}

/// Converts a supported import statement into unresolved records.
pub(super) fn parse(language: &str, line: &str, line_number: usize) -> Vec<ImportRecord> {
    let text = line.trim();
    let entries = if language == "rust" && text.starts_with("use ") {
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
    };
    entries
        .into_iter()
        .map(|(source, local_name, imported_name)| ImportRecord {
            id: String::new(),
            record_type: "import",
            repo: String::new(),
            path: String::new(),
            language: language_name(language),
            line: line_number,
            source,
            local_name,
            imported_name,
            resolved_path: None,
            resolved_symbol: None,
            resolution: "unresolved",
        })
        .collect()
}
