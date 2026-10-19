//! Selects TypeScript and TSX grammars for shared complexity assessment.

use super::Adapter;
use crate::metrics;
use declarations::{collect_declarations, has_file_docstring};
use tree_sitter::Parser;
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
        let mut parser = Parser::new();
        parser
            .set_language(
                if std::path::Path::new(path)
                    .extension()
                    .is_some_and(|ext| ext == "tsx")
                {
                    tree_sitter_typescript::language_tsx()
                } else {
                    tree_sitter_typescript::language_typescript()
                },
            )
            .map_err(|error| format!("could not configure TypeScript parser: {error}"))?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "TypeScript parser returned no syntax tree".to_owned())?;

        if tree.root_node().has_error() {
            return Err("parse error in TypeScript source".to_owned());
        }

        let mut raw = Vec::new();
        let mut classes = Vec::new();
        collect_declarations(tree.root_node(), source.as_bytes(), &mut raw, &mut classes);
        let functions = raw.iter().map(metrics::compute).collect();
        Ok(metrics::aggregate(
            self.language(),
            path,
            &raw,
            functions,
            classes,
            has_file_docstring(tree.root_node(), source.as_bytes()),
            source.lines().count(),
        ))
    }
}
