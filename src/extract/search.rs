//! Locates matching declarations across repository source files.

use super::{file_records, repo_name, symbol_info, SymbolInfo};
use std::fs;

/// Find all extracted function and class definitions matching a name.
pub fn find(root: &str, query: &str) -> Result<Vec<SymbolInfo>, String> {
    let root = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let repo = repo_name(&root);
    let mut paths = Vec::new();
    crate::paths::collect(&root, &mut paths)?;
    let mut matches = Vec::new();
    for path in paths {
        for value in file_records(&root, &repo, &path) {
            if let Some(symbol) = symbol_info(&value) {
                let path_name = format!("{}::{}", symbol.path, symbol.name);
                if symbol.name == query || symbol.qualified_name == query || path_name == query {
                    matches.push(symbol);
                }
            }
        }
    }
    matches.sort_by(|a, b| a.path.cmp(&b.path).then(a.lines[0].cmp(&b.lines[0])));
    Ok(matches)
}
