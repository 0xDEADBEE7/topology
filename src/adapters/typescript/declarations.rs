//! Discovers functions, classes, and their documentation boundaries.

use super::{
    counts::{count_node, Counts},
    syntax::{is_class, is_function},
};
use crate::visit::RawCounts;
use tree_sitter::Node;

/// Collects each function independently, including nested callbacks.
pub(super) fn collect_declarations(
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

/// Recognizes JSDoc attached to the containing declaration.
fn has_preceding_comment(node: Node<'_>, source: &[u8]) -> bool {
    node.prev_named_sibling().is_some_and(|sibling| {
        sibling.kind() == "comment"
            && sibling
                .utf8_text(source)
                .is_ok_and(|text| text.trim_start().starts_with("/**"))
    })
}

/// Measures the body while retaining its declaration's source extent.
fn count_function(node: Node<'_>, source: &[u8]) -> RawCounts {
    let name = function_name(node, source);
    let mut counts = Counts::default();
    if let Some(body) = node.child_by_field_name("body") {
        count_node(body, 0, source, &mut counts);
    }

    RawCounts {
        name,
        line_start: documented_node(node).start_position().row + 1,
        line_end: node.end_position().row + 1,
        decisions: 1 + counts.decisions,
        cognitive: counts.cognitive,
        n1: counts.operators,
        n2: counts.operands,
        dn1: counts.operators.max(1),
        dn2: counts.operands.max(1),
        has_docstring: has_preceding_comment(documented_node(node), source),
    }
}

/// Uses the function's own name or its directly assigned binding.
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

/// Follow wrappers so exported components inherit their JSDoc.
fn documented_node(mut node: Node<'_>) -> Node<'_> {
    while let Some(parent) = node.parent() {
        if !matches!(
            parent.kind(),
            "variable_declarator"
                | "lexical_declaration"
                | "variable_declaration"
                | "export_statement"
        ) {
            break;
        }
        node = parent;
    }
    node
}
