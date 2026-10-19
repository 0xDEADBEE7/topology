//! Locates matching declarations across repository source files.

use super::{file_records, repo_name, symbol_info, SymbolInfo};
use std::fs;

/// Find all extracted function and class definitions matching a name.
pub fn find(root: &str, query: &str) -> Result<Vec<SymbolInfo>, String> {
    let root = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let repo = repo_name(&root);
    let mut paths = Vec::new();
    crate::paths::collect(&root, &mut paths)?;
    let mut matches: Vec<_> = (paths.iter())
        .flat_map(|path| file_records(&root, &repo, path))
        .filter_map(|value| symbol_info(&value))
        .filter(|symbol| {
            is_named(&symbol.name, query)
                || symbol.qualified_name == query
                || format!("{}::{}", symbol.path, symbol.name) == query
        })
        .collect();
    matches.sort_by(|a, b| a.path.cmp(&b.path).then(a.lines[0].cmp(&b.lines[0])));
    Ok(matches)
}

/// Matches a symbol by its full name or by the last segment of `View.render` or `Type::method`.
pub fn is_named(symbol: &str, query: &str) -> bool {
    symbol == query || symbol.rsplit(['.', ':']).next() == Some(query)
}
