//! Discovers functions and classes, names them by their bindings, and folds unnamed callbacks.

use super::counts::cognitive;
use crate::metrics::{ClassMetrics, FnMetrics};
use tree_sitter::Node;

/// Expressions a binding sees through to reach the function it names.
const WRAPPERS: [&str; 4] = [
    "parenthesized_expression",
    "as_expression",
    "satisfies_expression",
    "non_null_expression",
];
/// Calls whose callback implements the binding, as in `const Button = forwardRef(...)`.
const WRAPPER_CALLS: [&str; 6] = [
    "forwardRef",
    "memo",
    "useCallback",
    "useMemo",
    "lazy",
    "observer",
];
/// Nodes between a declaration and the comment or statement that introduces it.
const DECLARATION_PARENTS: [&str; 6] = [
    "variable_declarator",
    "lexical_declaration",
    "variable_declaration",
    "export_statement",
    "arguments",
    "call_expression",
];

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

fn is_class(kind: &str) -> bool {
    matches!(
        kind,
        "class_declaration" | "abstract_class_declaration" | "class"
    )
}

/// Collects named functions as units below `parent`, folding unnamed callbacks into it.
pub(super) fn collect_declarations(
    node: Node<'_>,
    source: &[u8],
    parent: Option<usize>,
    functions: &mut Vec<FnMetrics>,
    classes: &mut Vec<ClassMetrics>,
) {
    let mut parent = parent;
    if is_class(node.kind()) {
        classes.push(ClassMetrics {
            name: class_name(node, source),
            line_start: node.start_position().row + 1,
            line_end: node.end_position().row + 1,
        });
    } else if let Some(name) = is_function(node.kind())
        .then(|| unit_name(node, source, parent.is_some()))
        .flatten()
    {
        let unit = measure(node, name, source);
        if let Some(index) = parent {
            functions[index].nested_lines += unit.line_end + 1 - unit.line_start;
        }
        functions.push(unit);
        parent = Some(functions.len() - 1);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_declarations(child, source, parent, functions, classes);
    }
}

/// Recognizes nested functions measured as their own units rather than folded.
pub(super) fn is_unit(node: Node<'_>, source: &[u8]) -> bool {
    unit_name(node, source, true).is_some()
}

/// Names a function by itself, its binding, or its test block; None folds a nested callback.
fn unit_name(node: Node<'_>, source: &[u8], nested: bool) -> Option<String> {
    let own = node
        .child_by_field_name("name")
        .and_then(|name| text(name, source));
    own.map(|name| qualify(node, name, source))
        .or_else(|| bound_name(node, source, nested))
        .or_else(|| test_block(node, source))
        .or_else(|| (!nested).then(|| fallback(node, source)))
}

/// Follows wrappers (and, at the top level, any call) up to the binding that names `node`.
fn bound_name(node: Node<'_>, source: &[u8], nested: bool) -> Option<String> {
    let mut current = node.parent()?;
    loop {
        let call = current.parent().filter(|_| current.kind() == "arguments");
        if WRAPPERS.contains(&current.kind()) {
            current = current.parent()?;
        } else if let Some(call) = call.filter(|call| !nested || is_wrapper_call(*call, source)) {
            current = call.parent()?;
        } else {
            break;
        }
    }
    let field = match current.kind() {
        "variable_declarator" | "public_field_definition" => "name",
        "pair" => "key",
        "assignment_expression" => "left",
        "export_statement" => return Some("default".to_owned()),
        _ => return None,
    };
    let name = text(current.child_by_field_name(field)?, source)?;
    let name = name.rsplit('.').next()?.trim_matches(['"', '\'']);
    let simple = !name.is_empty() && !name.contains(['{', '[', ' ']);
    simple.then(|| qualify(current, name.to_owned(), source))
}

/// Names a callback passed after a string, such as `it("undoes")` or `app.get("/users")`.
fn test_block(node: Node<'_>, source: &[u8]) -> Option<String> {
    let arguments = node
        .parent()
        .filter(|parent| parent.kind() == "arguments")?;
    let label = (arguments.named_child(0)).filter(|first| {
        first.id() != node.id() && matches!(first.kind(), "string" | "template_string")
    })?;
    let callee = text(arguments.parent()?.child_by_field_name("function")?, source)?;
    Some(format!("{callee}({})", text(label, source)?))
}

/// Names a top-level callback after the call that receives it.
fn fallback(node: Node<'_>, source: &[u8]) -> String {
    let call = node.parent().filter(|parent| parent.kind() == "arguments");
    let callee = call.and_then(|call| call.parent()?.child_by_field_name("function"));
    callee
        .and_then(|callee| text(callee, source))
        .map_or_else(|| "<anonymous>".to_owned(), |callee| format!("{callee}(…)"))
}

/// Prefixes members of a class body with the class name, as in `View.render`.
fn qualify(member: Node<'_>, name: String, source: &[u8]) -> String {
    let class = member
        .parent()
        .filter(|parent| parent.kind() == "class_body")
        .and_then(|body| body.parent());
    match class {
        Some(class) => format!("{}.{name}", class_name(class, source)),
        None => name,
    }
}

fn class_name(class: Node<'_>, source: &[u8]) -> String {
    let own = class
        .child_by_field_name("name")
        .and_then(|name| text(name, source));
    own.or_else(|| bound_name(class, source, true))
        .unwrap_or_else(|| "<anonymous>".to_owned())
}

fn text(node: Node<'_>, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(str::to_owned)
}

/// Measures a unit's body, retaining the extent of the statement that declares it.
fn measure(node: Node<'_>, name: String, source: &[u8]) -> FnMetrics {
    let declaration = declaration(node);
    FnMetrics {
        name,
        line_start: declaration.start_position().row + 1,
        line_end: node.end_position().row + 1,
        cognitive: node
            .child_by_field_name("body")
            .map_or(0, |body| cognitive(body, 0, source)),
        has_docstring: declaration.prev_named_sibling().is_some_and(|sibling| {
            sibling.kind() == "comment"
                && text(sibling, source).is_some_and(|doc| doc.starts_with("/**"))
        }),
        nested_lines: 0,
    }
}

/// Follows wrappers and declarations so a unit starts where its JSDoc attaches.
fn declaration(mut node: Node<'_>) -> Node<'_> {
    while let Some(parent) = node.parent() {
        let kind = parent.kind();
        if !(WRAPPERS.contains(&kind) || DECLARATION_PARENTS.contains(&kind)) {
            break;
        }
        node = parent;
    }
    node
}

/// Recognizes calls such as `memo(...)` or `React.forwardRef(...)` that wrap a component.
fn is_wrapper_call(call: Node<'_>, source: &[u8]) -> bool {
    let callee = call
        .child_by_field_name("function")
        .and_then(|callee| text(callee, source));
    callee.is_some_and(|callee| WRAPPER_CALLS.contains(&callee.rsplit('.').next().unwrap_or("")))
}
