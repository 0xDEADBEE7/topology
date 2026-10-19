//! Normalizes source metadata and resolves local module references.

pub(super) use super::{
    declarations::{is_exported, visibility},
    documentation::docstring,
    resolution::resolve_import,
};
use crate::adapters::ECMA_EXTENSIONS;

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

/// Remove a supported source extension from a relative module path.
pub(super) fn module_path(path: &str) -> &str {
    match path.rsplit_once('.') {
        Some((stem, "rs" | "py")) => stem,
        Some((stem, extension)) if ECMA_EXTENSIONS.contains(&extension) => stem,
        _ => path,
    }
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
