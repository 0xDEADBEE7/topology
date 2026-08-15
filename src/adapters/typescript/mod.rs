use super::Adapter;
use crate::{metrics, visit::RawCounts};
use tree_sitter::{Node, Parser};

pub struct TypeScriptAdapter;

impl Adapter for TypeScriptAdapter {
    fn language(&self) -> &'static str {
        "typescript"
    }

    /// Analyse TypeScript declarations and function bodies with tree-sitter.
    fn analyse(&self, path: &str, source: &str) -> Result<metrics::FileMetrics, String> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_typescript::language_typescript())
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

fn collect_declarations(
    node: Node<'_>,
    source: &[u8],
    functions: &mut Vec<RawCounts>,
    classes: &mut Vec<crate::metrics::ClassMetrics>,
) {
    if is_function(node.kind()) {
        functions.push(count_function(node, source));
    } else if is_class(node.kind()) {
        classes.push(crate::metrics::ClassMetrics {
            name: node
                .child_by_field_name("name")
                .and_then(|name| name.utf8_text(source).ok())
                .unwrap_or("<anonymous>")
                .to_owned(),
            line_start: node.start_position().row + 1,
            line_end: node.end_position().row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_declarations(child, source, functions, classes);
    }
}

fn has_file_docstring(root: Node<'_>, source: &[u8]) -> bool {
    root.named_child(0).is_some_and(|node| {
        node.kind() == "comment"
            && node
                .utf8_text(source)
                .is_ok_and(|text| text.trim_start().starts_with("/**"))
    })
}

fn has_preceding_comment(node: Node<'_>, source: &[u8]) -> bool {
    node.prev_named_sibling().is_some_and(|sibling| {
        sibling.kind() == "comment"
            && sibling
                .utf8_text(source)
                .is_ok_and(|text| text.trim_start().starts_with("/**"))
    })
}

fn count_function(node: Node<'_>, source: &[u8]) -> RawCounts {
    let name = function_name(node, source);
    let mut counts = Counts::default();
    count_node(node, node.kind(), 0, source, &mut counts);

    RawCounts {
        name,
        line_start: node.start_position().row + 1,
        line_end: node.end_position().row + 1,
        decisions: 1 + counts.decisions,
        cognitive: counts.cognitive,
        n1: counts.operators,
        n2: counts.operands,
        dn1: counts.operators.max(1),
        dn2: counts.operands.max(1),
        has_docstring: has_preceding_comment(node, source),
    }
}

fn function_name(node: Node<'_>, source: &[u8]) -> String {
    node.child_by_field_name("name")
        .or_else(|| {
            node.parent()
                .and_then(|parent| parent.child_by_field_name("name"))
        })
        .and_then(|name| name.utf8_text(source).ok())
        .map(str::to_owned)
        .unwrap_or_else(|| "<anonymous>".to_owned())
}

#[derive(Default)]
struct Counts {
    decisions: u32,
    cognitive: u32,
    operators: u32,
    operands: u32,
}

fn count_node(node: Node<'_>, root_kind: &str, depth: u32, _source: &[u8], counts: &mut Counts) {
    if node.kind() != root_kind && is_function(node.kind()) {
        return;
    }

    let decision = is_decision(node.kind());
    if decision {
        counts.decisions += 1;
        counts.cognitive += 1 + depth;
    }
    if is_operator(node.kind()) {
        counts.operators += 1;
    }
    if is_operand(node) {
        counts.operands += 1;
    }

    let next_depth = if is_nesting(node.kind()) && decision {
        depth + 1
    } else {
        depth
    };
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        count_node(child, root_kind, next_depth, _source, counts);
    }
}

fn is_class(kind: &str) -> bool {
    matches!(kind, "class_declaration" | "abstract_class_declaration")
}
fn is_function(kind: &str) -> bool {
    matches!(
        kind,
        "function_declaration"
            | "function_expression"
            | "arrow_function"
            | "method_definition"
            | "generator_function_declaration"
            | "generator_function"
    )
}

fn is_decision(kind: &str) -> bool {
    matches!(
        kind,
        "if_statement"
            | "for_statement"
            | "for_in_statement"
            | "for_of_statement"
            | "while_statement"
            | "do_statement"
            | "switch_case"
            | "catch_clause"
            | "ternary_expression"
    )
}

fn is_nesting(kind: &str) -> bool {
    matches!(
        kind,
        "if_statement"
            | "for_statement"
            | "for_in_statement"
            | "for_of_statement"
            | "while_statement"
            | "do_statement"
            | "switch_statement"
            | "catch_clause"
    )
}

fn is_operator(kind: &str) -> bool {
    matches!(
        kind,
        "binary_expression"
            | "unary_expression"
            | "update_expression"
            | "assignment_expression"
            | "augmented_assignment_expression"
            | "call_expression"
            | "new_expression"
            | "member_expression"
            | "ternary_expression"
    )
}

fn is_operand(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "identifier"
            | "property_identifier"
            | "type_identifier"
            | "number"
            | "string"
            | "template_string"
            | "true"
            | "false"
            | "null"
    )
}

#[cfg(test)]
mod tests {
    use super::{Adapter, TypeScriptAdapter};

    const SAMPLE: &str = include_str!("../../../tests/fixtures/typescript/basic.ts");

    #[test]
    fn parses_typescript_functions_and_decisions() {
        let metrics = TypeScriptAdapter
            .analyse("basic.ts", SAMPLE)
            .expect("fixture should parse");

        assert_eq!(metrics.language, "typescript");
        assert_eq!(metrics.loc, 11);
        assert_eq!(metrics.fns.len(), 2);
        assert_eq!(metrics.fns[0].name, "classify");
        assert_eq!(metrics.fns[0].line_start, 1);
        assert!(metrics.fns[0].cc >= 4);
        assert_eq!(metrics.fns[1].name, "render");
        assert!(metrics.fns[1].cc >= 2);
    }

    #[test]
    fn rejects_invalid_typescript() {
        let error = match TypeScriptAdapter.analyse("invalid.ts", "function broken( {") {
            Ok(_) => panic!("invalid source should fail"),
            Err(error) => error,
        };
        assert!(error.contains("parse error"));
    }
}
