//! Parses source with tree-sitter and measures each function unit from its language's table.

use crate::{
    lang::{Language, Spec},
    metrics::{ClassMetrics, FileMetrics, FnMetrics},
};
use tree_sitter::{Node, Parser};

/// Measures one source file, warning when only part of it could be parsed.
pub fn analyse(path: &str, source: &str) -> Result<FileMetrics, String> {
    let language = Language::from_path(path).ok_or_else(|| unsupported(path))?;
    let metrics = match language {
        Language::Rust => crate::visit::analyse(path, source)?,
        Language::Python => crate::python::analyse_text(language.name(), path, source),
        _ => parse(language, path, source)?,
    };
    if metrics.parse_errors > 0 {
        eprintln!(
            "warning: {path}: syntax errors: {}; measured the recoverable code",
            metrics.parse_errors
        );
    }
    Ok(metrics)
}

fn unsupported(path: &str) -> String {
    match std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some(extension) => format!("unsupported language: .{extension}"),
        None => "cannot determine language: file has no extension".to_owned(),
    }
}

/// Parses with the language's grammar, recovering from syntax errors, and measures each unit.
fn parse(language: Language, path: &str, source: &str) -> Result<FileMetrics, String> {
    let (grammar, spec) = language.grammar().ok_or("no grammar for this language")?;
    let mut parser = Parser::new();
    parser
        .set_language(&grammar)
        .map_err(|error| format!("could not configure the parser: {error}"))?;
    let tree = parser
        .parse(source, None)
        .ok_or("the parser returned no tree")?;
    let mut walker = Walker {
        spec,
        source: source.as_bytes(),
        fns: Vec::new(),
        classes: Vec::new(),
    };
    walker.collect(tree.root_node(), None);
    Ok(FileMetrics {
        language: language.name(),
        path: path.to_owned(),
        loc: source.lines().count(),
        has_docstring: crate::extract::module_doc(source).is_some(),
        parse_errors: syntax_errors(tree.root_node()),
        fns: walker.fns,
        classes: walker.classes,
    })
}

/// Counts the ERROR and MISSING nodes left by tree-sitter's error recovery.
fn syntax_errors(node: Node<'_>) -> usize {
    if node.is_error() || node.is_missing() {
        return 1;
    }
    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    children
        .filter(|child| child.has_error())
        .map(syntax_errors)
        .sum()
}

/// Walks one syntax tree using its language's table, collecting units and classes.
struct Walker<'a> {
    spec: &'static Spec,
    source: &'a [u8],
    fns: Vec<FnMetrics>,
    classes: Vec<ClassMetrics>,
}

