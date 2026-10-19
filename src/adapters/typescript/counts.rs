//! Computes SonarSource cognitive complexity for function bodies, folding unnamed callbacks.

use super::declarations::{is_function, is_unit};
use tree_sitter::Node;

/// Structures that add 1 plus their nesting, with the fields nested one level deeper.
const STRUCTURES: [(&str, &[&str]); 7] = [
    ("switch_statement", &["body"]),
    ("for_statement", &["body"]),
    ("for_in_statement", &["body"]),
    ("while_statement", &["body"]),
    ("do_statement", &["body"]),
    ("catch_clause", &["body"]),
    ("ternary_expression", &["consequence", "alternative"]),
];

/// Sums a body's cognitive complexity; named nested functions are measured separately.
pub(super) fn cognitive(node: Node<'_>, nesting: u32, source: &[u8]) -> u32 {
    let structure = STRUCTURES.iter().find(|(kind, _)| *kind == node.kind());
    let rule = structure.map(|(_, nested)| (1 + nesting, *nested));
    let Some((mut total, nested)) = rule.or_else(|| increment(node, nesting, source)) else {
        return 0;
    };
    let mut cursor = node.walk();
    let mut more = cursor.goto_first_child();
    while more {
        let deeper = nested.contains(&cursor.field_name().unwrap_or(""));
        if cursor.node().is_named() {
            total += cognitive(cursor.node(), nesting + u32::from(deeper), source);
        }
        more = cursor.goto_next_sibling();
    }
    total
}

/// Rates functions, `if`/`else`, labelled jumps and logical runs; None skips a nested unit.
fn increment(
    node: Node<'_>,
    nesting: u32,
    source: &[u8],
) -> Option<(u32, &'static [&'static str])> {
    let in_else = |node: Node<'_>| node.parent().is_some_and(|p| p.kind() == "else_clause");
    let else_if = |node: Node<'_>| {
        node.named_child(0)
            .is_some_and(|c| c.kind() == "if_statement")
    };
    Some(match node.kind() {
        kind if is_function(kind) && is_unit(node, source) => return None,
        kind if is_function(kind) => (0, &["body"]),
        "if_statement" if in_else(node) => (1, &["consequence"]),
        "if_statement" => (1 + nesting, &["consequence"]),
        "else_clause" if else_if(node) => (0, &[]),
        "else_clause" => (1, &[""]),
        "break_statement" | "continue_statement" => {
            (u32::from(node.child_by_field_name("label").is_some()), &[])
        }
        _ => (logical_sequences(node), &[]),
    })
}

/// Counts runs of like `&&`/`||` operators in the expression rooted here (`a && b || c` is 2).
fn logical_sequences(node: Node<'_>) -> u32 {
    if logical_operator(node).is_none() {
        return 0;
    }
    let mut parent = node.parent();
    while let Some(wrapper) = parent.filter(|p| p.kind() == "parenthesized_expression") {
        parent = wrapper.parent();
    }
    if parent.and_then(logical_operator).is_some() {
        return 0;
    }
    let mut operators = Vec::new();
    flatten(node, &mut operators);
    1 + operators
        .windows(2)
        .filter(|pair| pair[0] != pair[1])
        .count() as u32
}

/// Lists the logical operators of a parenthesized `&&`/`||` expression in source order.
fn flatten(node: Node<'_>, operators: &mut Vec<&'static str>) {
    if logical_operator(node).is_none() && node.kind() != "parenthesized_expression" {
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "&&" | "||" => operators.push(child.kind()),
            _ => flatten(child, operators),
        }
    }
}

fn logical_operator(node: Node<'_>) -> Option<&'static str> {
    let operator = node.child_by_field_name("operator")?.kind();
    (node.kind() == "binary_expression" && matches!(operator, "&&" | "||")).then_some(operator)
}
