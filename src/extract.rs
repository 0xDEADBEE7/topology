//! Coordinates source extraction into file, import, and symbol records.

mod models;

use models::{Export, FileRecord, ImportRecord, Metrics, SymbolMetrics, SymbolRecord};

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
    let repo = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repository")
        .to_owned();
    let values = records(&root, &repo, &path)?;
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
            value.get("type").and_then(|v| v.as_str()) == Some("function")
                || value.get("type").and_then(|v| v.as_str()) == Some("class")
        })
        .filter_map(symbol_info)
        .collect();
    Ok((description, symbols))
}

/// Find all extracted function and class definitions matching a name.
pub fn find(root: &str, query: &str) -> Result<Vec<SymbolInfo>, String> {
    let root_path = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let repo = root_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repository")
        .to_owned();
    let mut paths = Vec::new();
    walk(&root_path, &mut paths)?;
    let mut matches = Vec::new();
    for path in paths {
        for value in records(&root_path, &repo, &path)? {
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

/// Convert one serialized symbol record into the public outline shape.
fn symbol_info(value: &serde_json::Value) -> Option<SymbolInfo> {
    let kind = value.get("kind")?.as_str()?.to_owned();
    Some(SymbolInfo {
        path: value.get("path")?.as_str()?.to_owned(),
        name: value.get("name")?.as_str()?.to_owned(),
        kind,
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
/// File descriptions from an extraction snapshot, ordered by path.
pub fn file_descriptions(root: &str) -> Result<Vec<(String, Option<String>)>, String> {
    let root = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let repo = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repository")
        .to_owned();
    let mut paths = Vec::new();
    walk(&root, &mut paths)?;
    paths.sort();
    let mut files = Vec::new();
    for path in paths {
        for record in records(&root, &repo, &path)? {
            if record.get("type").and_then(serde_json::Value::as_str) == Some("file") {
                let path = record
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let description = record
                    .get("description")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned);
                files.push((path, description));
                break;
            }
        }
    }
    Ok(files)
}

/// Analyse a repository and emit one JSON record per file and symbol.
pub fn run(root: &str) -> Result<(), String> {
    let root = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let repo = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repository")
        .to_owned();
    let mut paths = Vec::new();
    walk(&root, &mut paths)?;
    paths.sort();

    for path in paths {
        match records(&root, &repo, &path) {
            Ok(records) => {
                for record in records {
                    println!(
                        "{}",
                        serde_json::to_string(&record).map_err(|e| e.to_string())?
                    );
                }
            }
            Err(error) => eprintln!("{}: {error}", path.display()),
        }
    }
    Ok(())
}

/// Walk a repository while excluding generated and dependency directories.
fn walk(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))?;
    for entry in entries {
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

/// Report whether a path uses one of the supported source extensions.
fn supported(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs") | Some("py") | Some("ts") | Some("tsx")
    )
}

/// Build the file and symbol records used by the stash-backed views.
/// Parse one source file and assemble its file, import, and symbol records.
fn records(root: &Path, repo: &str, path: &Path) -> Result<Vec<serde_json::Value>, String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    let relative = relative.strip_prefix("./").unwrap_or(&relative).to_owned();
    let adapter = crate::adapters::for_path(path.to_string_lossy().as_ref())?;
    let analysed = adapter.analyse(&relative, &source)?;
    let language = analysed.language;
    let imports: Vec<ImportRecord> = source
        .lines()
        .enumerate()
        .flat_map(|(index, line)| import_records(language, line, index + 1))
        .enumerate()
        .map(|(ordinal, mut import)| {
            import.id = format!("{}#import:{}:{}", relative, import.line, ordinal);
            import.repo = repo.to_owned();
            import.path = relative.clone();
            import.language = language_name(language);
            import.resolved_path = resolve_import(root, &relative, language, &import.source);
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
        .collect();
    let import_ids: Vec<String> = imports.iter().map(|import| import.id.clone()).collect();
    let exports = analysed
        .fns
        .iter()
        .filter_map(|function| {
            if is_exported(language, &source, function.line_start) {
                Some(Export {
                    signature: signature(language, &source, function.line_start),
                    lines: [function.line_start, function.line_end],
                    description: docstring(language, &source, Some(function.line_start)),
                })
            } else {
                None
            }
        })
        .chain(analysed.classes.iter().filter_map(|class| {
            if is_exported(language, &source, class.line_start) {
                Some(Export {
                    signature: signature(language, &source, class.line_start),
                    lines: [class.line_start, class.line_end],
                    description: docstring(language, &source, Some(class.line_start)),
                })
            } else {
                None
            }
        }))
        .collect();
    let file_record = FileRecord {
        id: relative.clone(),
        record_type: "file",
        repo: repo.to_owned(),
        path: relative.clone(),
        description: docstring(language, &source, None),
        imports: import_ids,
        exports,
        metrics: Metrics {
            language,
            loc: analysed.loc,
            cc: analysed.cc,
            cognitive: analysed.cognitive,
            halstead: analysed.halstead,
            nmi: analysed.nmi,
        },
    };
    let mut output = vec![serde_json::to_value(file_record).map_err(|e| e.to_string())?];
    for import in &imports {
        output.push(serde_json::to_value(import).map_err(|e| e.to_string())?);
    }

    for function in &analysed.fns {
        output.push(
            serde_json::to_value(SymbolRecord {
                id: format!(
                    "{}#function:{}:{}",
                    relative, function.name, function.line_start
                ),
                record_type: "function",
                repo: repo.to_owned(),
                path: relative.clone(),
                name: function.name.clone(),
                kind: "function",
                qualified_name: qualified_name(&relative, &function.name),
                language,
                visibility: visibility(language, &source, function.line_start),
                parent: None,
                signature: signature(language, &source, function.line_start),
                lines: [function.line_start, function.line_end],
                description: docstring(language, &source, Some(function.line_start)),
                metrics: Some(SymbolMetrics {
                    loc: function.loc,
                    cc: function.cc,
                    cognitive: function.cognitive,
                    halstead: function.halstead,
                }),
            })
            .map_err(|e| e.to_string())?,
        );
    }

    for class in &analysed.classes {
        output.push(
            serde_json::to_value(SymbolRecord {
                id: format!("{}#class:{}:{}", relative, class.name, class.line_start),
                record_type: "class",
                repo: repo.to_owned(),
                path: relative.clone(),
                name: class.name.clone(),
                kind: class_kind(language),
                qualified_name: qualified_name(&relative, &class.name),
                language,
                visibility: visibility(language, &source, class.line_start),
                parent: None,
                signature: signature(language, &source, class.line_start),
                lines: [class.line_start, class.line_end],
                description: docstring(language, &source, Some(class.line_start)),
                metrics: None,
            })
            .map_err(|e| e.to_string())?,
        );
    }

    Ok(output)
}

/// Parse one source line into zero or more language-specific imports.
fn import_records(language: &str, line: &str, line_number: usize) -> Vec<ImportRecord> {
    let text = line.trim();
    let entries: Vec<(String, Option<String>, Option<String>)> = if language == "rust"
        && text.starts_with("use ")
    {
        let value = match text.strip_prefix("use ") {
            Some(value) => value.trim_end_matches(';').trim(),
            None => return Vec::new(),
        };
        if let Some((prefix, members)) = value.split_once('{') {
            members
                .trim_end_matches('}')
                .split(',')
                .filter_map(|member| {
                    let member = member.trim();
                    if member.is_empty() {
                        return None;
                    }
                    let (name, local) = member.split_once(" as ").unwrap_or((member, member));
                    let imported = name.rsplit("::").next().unwrap_or(name);
                    let local = if local == member { imported } else { local };
                    Some((
                        format!("{}::{name}", prefix.trim_end_matches("::")),
                        Some(local.to_owned()),
                        Some(imported.to_owned()),
                    ))
                })
                .collect()
        } else {
            let name = value.rsplit("::").next().unwrap_or(value).to_owned();
            vec![(value.to_owned(), Some(name.clone()), Some(name))]
        }
    } else if language == "python" && text.starts_with("from ") {
        let mut parts = text.split_whitespace();
        let module = match parts.nth(1) {
            Some(value) => value,
            None => return Vec::new(),
        };
        parts
            .next()
            .and_then(|_| parts.next())
            .map(|name| {
                vec![(
                    format!("{module}.{name}"),
                    Some(name.to_owned()),
                    Some(name.to_owned()),
                )]
            })
            .unwrap_or_default()
    } else if language == "python" && text.starts_with("import ") {
        text.strip_prefix("import ")
            .and_then(|value| value.split_whitespace().next())
            .map(|value| {
                let local = value.rsplit('.').next().unwrap_or(value).to_owned();
                vec![(value.to_owned(), Some(local), Some(value.to_owned()))]
            })
            .unwrap_or_default()
    } else if (language == "typescript" || language == "javascript") && text.starts_with("import ")
    {
        text.rsplit(" from ")
            .next()
            .map(|source| {
                vec![(
                    source.trim().trim_matches(['\'', '\"', ';']).to_owned(),
                    None,
                    None,
                )]
            })
            .unwrap_or_default()
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

/// Remove matching Python triple-quote delimiters from a documentation line.
fn quoted_doc(line: &str) -> Option<String> {
    let quote = if line.starts_with("\"\"\"") {
        "\"\"\""
    } else {
        "'''"
    };
    let text = line.strip_prefix(quote)?;
    Some(text.strip_suffix(quote).unwrap_or(text).trim().to_owned())
}

/// Normalize an adapter language identifier for serialized output.
fn language_name(language: &str) -> &'static str {
    match language {
        "rust" => "rust",
        "python" => "python",
        "typescript" => "typescript",
        "javascript" => "javascript",
        _ => "unknown",
    }
}

/// Resolve an import against files local to the repository.
fn resolve_import(root: &Path, current: &str, language: &str, source: &str) -> Option<String> {
    let candidates = if language == "rust" {
        let module = source.strip_prefix("crate::")?.split("::").next()?;
        vec![
            root.join("src").join(format!("{module}.rs")),
            root.join("src").join(module).join("mod.rs"),
        ]
    } else if language == "python" {
        let module = source
            .rsplit_once('.')
            .map_or(source, |(module, _)| module)
            .replace('.', "/");
        let base = root.join(Path::new(&module));
        vec![base.with_extension("py"), base.join("__init__.py")]
    } else if language == "typescript" || language == "javascript" {
        if !source.starts_with('.') {
            return None;
        }
        let base = Path::new(current)
            .parent()
            .unwrap_or(Path::new("."))
            .join(source);
        vec![
            root.join(&base).with_extension("ts"),
            root.join(&base).join("index.ts"),
        ]
    } else {
        return None;
    };

    candidates.into_iter().find_map(|candidate| {
        candidate
            .exists()
            .then(|| candidate.strip_prefix(root).ok())
            .flatten()
            .map(|path| path.to_string_lossy().to_string())
    })
}

/// Remove a supported source extension from a relative module path.
fn module_path(path: &str) -> &str {
    path.strip_suffix(".rs")
        .or_else(|| path.strip_suffix(".py"))
        .or_else(|| path.strip_suffix(".ts"))
        .or_else(|| path.strip_suffix(".tsx"))
        .unwrap_or(path)
}

/// Build the stable symbol identifier used by extraction consumers.
fn qualified_name(path: &str, name: &str) -> String {
    format!("{}::{name}", module_path(path))
}

/// Map an adapter language to the exported record kind.
fn class_kind(language: &str) -> &'static str {
    match language {
        "rust" => "struct",
        "typescript" => "class",
        _ => "class",
    }
}

/// Infer source visibility from the declaration line.
fn visibility(language: &str, source: &str, line: usize) -> &'static str {
    let text = source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim_start();
    if (language == "rust" && text.starts_with("pub ")) || text.starts_with("export ") {
        "public"
    } else if language == "python" {
        if text.contains("__") {
            "private"
        } else {
            "public"
        }
    } else {
        "private"
    }
}
/// Collect a declaration signature across continuation lines.
fn signature(language: &str, source: &str, line: usize) -> String {
    let mut result = Vec::new();
    for text in source.lines().skip(line.saturating_sub(1)).take(30) {
        let text = text.trim();
        result.push(text);
        let complete = if language == "python" {
            text.ends_with(':')
        } else {
            text.contains('{') || text.ends_with(';') || text.ends_with("=>")
        };
        if complete {
            break;
        }
    }
    result.join(" ")
}

/// Determine whether a declaration is part of the language's public surface.
fn is_exported(language: &str, source: &str, line: usize) -> bool {
    let text = source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim_start();
    match language {
        "rust" => text.starts_with("pub "),
        "typescript" | "javascript" => {
            text.starts_with("export ")
                || source
                    .lines()
                    .any(|l| l.contains("export") && l.contains(text))
        }
        "python" => true,
        _ => false,
    }
}

/// Extract documentation immediately preceding a file or symbol.
fn docstring(language: &str, source: &str, symbol_line: Option<usize>) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    if language == "python" {
        let start = symbol_line.map_or(0, |line| line.saturating_sub(1));
        let candidate = lines[start..]
            .iter()
            .take(12)
            .map(|line| line.trim())
            .skip_while(|line| line.is_empty())
            .find(|line| line.starts_with("\"\"\"") || line.starts_with("'''"));
        return candidate.and_then(quoted_doc);
    }

    if symbol_line.is_none() {
        let mut docs = Vec::new();
        for line in &lines {
            let line = line.trim();
            if let Some(text) = line.strip_prefix("//!") {
                docs.push(text.trim().to_owned());
            } else if let Some(text) = line.strip_prefix("/**") {
                docs.push(text.trim().trim_end_matches("*/").trim().to_owned());
                break;
            } else if !line.is_empty() {
                break;
            }
        }
        docs.dedup();
        return (!docs.is_empty()).then(|| docs.join(" "));
    }

    let end = symbol_line.unwrap_or(0).min(lines.len());
    let mut docs = Vec::new();
    let mut in_block = false;
    for line in lines[..end].iter().rev() {
        let line = line.trim();
        if line == "*/" {
            in_block = true;
            continue;
        }
        let text = if language == "rust" {
            line.strip_prefix("///")
                .or_else(|| line.strip_prefix("//!"))
        } else if in_block {
            line.strip_prefix('*').map(str::trim)
        } else {
            line.strip_prefix("/**").or_else(|| line.strip_prefix("//"))
        };
        match text {
            Some(text) => {
                docs.push(text.trim().trim_end_matches("*/").trim().to_owned());
                in_block = in_block || line.starts_with("/**");
            }
            None if docs.is_empty() => break,
            None => break,
        }
    }
    (!docs.is_empty()).then(|| {
        docs.reverse();
        docs.join(" ")
    })
}

#[cfg(test)]
mod tests {
    use super::{import_records, module_path, signature};

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
        let imports = import_records("rust", "use crate::{metrics, visit::RawCounts};", 3);
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