impl Walker<'_> {
    /// Collects named functions as units below `parent`, folding unnamed callbacks into it.
    fn collect(&mut self, node: Node<'_>, parent: Option<usize>) {
        let mut parent = parent;
        if self.spec.classes.contains(&node.kind()) {
            self.classes.push(ClassMetrics {
                name: self.class_name(node),
                line_start: node.start_position().row + 1,
                line_end: node.end_position().row + 1,
            });
        } else if let Some(name) = self
            .is_function(node)
            .then(|| self.unit_name(node, parent.is_some()))
            .flatten()
        {
            let unit = self.unit(node, name);
            if let Some(index) = parent {
                self.fns[index].nested_lines += unit.line_end + 1 - unit.line_start;
            }
            self.fns.push(unit);
            parent = Some(self.fns.len() - 1);
        }
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.collect(child, parent);
        }
    }

    fn is_function(&self, node: Node<'_>) -> bool {
        self.spec.functions.contains(&node.kind())
    }

    /// Names a function by itself, its binding, or its test block; None folds a nested callback.
    fn unit_name(&self, node: Node<'_>, nested: bool) -> Option<String> {
        let own = node
            .child_by_field_name("name")
            .and_then(|name| self.text(name));
        own.map(|name| self.qualify(node, name))
            .or_else(|| self.bound_name(node, nested))
            .or_else(|| self.test_block(node))
            .or_else(|| (!nested).then(|| self.fallback(node)))
    }

    /// Follows wrappers (and, at the top level, any call) up to the binding that names `node`.
    fn bound_name(&self, node: Node<'_>, nested: bool) -> Option<String> {
        let mut current = node.parent()?;
        loop {
            let call = current.parent().filter(|_| current.kind() == "arguments");
            if self.spec.wrappers.contains(&current.kind()) {
                current = current.parent()?;
            } else if let Some(call) = call.filter(|call| !nested || self.is_wrapper_call(*call)) {
                current = call.parent()?;
            } else {
                break;
            }
        }
        if current.kind() == "export_statement" {
            return Some("default".to_owned());
        }
        let (_, field) = self
            .spec
            .bindings
            .iter()
            .find(|(kind, _)| *kind == current.kind())?;
        let name = self.text(current.child_by_field_name(field)?)?;
        let name = name.rsplit('.').next()?.trim_matches(['"', '\'']);
        let simple = !name.is_empty() && !name.contains(['{', '[', ' ']);
        simple.then(|| self.qualify(current, name.to_owned()))
    }

    /// Names a callback passed after a string, such as `it("undoes")` or `app.get("/users")`.
    fn test_block(&self, node: Node<'_>) -> Option<String> {
        let arguments = node
            .parent()
            .filter(|parent| parent.kind() == "arguments")?;
        let label = (arguments.named_child(0)).filter(|first| {
            first.id() != node.id() && matches!(first.kind(), "string" | "template_string")
        })?;
        let callee = self.text(arguments.parent()?.child_by_field_name("function")?)?;
        Some(format!("{callee}({})", self.text(label)?))
    }

    /// Names a top-level callback after the call that receives it.
    fn fallback(&self, node: Node<'_>) -> String {
        let call = node.parent().filter(|parent| parent.kind() == "arguments");
        let callee = call.and_then(|call| call.parent()?.child_by_field_name("function"));
        callee
            .and_then(|callee| self.text(callee))
            .map_or_else(|| "<anonymous>".to_owned(), |callee| format!("{callee}(…)"))
    }

    /// Prefixes members of a class body with the class name, as in `View.render`.
    fn qualify(&self, member: Node<'_>, name: String) -> String {
        let class = member
            .parent()
            .filter(|parent| parent.kind() == "class_body")
            .and_then(|body| body.parent());
        match class {
            Some(class) => format!("{}.{name}", self.class_name(class)),
            None => name,
        }
    }

    fn class_name(&self, class: Node<'_>) -> String {
        let own = class
            .child_by_field_name("name")
            .and_then(|name| self.text(name));
        own.or_else(|| self.bound_name(class, true))
            .unwrap_or_else(|| "<anonymous>".to_owned())
    }

    fn text(&self, node: Node<'_>) -> Option<String> {
        node.utf8_text(self.source).ok().map(str::to_owned)
    }

    /// Measures a unit's body, retaining the extent of the statement that declares it.
    fn unit(&self, node: Node<'_>, name: String) -> FnMetrics {
        let declaration = self.declaration(node);
        let body = node.child_by_field_name("body");
        FnMetrics {
            name,
            line_start: declaration.start_position().row + 1,
            line_end: node.end_position().row + 1,
            cognitive: body.map_or(0, |body| self.cognitive(body, 0)),
            has_docstring: declaration.prev_named_sibling().is_some_and(|sibling| {
                sibling.kind() == "comment"
                    && self.text(sibling).is_some_and(|doc| doc.starts_with("/**"))
            }),
            nested_lines: 0,
        }
    }

    /// Follows wrappers and declarations so a unit starts where its JSDoc attaches.
    fn declaration<'t>(&self, mut node: Node<'t>) -> Node<'t> {
        while let Some(parent) = node.parent() {
            let kind = parent.kind();
            if !(self.spec.wrappers.contains(&kind) || self.spec.declarations.contains(&kind)) {
                break;
            }
            node = parent;
        }
        node
    }

    /// Recognizes calls such as `memo(...)` or `React.forwardRef(...)` that wrap a component.
    fn is_wrapper_call(&self, call: Node<'_>) -> bool {
        let callee = call
            .child_by_field_name("function")
            .and_then(|f| self.text(f));
        let name = callee
            .as_deref()
            .map(|callee| callee.rsplit('.').next().unwrap_or(""));
        name.is_some_and(|name| self.spec.wrapper_calls.contains(&name))
    }

    /// Sums a body's cognitive complexity; named nested functions are measured separately.
    fn cognitive(&self, node: Node<'_>, nesting: u32) -> u32 {
        let structure = self
            .spec
            .structures
            .iter()
            .find(|(kind, _)| *kind == node.kind());
        let rule = structure.map(|(_, nested)| (1 + nesting, *nested));
        let Some((mut total, nested)) = rule.or_else(|| self.increment(node, nesting)) else {
            return 0;
        };
        let mut cursor = node.walk();
        let mut more = cursor.goto_first_child();
        while more {
            let deeper = nested.contains(&cursor.field_name().unwrap_or(""));
            if cursor.node().is_named() {
                total += self.cognitive(cursor.node(), nesting + u32::from(deeper));
            }
            more = cursor.goto_next_sibling();
        }
        total
    }

    /// Rates functions, `if`/`else`, labelled jumps and logical runs; None skips a nested unit.
    fn increment(&self, node: Node<'_>, nesting: u32) -> Option<(u32, &'static [&'static str])> {
        let [if_kind, else_kind] = self.spec.conditional;
        let kind = node.kind();
        let parent_is = |kind| node.parent().is_some_and(|p| p.kind() == kind);
        let first_is = |kind| node.named_child(0).is_some_and(|c| c.kind() == kind);
        Some(match kind {
            _ if self.is_function(node) && self.unit_name(node, true).is_some() => return None,
            _ if self.is_function(node) => (0, &["body"]),
            _ if kind == if_kind && parent_is(else_kind) => (1, &["consequence"]),
            _ if kind == if_kind => (1 + nesting, &["consequence"]),
            _ if kind == else_kind && first_is(if_kind) => (0, &[]),
            _ if kind == else_kind => (1, &[""]),
            _ if self.spec.jumps.contains(&kind) => {
                (u32::from(node.child_by_field_name("label").is_some()), &[])
            }
            _ => (self.logical_sequences(node), &[]),
        })
    }

    /// Counts runs of like logical operators in the expression rooted here (`a && b || c` is 2).
    fn logical_sequences(&self, node: Node<'_>) -> u32 {
        if self.logical_operator(node).is_none() {
            return 0;
        }
        let mut parent = node.parent();
        while let Some(wrapper) = parent.filter(|p| p.kind() == "parenthesized_expression") {
            parent = wrapper.parent();
        }
        if parent
            .and_then(|parent| self.logical_operator(parent))
            .is_some()
        {
            return 0;
        }
        let mut operators = Vec::new();
        self.flatten(node, &mut operators);
        1 + operators
            .windows(2)
            .filter(|pair| pair[0] != pair[1])
            .count() as u32
    }

    /// Lists the logical operators of a parenthesized logical expression in source order.
    fn flatten(&self, node: Node<'_>, operators: &mut Vec<&'static str>) {
        if self.logical_operator(node).is_none() && node.kind() != "parenthesized_expression" {
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if self.spec.logical.contains(&child.kind()) {
                operators.push(child.kind());
            } else {
                self.flatten(child, operators);
            }
        }
    }

    fn logical_operator(&self, node: Node<'_>) -> Option<&'static str> {
        let operator = node.child_by_field_name("operator")?.kind();
        self.spec.logical.contains(&operator).then_some(operator)
    }
}
