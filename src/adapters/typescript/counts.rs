//! Sums cognitive complexity within individual function bodies.

use super::syntax::{is_decision, is_function, is_nesting};
use tree_sitter::Node;

/// Sums a body's cognitive increments without charging nested functions to it.
pub(super) fn cognitive(node: Node<'_>, depth: u32, source: &[u8]) -> u32 {
    if is_function(node.kind()) {
        return 0;
    }
    let decision = is_decision(node.kind()) || is_short_circuit(node, source);
    let own = if decision { 1 + depth } else { 0 };
    let depth = depth + u32::from(is_nesting(node.kind()));
    let mut cursor = node.walk();
    let children = node.named_children(&mut cursor);
    own + children
        .map(|child| cognitive(child, depth, source))
        .sum::<u32>()
}

/// Logical operators introduce alternate execution paths, including JSX guards.
fn is_short_circuit(node: Node<'_>, source: &[u8]) -> bool {
    node.child_by_field_name("operator")
        .and_then(|operator| operator.utf8_text(source).ok())
        .is_some_and(|operator| matches!(operator, "&&" | "||" | "??"))
}
