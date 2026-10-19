//! Reads what the syntax tree says about declarations and files: docs, headers, exports
//! and imports.

use crate::{lang::Spec, metrics::Import};
use tree_sitter::Node;

/// Recognizes a bare string statement, which documents rather than computes.
pub fn is_docstring(spec: &Spec, node: Node<'_>) -> bool {
    spec.docs.is_empty()
        && node.kind() == "expression_statement"
        && node.named_child_count() == 1
        && node
            .named_child(0)
            .is_some_and(|child| matches!(child.kind(), "string" | "concatenated_string"))
}

/// The outermost node starting on the declaration's row short of the object, class body or
/// block holding it, which a doc comment sits above, such as its export or statement.
pub fn anchor(declaration: Node<'_>) -> Node<'_> {
    let row = declaration.start_position().row;
    let containers = [
        "object",
        "class_body",
        "declaration_list",
        "statement_block",
        "block",
    ];
    let same_row = |node: &Node<'_>| {
        let container = containers.contains(&node.kind()) || node.parent().is_none();
        node.start_position().row == row && !container
    };
    let outer = std::iter::successors(Some(declaration), |node| node.parent().filter(same_row));
    outer.last().unwrap_or(declaration)
}

/// The declaration from its anchor up to its body or first comment, on one line.
pub fn signature(source: &str, node: Node<'_>, anchor: Node<'_>) -> String {
    let body = node.child_by_field_name("body");
    let mut cursor = node.walk();
    let mut children = node.children(&mut cursor);
    let end = children.find(|child| child.is_extra() || Some(*child) == body);
    let header = &source[anchor.start_byte()..end.map_or(node.end_byte(), |e| e.start_byte())];
    header.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether a declaration belongs to its module's interface: `export`, a top-level name
/// that a local `export { }` or `export default` names, `pub`, or a public top-level
/// Python name.
pub fn exported(declaration: Node<'_>, name: &str, exports: &[String]) -> bool {
    let mut parent = declaration.parent();
    if parent.is_some_and(|p| p.kind() == "decorated_definition") {
        parent = parent.and_then(|p| p.parent());
    }
    declaration.kind() == "export_statement"
        || declaration
            .child(0)
            .is_some_and(|c| c.kind() == "visibility_modifier")
        || parent.is_some_and(|p| p.kind() == "module") && !name.starts_with('_')
        || parent.is_some_and(|p| p.kind() == "program") && exports.iter().any(|e| e == name)
}

/// Names that local `export { a, b as c }` and `export default a` statements expose.
pub fn export_names(root: Node<'_>, source: &str) -> Vec<String> {
    let mut cursor = root.walk();
    let local = (root.named_children(&mut cursor))
        .filter(|s| s.kind() == "export_statement" && s.child_by_field_name("source").is_none());
    let names = local.flat_map(|statement| {
        let clause = statement
            .named_child(0)
            .filter(|c| c.kind() == "export_clause");
        let count = clause.map_or(0, |clause| clause.named_child_count());
        let specifiers =
            (0..count).filter_map(move |i| clause?.named_child(i)?.child_by_field_name("name"));
        let default = statement.child_by_field_name("value");
        specifiers.chain(default.filter(|value| value.kind() == "identifier"))
    });
    names
        .map(|name| source[name.byte_range()].to_owned())
        .collect()
}

/// Summarizes the doc comments above a declaration's anchor (past attributes and plain
/// comments; a JSDoc block ends the search), or the docstring opening its body.
pub fn doc(spec: &Spec, source: &str, node: Node<'_>, anchor: Node<'_>) -> Option<String> {
    if spec.docs.is_empty() {
        return docstring(spec, source, node.child_by_field_name("body")?);
    }
    let mut text = String::new();
    let mut sibling = anchor.prev_named_sibling();
    while let Some(node) = sibling {
        let doc = is_doc(spec, &source[node.byte_range()]);
        if doc {
            text.insert_str(0, &doc_text(source, node));
        } else if !text.is_empty() || !spec.attributes.contains(&node.kind()) {
            break;
        }
        let block = node.kind() == "comment" && doc;
        sibling = node.prev_named_sibling().filter(|_| !block);
    }
    summary(&text)
}

/// Recognizes a doc comment or attribute (`///` but not `////`, or `#[doc = "…"]`).
fn is_doc(spec: &Spec, text: &str) -> bool {
    let doc = |prefix: &&str| {
        let rest = text.strip_prefix(*prefix);
        rest.is_some_and(|rest| !rest.starts_with(&prefix[prefix.len() - 1..]))
    };
    spec.docs.iter().any(doc)
}

/// Summarizes the docstring that opens a Python body or module.
fn docstring(spec: &Spec, source: &str, body: Node<'_>) -> Option<String> {
    let mut cursor = body.walk();
    let first = body.named_children(&mut cursor).find(|s| !s.is_extra())?;
    let doc = is_docstring(spec, first) && !formatted(source, first);
    doc.then(|| summary(&doc_text(source, first)))?
}

/// Whether a string statement holds an f-string or bytes literal, which Python never
/// takes as a docstring.
fn formatted(source: &str, node: Node<'_>) -> bool {
    node.kind() == "string_start" && source[node.byte_range()].contains(['f', 'F', 'b', 'B'])
        || (node.named_children(&mut node.walk())).any(|child| formatted(source, child))
}

/// Describes a file by its module docstring, or by the doc comments before its first
/// code, skipping licenses, shebangs, directives and inner attributes.
pub fn module_doc(spec: &Spec, source: &str, root: Node<'_>) -> Option<String> {
    if spec.docs.is_empty() {
        return docstring(spec, source, root);
    }
    let mut cursor = root.walk();
    let mut text = String::new();
    for child in root.named_children(&mut cursor) {
        let code = &source[child.byte_range()];
        if ["//!", "/*!", "/**"]
            .iter()
            .any(|doc| code.starts_with(doc))
        {
            text += &doc_text(source, child);
        } else if !child.is_extra() && !preamble(child) {
            break;
        }
    }
    summary(&text)
}

/// The text of a doc node: a Rust doc comment's `doc` part and string contents, one
/// line each, or a whole JSDoc block as its own paragraph.
fn doc_text(source: &str, node: Node<'_>) -> String {
    let text = source[node.byte_range()].trim_end();
    match node.kind() {
        "comment" if summary(text).is_none() => String::new(),
        "comment" => format!("{text}\n\n"),
        "doc_comment" | "string_content" => format!("{text}\n"),
        _ => {
            let mut cursor = node.walk();
            let parts = node.named_children(&mut cursor);
            parts.map(|part| doc_text(source, part)).collect()
        }
    }
}

/// Recognizes what may precede a module doc: shebangs, inner attributes and directives
/// such as `'use client'`.
fn preamble(node: Node<'_>) -> bool {
    let first = node.named_child(0).map(|child| child.kind());
    matches!(
        node.kind(),
        "hash_bang_line" | "shebang" | "inner_attribute_item"
    ) || node.kind() == "expression_statement" && first == Some("string")
}

/// Joins the first paragraph up to its tags (`@param`), past a reStructuredText title
/// and comment markers; None when that paragraph is a license or only tags.
fn summary(text: &str) -> Option<String> {
    let lines: Vec<&str> = text
        .lines()
        .map(|line| {
            let line = line.trim();
            let marker = ["/**/", "/**", "/*!", "* "]
                .iter()
                .find_map(|m| line.strip_prefix(m));
            let line = marker.unwrap_or(if line == "*" { "" } else { line });
            line.trim_end_matches("*/").trim()
        })
        .collect();
    let mut paragraphs = lines.split(|line| line.is_empty());
    let paragraph = paragraphs.find(|p| !p.is_empty() && !is_heading(p))?;
    let prose = paragraph.iter().take_while(|line| !line.starts_with('@'));
    let prose: Vec<&str> = prose.copied().collect();
    (!license(paragraph) && !prose.is_empty()).then(|| prose.join(" "))
}

/// Recognizes a license paragraph, which never describes code.
fn license(paragraph: &[&str]) -> bool {
    let lower = paragraph.join(" ").to_ascii_lowercase();
    let markers = ["spdx-license", "@license", "licensed under"];
    lower.starts_with("copyright") || markers.iter().any(|marker| lower.contains(marker))
}

/// Recognizes a reStructuredText title: one line of text above an underline.
fn is_heading(paragraph: &[&str]) -> bool {
    matches!(paragraph, [_, underline] if underline.chars().all(|c| "=-~^".contains(c)))
}

/// Adds the imports a node makes: JS `import`, `export … from`, `import()` and `require()`
/// (one per specifier), Python `import` and `from … import` (one per name) and Rust `use`
/// (one per expanded path).
pub fn imports(node: Node<'_>, source: &str, list: &mut Vec<Import>) {
    let line = node.start_position().row + 1;
    let specifier = match node.kind() {
        "import_statement" | "export_statement" | "import_require_clause" => {
            node.child_by_field_name("source")
        }
        "call_expression" => called(node, source),
        _ => None,
    };
    if let Some(string) = specifier.filter(|s| s.kind() == "string" && !s.has_error()) {
        let quoted = &source[string.byte_range()];
        let source = quoted[1..quoted.len() - 1].to_owned();
        list.push(Import {
            line,
            source,
            name: None,
        });
    }
    match node.kind() {
        "import_statement" | "import_from_statement" => python_imports(node, source, list),
        "use_declaration" => {
            let tree = node.child_by_field_name("argument");
            tree.into_iter()
                .for_each(|tree| use_tree(tree, "", line, source, list));
        }
        _ => {}
    }
}

/// The string named by `import("x")` or `require("x")`, past any comment argument.
fn called<'t>(call: Node<'t>, source: &str) -> Option<Node<'t>> {
    let callee = &source[call.child(0)?.byte_range()];
    let arguments = call.child_by_field_name("arguments")?;
    let mut cursor = arguments.walk();
    let first = arguments
        .named_children(&mut cursor)
        .find(|a| a.kind() != "comment");
    first.filter(|_| matches!(callee, "import" | "require"))
}

