//! Reads documentation and imports: doc nodes and import statements for the walker, and
//! leading comments and docstrings by line.

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

/// Finds a doc comment or attribute above the declaration, or a docstring opening the body.
pub fn documented(spec: &Spec, source: &[u8], node: Node<'_>, declaration: Node<'_>) -> bool {
    if spec.docs.is_empty() {
        let mut cursor = node.walk();
        let body = node.child_by_field_name("body");
        let first = body.and_then(|body| body.named_children(&mut cursor).find(|s| !s.is_extra()));
        return first.is_some_and(|statement| is_docstring(spec, statement));
    }
    let mut sibling = declaration.prev_named_sibling();
    while let Some(node) = sibling {
        let text = node.utf8_text(source).unwrap_or_default();
        let doc = |prefix: &&str| {
            let rest = text.strip_prefix(*prefix);
            rest.is_some_and(|rest| !rest.starts_with(&prefix[prefix.len() - 1..]))
        };
        if spec.docs.iter().any(doc) && !text.starts_with("#[doc(hidden") {
            return true;
        }
        if !spec.attributes.contains(&node.kind()) {
            return false;
        }
        sibling = node.prev_named_sibling();
    }
    false
}

/// Describe a file by its leading module documentation, in any supported language.
pub fn module_doc(source: &str) -> Option<String> {
    let source = source.trim_start_matches('\u{feff}');
    leading_docs(&source.lines().collect::<Vec<_>>())
}

/// Extract documentation for the declaration that starts on `line`.
pub fn docstring(language: &str, source: &str, line: usize) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    if language == "python" {
        return python_docstring(&lines, line);
    }
    preceding_docs(&lines, line, language)
}

/// Reads the docstring that opens the body of the `def` or `class` starting on `line`.
fn python_docstring(lines: &[&str], line: usize) -> Option<String> {
    let mut depth = 0;
    let end = (line.saturating_sub(1)..lines.len()).find(|&index| {
        depth += lines[index].matches(['(', '[']).count() as i64;
        depth -= lines[index].matches([')', ']']).count() as i64;
        depth <= 0
    })?;
    let header = lines[end].split(" #").next().unwrap_or_default().trim_end();
    let body = end
        + 1
        + lines[end + 1..]
            .iter()
            .position(|text| !text.trim().is_empty())?;
    (header.ends_with(':') && opens_block(lines[body])).then(|| leading_docs(&lines[body..]))?
}

/// Collects the comment block immediately above a declaration.
fn preceding_docs(lines: &[&str], end: usize, language: &str) -> Option<String> {
    let mut docs = Vec::new();
    let mut block = false;
    for line in lines[..end.saturating_sub(1).min(lines.len())].iter().rev() {
        let line = line.trim();
        if line == "*/" {
            block = true;
            continue;
        }
        let text = if language == "rust" {
            line.strip_prefix("///")
        } else if block {
            line.strip_prefix("/**")
                .or_else(|| line.strip_prefix('*'))
                .map(str::trim)
        } else {
            line.strip_prefix("/**").or_else(|| line.strip_prefix("//"))
        };
        let Some(text) = text else {
            break;
        };
        docs.push(text.trim().trim_end_matches("*/").trim().to_owned());
        if line.starts_with("/**") {
            break;
        }
    }
    (!docs.is_empty()).then(|| {
        docs.reverse();
        docs.join(" ")
    })
}

/// Block comments and docstrings: opener, terminator, and whether the block documents code.
const BLOCKS: [(&str, &str, bool); 7] = [
    ("/*!", "*/", true),
    ("/**", "*/", true),
    ("\"\"\"", "\"\"\"", true),
    ("'''", "'''", true),
    ("r\"\"\"", "\"\"\"", true),
    ("r'''", "'''", true),
    ("/*", "*/", false),
];

/// Recognizes the start of a documentation block comment or docstring.
fn opens_block(line: &str) -> bool {
    BLOCKS
        .iter()
        .any(|(open, _, documents)| *documents && line.trim().starts_with(open))
}

/// Summarizes the first leading documentation block, skipping license headers and pragmas.
fn leading_docs(lines: &[&str]) -> Option<String> {
    let mut index = 0;
    while let Some(line) = lines.get(index).map(|line| line.trim()) {
        index += 1;
        if is_preamble(line) && !line.starts_with("//!") {
            continue;
        }
        if let Some(summary) = summary(&next_block(lines, &mut index, line)?) {
            return Some(summary);
        }
    }
    None
}

/// Reads the comment block or docstring starting on `line`; None when the line is code.
fn next_block<'a>(lines: &[&'a str], index: &mut usize, line: &'a str) -> Option<Vec<&'a str>> {
    if let Some(first) = line.strip_prefix("//!") {
        return Some(line_docs(lines, index, first));
    }
    let (open, close, documents) = BLOCKS.iter().find(|(open, ..)| line.starts_with(open))?;
    let text = block_docs(lines, index, &line[open.len()..], close);
    Some(text.filter(|_| *documents).unwrap_or_default())
}

/// Recognizes lines that may precede module documentation: blanks, comments,
/// shebangs, attributes and directives such as `'use client'`.
fn is_preamble(line: &str) -> bool {
    line.is_empty()
        || line.starts_with("//")
        || line.starts_with('#')
        || line.starts_with("'use ")
        || line.starts_with("\"use ")
        || line.starts_with("/**/")
}

/// Collects a run of `//!` lines that starts with `first`.
fn line_docs<'a>(lines: &[&'a str], index: &mut usize, first: &'a str) -> Vec<&'a str> {
    let mut text = vec![first];
    while let Some(next) = lines.get(*index).and_then(|l| l.trim().strip_prefix("//!")) {
        text.push(next);
        *index += 1;
    }
    text
}

/// Collects a block comment or docstring body up to its terminator, if it has one.
fn block_docs<'a>(
    lines: &[&'a str],
    index: &mut usize,
    first: &'a str,
    close: &str,
) -> Option<Vec<&'a str>> {
    let mut text = Vec::new();
    let mut rest = first;
    while !rest.contains(close) {
        text.push(rest);
        rest = lines.get(*index)?.trim();
        *index += 1;
    }
    text.push(rest.split(close).next().unwrap_or(rest));
    Some(text)
}

/// Joins the first paragraph up to its tags (`@param`), unless it is a license or a pragma.
fn summary(text: &[&str]) -> Option<String> {
    let lines: Vec<&str> = text
        .iter()
        .map(|line| line.trim().trim_start_matches('*').trim())
        .collect();
    let paragraph = lines
        .split(|line| line.is_empty())
        .find(|paragraph| !paragraph.is_empty() && !is_heading(paragraph))?;
    let lower = paragraph.join(" ").to_ascii_lowercase();
    let license = ["copyright", "spdx-license", "@license", "licensed under"]
        .iter()
        .any(|marker| lower.contains(marker));
    let prose: Vec<&str> = (paragraph.iter())
        .take_while(|line| !line.starts_with('@'))
        .copied()
        .collect();
    (!license && !prose.is_empty()).then(|| prose.join(" "))
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
    if let Some(string) = specifier.filter(|string| string.kind() == "string") {
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
