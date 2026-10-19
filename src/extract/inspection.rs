//! Builds file outlines and repository descriptions for progressive inspection.

use super::{records, repo_name, symbol_info, SymbolInfo};
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
                Some("function") | Some("class")
            )
        })
        .filter_map(symbol_info)
        .collect();
    Ok((description, symbols))
}

/// File descriptions from an extraction snapshot, ordered by path.
pub fn file_descriptions(root: &str) -> Result<Vec<(String, Option<String>)>, String> {
    let root = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let repo = repo_name(&root);
    let mut paths = Vec::new();
    crate::paths::collect(&root, &mut paths)?;
    paths.sort();
    let mut files = Vec::new();
    for path in paths {
        for record in records::records(&root, &repo, &path)? {
            if record.get("type").and_then(serde_json::Value::as_str) == Some("file") {
                files.push((
                    record
                        .get("path")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                    record
                        .get("description")
                        .and_then(|value| value.as_str())
                        .map(str::to_owned),
                ));
                break;
            }
        }
    }
    Ok(files)
}
