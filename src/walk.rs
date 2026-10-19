//! Parses source with tree-sitter and measures each function unit from its language's table.

use crate::{
    lang::{Language, Spec},
    metrics::{ClassMetrics, FileMetrics, FnMetrics},
    syntax,
};
use std::cmp::Reverse;
use tree_sitter::{Node, Parser};

const NOTHING: &[&str] = &[];
const BODY: &[&str] = &["body"];

/// Measures one source file, warning when only part of it could be parsed.
pub fn analyse(path: &str, source: &str) -> Result<FileMetrics, String> {
    let language = Language::from_path(path).ok_or_else(|| unsupported(path))?;
    let (grammar, spec) = language.grammar();
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
        code: vec![false; source.lines().count() + 1],
        fns: Vec::new(),
        classes: Vec::new(),
    };
    walker.walk(tree.root_node(), None, 0);
    let parse_errors = syntax_errors(tree.root_node());
    if parse_errors > 0 {
        eprintln!("warning: {path}: syntax errors: {parse_errors}; measured the recoverable code");
    }
    Ok(FileMetrics {
        language: language.name(),
        path: path.to_owned(),
        loc: walker.code.iter().filter(|code| **code).count(),
        has_docstring: syntax::module_doc(source).is_some(),
        parse_errors,
        fns: walker.units(),
        classes: walker.classes,
    })
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

/// Walks one syntax tree using its language's table, collecting units, classes and code.
struct Walker<'a> {
    spec: &'static Spec,
    source: &'a [u8],
    /// Whether each row holds code: text that is neither a comment nor a string statement.
    code: Vec<bool>,
    fns: Vec<FnMetrics>,
    classes: Vec<ClassMetrics>,
}

