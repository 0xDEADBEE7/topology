//! Coordinates source extraction into file, import, and symbol records.

mod imports;
mod metadata;
mod models;
mod records;

use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct SymbolInfo {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub qualified_name: String,
    pub signature: String,
    pub lines: [usize; 2],
    pub description: Option<String>,
}

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

/// Find all extracted function and class definitions matching a name.
pub fn find(root: &str, query: &str) -> Result<Vec<SymbolInfo>, String> {
    let root = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let repo = repo_name(&root);
    let mut paths = Vec::new();
    walk(&root, &mut paths)?;
    let mut matches = Vec::new();
    for path in paths {
        for value in records::records(&root, &repo, &path)? {
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

/// File descriptions from an extraction snapshot, ordered by path.
pub fn file_descriptions(root: &str) -> Result<Vec<(String, Option<String>)>, String> {
    let root = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let repo = repo_name(&root);
    let mut paths = Vec::new();
    walk(&root, &mut paths)?;
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

/// Analyse a repository and emit one JSON record per file and symbol.
pub fn run(root: &str) -> Result<(), String> {
    let root = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let repo = repo_name(&root);
    let mut paths = Vec::new();
    walk(&root, &mut paths)?;
    paths.sort();
    for path in paths {
        match records::records(&root, &repo, &path) {
            Ok(values) => {
                for value in values {
                    println!(
                        "{}",
                        serde_json::to_string(&value).map_err(|e| e.to_string())?
                    );
                }
            }
            Err(error) => eprintln!("{}: {error}", path.display()),
        }
    }
    Ok(())
}

fn repo_name(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repository")
        .to_owned()
}
fn walk(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.')
            || matches!(
                name,
                "target" | "node_modules" | "vendor" | "dist" | "build" | "bin"
            )
        {
            continue;
        }
        if path.is_dir() {
            walk(&path, paths)?;
        } else if supported(&path) {
            paths.push(path);
        }
    }
    Ok(())
}
fn supported(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs") | Some("py") | Some("ts") | Some("tsx")
    )
}
fn symbol_info(value: &serde_json::Value) -> Option<SymbolInfo> {
    Some(SymbolInfo {
        path: value.get("path")?.as_str()?.to_owned(),
        name: value.get("name")?.as_str()?.to_owned(),
        kind: value.get("kind")?.as_str()?.to_owned(),
        qualified_name: value.get("qualified_name")?.as_str()?.to_owned(),
        signature: value.get("signature")?.as_str()?.to_owned(),
        lines: [
            value.get("lines")?.get(0)?.as_u64()? as usize,
            value.get("lines")?.get(1)?.as_u64()? as usize,
        ],
        description: value
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_owned),
    })
}

#[cfg(test)]
mod tests {
    use super::imports;
    use super::metadata::{module_path, signature};

    #[test]
    fn captures_multiline_signature() {
        let source = "pub fn build(\n    input: &str,\n) -> Result<(), Error> {\n    Ok(())\n}";
        assert_eq!(
            signature("rust", source, 1),
            "pub fn build( input: &str, ) -> Result<(), Error> {"
        );
    }

    #[test]
    fn splits_grouped_rust_imports() {
        let imports = imports::parse("rust", "use crate::{metrics, visit::RawCounts};", 3);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].local_name.as_deref(), Some("metrics"));
        assert_eq!(imports[1].local_name.as_deref(), Some("RawCounts"));
    }

    #[test]
    fn strips_supported_extensions_from_module_paths() {
        assert_eq!(module_path("src/metrics.rs"), "src/metrics");
        assert_eq!(module_path("src/widget.ts"), "src/widget");
    }
}
