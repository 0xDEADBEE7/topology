//! Selects TypeScript and TSX grammars for shared complexity assessment.

use super::Adapter;
use crate::metrics;
use declarations::collect_declarations;
use tree_sitter::{Node, Parser};
use tree_sitter_typescript::{LANGUAGE_TSX, LANGUAGE_TYPESCRIPT};
mod counts;
mod declarations;
mod syntax;
#[cfg(test)]
mod tests;

/// Assesses TypeScript modules and React components with the matching grammar.
pub struct TypeScriptAdapter;

impl Adapter for TypeScriptAdapter {
    fn language(&self) -> &'static str {
        "typescript"
    }

    /// Analyse TypeScript and JavaScript declarations and function bodies with tree-sitter.
    fn analyse(&self, path: &str, source: &str) -> Result<metrics::FileMetrics, String> {
        let extension = std::path::Path::new(path).extension();
        let (language, grammar) = match extension.and_then(|ext| ext.to_str()) {
            Some("ts" | "mts" | "cts") => (self.language(), LANGUAGE_TYPESCRIPT),
            Some("tsx") => (self.language(), LANGUAGE_TSX),
            _ => ("javascript", LANGUAGE_TSX),
        };
        let mut parser = Parser::new();
        parser
            .set_language(&grammar.into())
            .map_err(|error| format!("could not configure TypeScript parser: {error}"))?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "TypeScript parser returned no syntax tree".to_owned())?;

        let root = tree.root_node();
        let mut fns = Vec::new();
        let mut classes = Vec::new();
        collect_declarations(root, source.as_bytes(), &mut fns, &mut classes);
        Ok(metrics::FileMetrics {
            language,
            path: path.to_owned(),
            loc: source.lines().count(),
            has_docstring: crate::extract::module_doc(source).is_some(),
            parse_errors: syntax_errors(root),
            fns,
            classes,
        })
    }
}

/// Counts the ERROR and MISSING nodes left by tree-sitter's error recovery.
fn syntax_errors(node: Node<'_>) -> usize {
    if node.is_error() || node.is_missing() {
        return 1;
    }
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| child.has_error())
        .map(syntax_errors)
        .sum()
}
