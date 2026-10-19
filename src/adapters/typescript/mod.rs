//! Selects TypeScript and TSX grammars for shared complexity assessment.

use super::Adapter;
use crate::metrics;
use declarations::{collect_declarations, has_file_docstring};
use tree_sitter::{Node, Parser};
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

    /// Analyse TypeScript declarations and function bodies with tree-sitter.
    fn analyse(&self, path: &str, source: &str) -> Result<metrics::FileMetrics, String> {
        let grammar = if std::path::Path::new(path)
            .extension()
            .is_some_and(|ext| ext == "tsx")
        {
            tree_sitter_typescript::LANGUAGE_TSX
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT
        };
        let mut parser = Parser::new();
        parser
            .set_language(&grammar.into())
            .map_err(|error| format!("could not configure TypeScript parser: {error}"))?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "TypeScript parser returned no syntax tree".to_owned())?;

        let root = tree.root_node();
        let mut raw = Vec::new();
        let mut classes = Vec::new();
        collect_declarations(root, source.as_bytes(), &mut raw, &mut classes);
        let functions = raw.iter().map(metrics::compute).collect();
        let mut file = metrics::aggregate(
            self.language(),
            path,
            &raw,
            functions,
            classes,
            has_file_docstring(root, source.as_bytes()),
            source.lines().count(),
        );
        file.parse_errors = syntax_errors(root);
        Ok(file)
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
