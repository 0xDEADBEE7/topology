//! Builds file outlines and repository descriptions for progressive inspection.

use super::{records, repo_name, symbol_info, SymbolInfo};
use crate::syntax::module_doc;
use std::fs;

/// Return the indexed symbols belonging to one source file.
pub fn file_outline(path: &str) -> Result<(Option<String>, Vec<SymbolInfo>), String> {
    let root = fs::canonicalize(".").map_err(|e| e.to_string())?;
    let file = fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?;
    let values =
        records::records(&root, &repo_name(&root), &file).map_err(|e| format!("{path}: {e}"))?;
    let description = values[0]["description"].as_str().map(str::to_owned);
    let symbols = values
        .iter()
        .filter(|value| {
            matches!(
                value["type"].as_str(),
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
