//! Coordinates source extraction into file, import, and symbol records.

mod declarations;
mod documentation;
mod import_syntax;
mod imports;
mod inspection;
mod resolution;
mod search;
mod symbols;
pub(crate) use documentation::module_doc;
pub use inspection::{file_descriptions, file_outline};
pub use search::{find, is_named};
mod metadata;
mod models;
mod records;

use std::{fs, path::Path};

#[derive(Clone, Debug, serde::Deserialize)]
/// Describes a source declaration for tree and symbol inspection.
pub struct SymbolInfo {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub qualified_name: String,
    pub lines: [usize; 2],
    pub description: Option<String>,
}

/// Analyse a repository and emit one JSON record per file and symbol.
pub fn run(root: &str) -> Result<(), String> {
    let target = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let root = target
        .parent()
        .filter(|_| target.is_file())
        .unwrap_or(&target);
    let repo = repo_name(root);
    let mut paths = Vec::new();
    crate::paths::collect(&target, &mut paths)?;
    paths.sort();
    for path in paths {
        for value in file_records(root, &repo, &path) {
            println!(
                "{}",
                serde_json::to_string(&value).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(())
}

/// Builds one file's records, or warns and skips it so a bad file never aborts a command.
fn file_records(root: &Path, repo: &str, path: &Path) -> Vec<serde_json::Value> {
    records::records(root, repo, path).unwrap_or_else(|error| {
        let shown = path.strip_prefix(root).unwrap_or(path);
        eprintln!("warning: {}: skipped: {error}", shown.display());
        Vec::new()
    })
}

fn repo_name(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repository")
        .to_owned()
}
fn symbol_info(value: &serde_json::Value) -> Option<SymbolInfo> {
    serde_json::from_value(value.clone()).ok()
}
