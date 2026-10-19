//! Builds file outlines and repository descriptions for progressive inspection.

use super::{documentation::module_doc, records, repo_name, symbol_info, SymbolInfo};
use std::fs;

/// Return the indexed symbols belonging to one source file.
pub fn file_outline(path: &str) -> Result<(Option<String>, Vec<SymbolInfo>), String> {
    let root = fs::canonicalize(".").map_err(|e| e.to_string())?;
    let path = fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?;
    let values = records::records(&root, &repo_name(&root), &path)?;
    let description = values.iter().find_map(|value| {
        (value.get("type")?.as_str()? == "file")
            .then(|| {
                value
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned)
            })
            .flatten()
    });
    let symbols = values
        .iter()
        .filter(|value| {
            matches!(
                value.get("type").and_then(|v| v.as_str()),
                Some("function" | "class" | "struct")
            )
        })
        .filter_map(symbol_info)
        .collect();
    Ok((description, symbols))
}

/// File descriptions read from leading comments without parsing, ordered by path.
pub fn file_descriptions(root: &str) -> Result<Vec<(String, Option<String>)>, String> {
    let root = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let mut paths = Vec::new();
    crate::paths::collect(&root, &mut paths)?;
    paths.sort();
    Ok(paths
        .iter()
        .map(|path| {
            let relative = path.strip_prefix(&root).unwrap_or(path).to_string_lossy();
            let description = fs::read_to_string(path)
                .map_err(|error| eprintln!("warning: {relative}: {error}"))
                .ok()
                .and_then(|source| module_doc(&source));
            (relative.into_owned(), description)
        })
        .collect())
}
