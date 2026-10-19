//! Classifies TypeScript syntax for declaration and complexity walkers.

use tree_sitter::Node;

/// Recognizes declarations represented as classes in outlines.
pub(super) fn is_class(kind: &str) -> bool {
    matches!(kind, "class_declaration" | "abstract_class_declaration")
}
/// Recognizes executable function forms, including methods and arrows.
pub(super) fn is_function(kind: &str) -> bool {
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

/// Identifies branching control-flow syntax.
pub(super) fn is_decision(kind: &str) -> bool {
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

/// Identifies constructs that increase cognitive nesting depth.
pub(super) fn is_nesting(kind: &str) -> bool {
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

/// Identifies syntax operations counted toward Halstead volume.
pub(super) fn is_operator(kind: &str) -> bool {
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

/// Identifies value and name nodes counted as operands.
pub(super) fn is_operand(node: Node<'_>) -> bool {
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