/// Python `import a.b as c` and `from ..a import b, *`: one import per name, keeping the
/// module's leading dots.
fn python_imports(node: Node<'_>, source: &str, list: &mut Vec<Import>) {
    let text = |node: Node<'_>| source[node.byte_range()].to_owned();
    let line = node.start_position().row + 1;
    let module = node.child_by_field_name("module_name").map(text);
    let wildcard = text(node).ends_with('*').then(|| "*".to_owned());
    let mut cursor = node.walk();
    let names = node.children_by_field_name("name", &mut cursor);
    for name in names
        .map(|n| text(n.child_by_field_name("name").unwrap_or(n)))
        .chain(wildcard)
    {
        let (source, name) = match &module {
            Some(module) => (module.clone(), Some(name)),
            None => (name, None),
        };
        list.push(Import { line, source, name });
    }
}

/// Expands a Rust use tree into one import per path; `self` in a list names its prefix.
fn use_tree(node: Node<'_>, prefix: &str, line: usize, source: &str, list: &mut Vec<Import>) {
    let own = match node.kind() {
        "scoped_use_list" | "use_as_clause" => node.child_by_field_name("path"),
        _ => Some(node),
    };
    let own = own.map_or("", |own| &source[own.byte_range()]);
    let path = match (prefix, own) {
        ("", own) | (own, "self") => own.to_owned(),
        _ => format!("{prefix}::{own}"),
    };
    match node.kind() {
        "use_list" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                use_tree(child, prefix, line, source, list);
            }
        }
        "scoped_use_list" => {
            let tree = node.child_by_field_name("list");
            tree.into_iter()
                .for_each(|tree| use_tree(tree, &path, line, source, list));
        }
        kind if kind.ends_with("comment") => {}
        _ => list.push(Import {
            line,
            source: path,
            name: None,
        }),
    }
}
