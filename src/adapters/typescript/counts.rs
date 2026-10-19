//! Counts decisions and syntax tokens within individual function bodies.

use super::syntax::{is_decision, is_function, is_nesting, is_operand, is_operator};
use tree_sitter::Node;

#[derive(Default)]
pub(super) struct Counts {
    pub(super) decisions: u32,
    pub(super) cognitive: u32,
    pub(super) operators: u32,
    pub(super) operands: u32,
}

/// Measure a body without charging nested functions to their parent.
pub(super) fn count_node(node: Node<'_>, depth: u32, source: &[u8], counts: &mut Counts) {
    if is_function(node.kind()) {
        return;
    }

    let decision = is_decision(node.kind()) || is_short_circuit(node, source);
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

    let next_depth = if is_nesting(node.kind()) {
        depth + 1
    } else {
        depth
    };
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        count_node(child, next_depth, source, counts);
    }
}

/// Logical operators introduce alternate execution paths, including JSX guards.
fn is_short_circuit(node: Node<'_>, source: &[u8]) -> bool {
    node.child_by_field_name("operator")
        .and_then(|operator| operator.utf8_text(source).ok())
        .is_some_and(|operator| matches!(operator, "&&" | "||" | "??"))
}
