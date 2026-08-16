//! Normalizes source metadata and resolves local module references.

use std::path::Path;

/// Normalize an adapter language identifier for serialized output.
pub(super) fn language_name(language: &str) -> &'static str {
    match language {
        "rust" => "rust",
        "python" => "python",
        "typescript" => "typescript",
        "javascript" => "javascript",
        _ => "unknown",
    }
}

/// Resolve an import against files local to the repository.
pub(super) fn resolve_import(
    root: &Path,
    current: &str,
    language: &str,
    source: &str,
) -> Option<String> {
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
pub(super) fn module_path(path: &str) -> &str {
    path.strip_suffix(".rs")
        .or_else(|| path.strip_suffix(".py"))
        .or_else(|| path.strip_suffix(".ts"))
        .or_else(|| path.strip_suffix(".tsx"))
        .unwrap_or(path)
}

/// Build the stable symbol identifier used by extraction consumers.
pub(super) fn qualified_name(path: &str, name: &str) -> String {
    format!("{}::{name}", module_path(path))
}

/// Map an adapter language to the exported record kind.
pub(super) fn class_kind(language: &str) -> &'static str {
    if language == "rust" {
        "struct"
    } else {
        "class"
    }
}

/// Infer source visibility from the declaration line.
pub(super) fn visibility(language: &str, source: &str, line: usize) -> &'static str {
    let text = source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim_start();
    if (language == "rust" && text.starts_with("pub ")) || text.starts_with("export ") {
        "public"
    } else if language == "python" && text.contains("__") {
        "private"
    } else if language == "python" {
        "public"
    } else {
        "private"
    }
}

/// Collect a declaration signature across continuation lines.
pub(super) fn signature(language: &str, source: &str, line: usize) -> String {
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
pub(super) fn is_exported(language: &str, source: &str, line: usize) -> bool {
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
pub(super) fn docstring(
    language: &str,
    source: &str,
    symbol_line: Option<usize>,
) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    if language == "python" {
        let start = symbol_line.map_or(0, |line| line.saturating_sub(1));
        return lines[start..]
            .iter()
            .take(12)
            .map(|line| line.trim())
            .skip_while(|line| line.is_empty())
            .find(|line| line.starts_with("\"\"\"") || line.starts_with("'''"))
            .and_then(quoted_doc);
    }
    if symbol_line.is_none() {
        return file_docs(&lines);
    }
    preceding_docs(&lines, symbol_line.unwrap_or(0), language)
}

fn quoted_doc(line: &str) -> Option<String> {
    let quote = if line.starts_with("\"\"\"") {
        "\"\"\""
    } else {
        "'''"
    };
    let text = line.strip_prefix(quote)?;
    Some(text.strip_suffix(quote).unwrap_or(text).trim().to_owned())
}

fn file_docs(lines: &[&str]) -> Option<String> {
    let mut docs = Vec::new();
    for line in lines {
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
    (!docs.is_empty()).then(|| docs.join(" "))
}

fn preceding_docs(lines: &[&str], end: usize, language: &str) -> Option<String> {
    let mut docs = Vec::new();
    let mut block = false;
    for line in lines[..end.min(lines.len())].iter().rev() {
        let line = line.trim();
        if line == "*/" {
            block = true;
            continue;
        }
        let text = if language == "rust" {
            line.strip_prefix("///")
                .or_else(|| line.strip_prefix("//!"))
        } else if block {
            line.strip_prefix('*').map(str::trim)
        } else {
            line.strip_prefix("/**").or_else(|| line.strip_prefix("//"))
        };
        match text {
            Some(text) => {
                docs.push(text.trim().trim_end_matches("*/").trim().to_owned());
                block |= line.starts_with("/**");
            }
            None => break,
        }
    }
    (!docs.is_empty()).then(|| {
        docs.reverse();
        docs.join(" ")
    })
}