impl Walker<'_> {
    /// Visits a node inside `unit`: opens named units, folds unnamed callbacks, and adds
    /// cognitive increments and code rows to the unit that owns them.
    fn walk(&mut self, node: Node<'_>, unit: Option<usize>, nesting: u32) {
        let kind = node.kind();
        if self.spec.comments.contains(&kind) || syntax::is_docstring(self.spec, node) {
            return;
        }
        let start = (node.start_position().row, node.start_byte());
        if node.child_count() == 0 {
            return self.mark_code(start, node.end_byte());
        }
        if self.spec.classes.contains(&kind) {
            self.classes.push(ClassMetrics {
                name: self.class_name(node),
                line_start: node.start_position().row + 1,
                line_end: node.end_position().row + 1,
            });
        }
        let function = self.spec.functions.contains(&kind);
        let named = function
            .then(|| self.unit_name(node, unit.is_some()))
            .flatten();
        let (unit, nesting, nested) = if let Some(name) = named {
            (Some(self.open(node, name)), 0, NOTHING)
        } else if function || self.spec.closures.contains(&kind) {
            (unit, nesting, BODY)
        } else {
            (unit, nesting, self.increment(node, unit, nesting))
        };
        let (mut cursor, mut gap) = (node.walk(), start);
        let mut more = cursor.goto_first_child();
        while more {
            let child = cursor.node();
            let deeper = nested.contains(&cursor.field_name().unwrap_or(""));
            self.mark_code(gap, child.start_byte());
            self.walk(child, unit, nesting + u32::from(deeper));
            gap = (child.end_position().row, child.end_byte());
            more = cursor.goto_next_sibling();
        }
        self.mark_code(gap, node.end_byte());
    }

    /// Starts a named unit at its declaration.
    fn open(&mut self, node: Node<'_>, name: String) -> usize {
        let declaration = self.declaration(node);
        self.fns.push(FnMetrics {
            name,
            line_start: declaration.start_position().row + 1,
            line_end: node.end_position().row + 1,
            loc: 0,
            cognitive: 0,
            has_docstring: syntax::documented(self.spec, self.source, node, declaration),
        });
        self.fns.len() - 1
    }

    /// Gives each code row to the smallest unit holding it, the earlier one on equal ranges.
    fn units(&mut self) -> Vec<FnMetrics> {
        let mut owners = vec![None; self.code.len()];
        let mut order: Vec<_> = (0..self.fns.len()).collect();
        order.sort_by_key(|&i| Reverse((self.fns[i].line_end - self.fns[i].line_start, i)));
        for i in order {
            let unit = &self.fns[i];
            owners[unit.line_start - 1..unit.line_end.min(self.code.len())].fill(Some(i));
        }
        for (owner, code) in owners.into_iter().zip(&self.code) {
            if let (Some(i), true) = (owner, code) {
                self.fns[i].loc += 1;
            }
        }
        std::mem::take(&mut self.fns)
    }

    /// Marks the rows of source text that no child node covers, such as tokens and the
    /// grammar's hidden tokens, as code.
    fn mark_code(&mut self, (row, start): (usize, usize), end: usize) {
        let text = self.source.get(start..end).unwrap_or_default();
        for (offset, line) in text.split(|byte| *byte == b'\n').enumerate() {
            let row = self.code.get_mut(row + offset);
            if let Some(code) = row.filter(|_| !line.trim_ascii().is_empty()) {
                *code = true;
            }
        }
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
        let (mut child, mut current) = (node, node.parent()?);
        loop {
            let call = current
                .parent()
                .filter(|_| self.passes_arguments(current, child));
            if self.spec.wrappers.contains(&current.kind()) {
                (child, current) = (current, current.parent()?);
            } else if let Some(call) = call.filter(|call| !nested || self.is_wrapper_call(*call)) {
                (child, current) = (call, call.parent()?);
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
        let name = current.child_by_field_name(field)?;
        let name = self.text(name.child_by_field_name("property").unwrap_or(name))?;
        let name = name.trim_matches(['"', '\'']);
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
        let callee = self.callee(arguments.parent()?.child_by_field_name("function")?)?;
        let label = self.text(label)?;
        let label = label.lines().map(str::trim).collect::<Vec<_>>().join(" ");
        Some(format!("{callee}({label})"))
    }

    /// Names a top-level callback after the call that receives it.
    fn fallback(&self, node: Node<'_>) -> String {
        let call = node.parent().filter(|parent| parent.kind() == "arguments");
        let callee = call.and_then(|call| call.parent()?.child_by_field_name("function"));
        callee
            .and_then(|callee| self.callee(callee))
            .map_or_else(|| "<anonymous>".to_owned(), |callee| format!("{callee}(…)"))
    }

    /// Renders a callee without its arguments, as in `describe.each(…)` or `app.get`.
    fn callee(&self, node: Node<'_>) -> Option<String> {
        let field = |name| node.child_by_field_name(name);
        match node.kind() {
            "call_expression" => Some(format!("{}(…)", self.callee(field("function")?)?)),
            "member_expression" => {
                let object = self.callee(field("object")?)?;
                Some(format!("{object}.{}", self.text(field("property")?)?))
            }
            _ => self
                .text(node)
                .filter(|text| !text.contains('\n'))
                .or(Some("…".to_owned())),
        }
    }

    /// Whether `child` is a callback its call binds: the first argument of a wrapper call, or
    /// one that no function or call argument precedes.
    fn passes_arguments(&self, arguments: Node<'_>, child: Node<'_>) -> bool {
        let mut cursor = arguments.walk();
        let mut earlier =
            (arguments.named_children(&mut cursor)).take_while(|a| a.id() != child.id());
        let wrapper = arguments
            .parent()
            .is_some_and(|call| self.is_wrapper_call(call));
        arguments.kind() == "arguments"
            && !(wrapper && earlier.next().is_some())
            && !earlier
                .any(|a| self.spec.functions.contains(&a.kind()) || a.kind() == "call_expression")
    }

    /// Prefixes a member with its containers up to the nearest function, as in `View.render`,
    /// `Type::method` or `Outer.Inner.method`.
    fn qualify(&self, member: Node<'_>, name: String) -> String {
        let spec = self.spec;
        let stops = [spec.functions, spec.closures, spec.scopes];
        let mut ancestors = std::iter::successors(member.parent(), |node| node.parent());
        let scope = ancestors.find(|node| stops.iter().any(|kinds| kinds.contains(&node.kind())));
        match scope.filter(|scope| spec.scopes.contains(&scope.kind())) {
            Some(scope) => format!("{}{}{name}", self.class_name(scope), spec.separator),
            None => name,
        }
    }

    /// Names a class or container by itself (qualified), its implemented type's own name, or
    /// its binding.
    fn class_name(&self, class: Node<'_>) -> String {
        let field = |name| {
            class
                .child_by_field_name(name)
                .and_then(|node| self.text(node))
        };
        let implemented = || {
            let path = field("type")?;
            let path = path.split('<').next()?.rsplit([':', ' ', '&']).next()?;
            Some(path.to_owned())
        };
        let own = field("name").or_else(implemented);
        own.map(|name| self.qualify(class, name))
            .or_else(|| self.bound_name(class, true))
            .unwrap_or_else(|| "<anonymous>".to_owned())
    }

    fn text(&self, node: Node<'_>) -> Option<String> {
        node.utf8_text(self.source).ok().map(str::to_owned)
    }

    /// Follows wrappers and declarations so a unit starts where its JSDoc attaches.
    fn declaration<'t>(&self, mut node: Node<'t>) -> Node<'t> {
        while let Some(parent) = node.parent().filter(|parent| self.climbs(*parent, node)) {
            node = parent;
        }
        node
    }

    /// Whether a declaration reaches `parent`: a wrapper, a declaration, the arguments that
    /// bind `child`, or a call, unless that call is a method whose value is not declared (so
    /// `items.filter(cb)` starts at its arguments but `it.each(table)(cb)` takes its table).
    fn climbs(&self, parent: Node<'_>, child: Node<'_>) -> bool {
        let kind = parent.kind();
        let through = self.spec.wrappers.contains(&kind) || self.spec.declarations.contains(&kind);
        let method = parent
            .child_by_field_name("function")
            .is_some_and(|callee| callee.kind() == "member_expression");
        through
            && match kind {
                "arguments" => self.passes_arguments(parent, child),
                "call_expression" if method => parent
                    .parent()
                    .is_some_and(|outer| self.climbs(outer, parent)),
                _ => true,
            }
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

    /// Adds the increment of a structure, `if`/`else`, labelled jump or logical run to `unit`
    /// and returns the fields it nests.
    fn increment(
        &mut self,
        node: Node<'_>,
        unit: Option<usize>,
        nesting: u32,
    ) -> &'static [&'static str] {
        let (spec, kind) = (self.spec, node.kind());
        let structure = spec
            .structures
            .iter()
            .find(|(structure, _)| *structure == kind);
        let [if_kind, else_kind] = spec.conditional;
        let parent_is = |kind| node.parent().is_some_and(|p| p.kind() == kind);
        let first_is = |kind| {
            let mut cursor = node.walk();
            let mut statements = node.named_children(&mut cursor).filter(|c| !c.is_extra());
            statements.next().is_some_and(|c| c.kind() == kind)
        };
        let (increment, nested): (u32, &[&str]) = match structure {
            Some((_, nested)) => (1 + nesting, *nested),
            None if kind == if_kind && parent_is(else_kind) => (1, &["consequence"]),
            None if kind == if_kind => (1 + nesting, &["consequence"]),
            None if spec.elif.contains(&kind) => (1, &["consequence"]),
            None if kind == else_kind && first_is(if_kind) => (0, NOTHING),
            None if kind == else_kind => (1, &["", "body"]),
            None if spec.jumps.contains(&kind) => (u32::from(self.labelled(node)), NOTHING),
            None => (self.logical_sequences(node), NOTHING),
        };
        if let Some(index) = unit {
            self.fns[index].cognitive += increment;
        }
        nested
    }

    /// Recognizes a jump to a label, given as a field (TypeScript) or a child (Rust).
    fn labelled(&self, jump: Node<'_>) -> bool {
        jump.child_by_field_name("label").is_some()
            || jump
                .named_child(0)
                .is_some_and(|child| child.kind() == "label")
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

    /// The operator of a logical expression; a Rust let chain is a run of `&&`.
    fn logical_operator(&self, node: Node<'_>) -> Option<&'static str> {
        if node.kind() == "let_chain" {
            return Some("&&");
        }
        let operator = node.child_by_field_name("operator")?.kind();
        self.spec.logical.contains(&operator).then_some(operator)
    }
}
