//! Parses source with tree-sitter into the record every command reads: what one file
//! declares, imports and measures, with each function unit measured from its language's
//! table.

use crate::{
    lang::{Language, Spec},
    syntax,
};
use std::cmp::Reverse;
use tree_sitter::{Node, Parser};

/// Names of the fields a construct nests one level deeper.
type Fields = &'static [&'static str];

const NOTHING: Fields = &[];
const BODY: Fields = &["body"];

/// A function, or a class with zero metrics: its extent, description, header and measures.
pub struct Unit {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    /// Code lines the function owns: those no smaller unit's range holds.
    pub loc: usize,
    pub cognitive: u32,
    /// First paragraph of its doc comment or docstring.
    pub doc: Option<String>,
    /// Declaration text up to the body, with whitespace collapsed.
    pub signature: String,
    /// Declared with `export` or `pub`, named by a local `export { }` or `export default`,
    /// or a public top-level Python name.
    pub exported: bool,
    /// The class, impl or trait whose name qualifies this one.
    pub parent: Option<String>,
}

/// A module a file imports, as written, and the Python name taken from it.
pub struct Import {
    pub line: usize,
    pub source: String,
    pub name: Option<String>,
}

/// Collects a file's declarations, imports, size, description and parse status.
pub struct FileInfo {
    pub language: &'static str,
    pub path: String,
    /// Code lines: lines with text that is neither a comment nor a Python string statement.
    pub loc: usize,
    pub doc: Option<String>,
    /// Counts syntax-error nodes; nonzero means only the recoverable code was measured.
    pub parse_errors: usize,
    pub fns: Vec<Unit>,
    pub classes: Vec<Unit>,
    pub imports: Vec<Import>,
}

impl FileInfo {
    /// Pairs each declaration with its kind: `function`, or `class` (`struct` in Rust).
    pub fn units(&self) -> impl Iterator<Item = (&Unit, &'static str)> {
        let class = match self.language {
            "rust" => "struct",
            _ => "class",
        };
        let fns = self.fns.iter().map(|unit| (unit, "function"));
        fns.chain(self.classes.iter().map(move |unit| (unit, class)))
    }
}

/// Measures one source file, warning when only part of it could be parsed.
pub fn analyse(path: &str, source: &str) -> Result<FileInfo, String> {
    let language = Language::from_path(path).ok_or("unsupported language")?;
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
        source,
        code: vec![false; source.lines().count() + 1],
        fns: Vec::new(),
        classes: Vec::new(),
        imports: Vec::new(),
        spans: Vec::new(),
        exports: syntax::export_names(tree.root_node(), source),
    };
    walker.walk(tree.root_node(), None, 0);
    let parse_errors = syntax_errors(tree.root_node());
    if parse_errors > 0 {
        eprintln!("warning: {path}: syntax errors: {parse_errors}; measured the recoverable code");
    }
    Ok(FileInfo {
        language: language.name(),
        path: path.to_owned(),
        loc: walker.code.iter().filter(|code| **code).count(),
        doc: syntax::module_doc(spec, source, tree.root_node()),
        parse_errors,
        fns: walker.units(),
        classes: walker.classes,
        imports: walker.imports,
    })
}

/// Counts the ERROR and MISSING nodes left by tree-sitter's error recovery.
fn syntax_errors(node: Node<'_>) -> usize {
    if node.is_error() || node.is_missing() {
        return 1;
    }
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).filter(|child| child.has_error());
    children.map(syntax_errors).sum()
}

/// The value a table gives a node kind, such as the field naming a `pair` or the fields a
/// structure nests.
fn lookup<T: Copy>(table: &[(&str, T)], kind: &str) -> Option<T> {
    let entry = table.iter().find(|(key, _)| *key == kind);
    entry.map(|(_, value)| *value)
}

/// Walks one syntax tree using its language's table, collecting units, classes, imports
/// and code.
struct Walker<'a> {
    spec: &'static Spec,
    source: &'a str,
    /// Whether each row holds code: text that is neither a comment nor a string statement.
    code: Vec<bool>,
    fns: Vec<Unit>,
    classes: Vec<Unit>,
    imports: Vec<Import>,
    /// Bytes from each unit's declaration to its end, which break ties between equal rows.
    spans: Vec<usize>,
    /// Names that `export { a }` or `export default a` expose.
    exports: Vec<String>,
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
            self.classes.push(self.unit(node, self.class_name(node)));
        }
        syntax::imports(node, self.source, &mut self.imports);
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
        let span = node.end_byte() - self.declaration(node).start_byte();
        self.spans.push(span);
        self.fns.push(self.unit(node, name));
        self.fns.len() - 1
    }

    /// Describes a function or class from its declaration: where it starts, its doc, its
    /// header up to the body, whether it is exported, and its container.
    fn unit(&self, node: Node<'_>, name: String) -> Unit {
        let declaration = self.declaration(node);
        let anchor = syntax::anchor(declaration);
        Unit {
            line_start: declaration.start_position().row + 1,
            line_end: node.end_position().row + 1,
            loc: 0,
            cognitive: 0,
            doc: syntax::doc(self.spec, self.source, node, anchor),
            signature: syntax::signature(self.source, node, anchor),
            exported: syntax::exported(declaration, &name, &self.exports),
            parent: self.scope(node),
            name,
        }
    }

    /// Gives each code row to the smallest unit holding it, in rows and then bytes, and to
    /// the earlier one on equal ranges.
    fn units(&mut self) -> Vec<Unit> {
        let mut owners = vec![None; self.code.len()];
        let mut order: Vec<_> = (0..self.fns.len()).collect();
        let rows = |i: usize| self.fns[i].line_end - self.fns[i].line_start;
        order.sort_by_key(|&i| Reverse((rows(i), self.spans[i], i)));
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
    /// grammar's hidden tokens, as code; Unicode and zero-width spaces are blank.
    fn mark_code(&mut self, (row, start): (usize, usize), end: usize) {
        let text = self.source.get(start..end).unwrap_or_default();
        let blank = |c: char| c.is_whitespace() || "\u{200B}\u{2060}\u{FEFF}".contains(c);
        for (code, line) in self.code.iter_mut().skip(row).zip(text.split('\n')) {
            *code |= !line.trim_matches(blank).is_empty();
        }
    }

    /// Names a function by itself, its binding, or the call receiving it; None folds a nested
    /// callback.
    fn unit_name(&self, node: Node<'_>, nested: bool) -> Option<String> {
        let own = node
            .child_by_field_name("name")
            .and_then(|name| self.text(name));
        own.map(|name| self.qualify(node, name))
            .or_else(|| self.bound_name(node, nested))
            .or_else(|| self.call_name(node, nested))
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
        let name = current.child_by_field_name(lookup(self.spec.bindings, current.kind())?)?;
        let name = self.text(name.child_by_field_name("property").unwrap_or(name))?;
        let name = name.trim_matches(['"', '\'']);
        let simple = !name.is_empty() && !name.contains(['{', '[', ' ']);
        simple.then(|| self.qualify(current, name.to_owned()))
    }

    /// Names a callback after the call receiving it and a string passed before it, such as
    /// `it("undoes")` or `app.get("/users")`, or, outside any unit, as `describe(…)`.
    fn call_name(&self, node: Node<'_>, nested: bool) -> Option<String> {
        let arguments = node.parent().filter(|parent| parent.kind() == "arguments");
        let call = arguments.and_then(|arguments| arguments.parent());
        let callee = call.and_then(|call| self.callee(call.child_by_field_name("function")?));
        let label = (arguments.and_then(|arguments| arguments.named_child(0))).filter(|first| {
            first.id() != node.id() && matches!(first.kind(), "string" | "template_string")
        });
        let label = label.and_then(|label| self.text(label));
        let label = label.map(|label| label.lines().map(str::trim).collect::<Vec<_>>().join(" "));
        match (callee, label) {
            (Some(callee), Some(label)) => Some(format!("{callee}({label})")),
            (Some(callee), None) if !nested => Some(format!("{callee}(…)")),
            _ => (!nested).then(|| "<anonymous>".to_owned()),
        }
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
        let arguments_before = arguments
            .named_children(&mut cursor)
            .filter(|a| !a.is_extra());
        let mut earlier = arguments_before.take_while(|a| a.id() != child.id());
        let wrapper = arguments
            .parent()
            .is_some_and(|call| self.is_wrapper_call(call));
        arguments.kind() == "arguments"
            && !(wrapper && earlier.next().is_some())
            && !earlier
                .any(|a| self.spec.functions.contains(&a.kind()) || a.kind() == "call_expression")
    }

    /// Prefixes a member with its containers up to the nearest function or detached
    /// expression, as in `View.render`, `Type::method` or `Outer.Inner.method`.
    fn qualify(&self, member: Node<'_>, name: String) -> String {
        match self.scope(member) {
            Some(scope) => format!("{scope}{}{name}", self.spec.separator),
            None => name,
        }
    }

    /// Names the class, impl or trait holding a member, unless a function or a detached
    /// expression (an object, decorator, heritage clause or static block) holds it first.
    fn scope(&self, member: Node<'_>) -> Option<String> {
        let spec = self.spec;
        let stops = [spec.functions, spec.closures, spec.scopes, spec.detached].concat();
        let mut ancestors = std::iter::successors(member.parent(), |node| node.parent());
        let scope = ancestors.find(|node| stops.contains(&node.kind()))?;
        let scoped = spec.scopes.contains(&scope.kind());
        scoped.then(|| self.class_name(scope))
    }

    /// Names a class or container by itself (qualified), its implemented type's own name, or
    /// its binding.
    fn class_name(&self, class: Node<'_>) -> String {
        let implemented = || {
            let named = std::iter::successors(class.child_by_field_name("type"), |node| {
                let field = match node.kind() {
                    "reference_type" | "generic_type" => "type",
                    "scoped_type_identifier" => "name",
                    _ => return None,
                };
                node.child_by_field_name(field)
            });
            let text = self.text(named.last()?)?;
            Some(text.split_whitespace().collect::<Vec<_>>().join(" "))
        };
        let own = (class.child_by_field_name("name"))
            .and_then(|node| self.text(node))
            .or_else(implemented);
        own.map(|name| self.qualify(class, name))
            .or_else(|| self.bound_name(class, true))
            .unwrap_or_else(|| "<anonymous>".to_owned())
    }

    fn text(&self, node: Node<'_>) -> Option<String> {
        self.source.get(node.byte_range()).map(str::to_owned)
    }

    /// Follows wrappers and declarations so a unit starts where its JSDoc attaches.
    fn declaration<'t>(&self, mut node: Node<'t>) -> Node<'t> {
        while let Some(parent) = node.parent().filter(|parent| self.climbs(*parent, node)) {
            node = parent;
        }
        node
    }

    /// Whether a declaration reaches `parent`: a wrapper, a declaration, a binding, the
    /// arguments that bind `child`, or a call, unless that call is a method whose value is
    /// neither bound nor declared (so `items.filter(cb)` starts at its arguments but
    /// `it.each(table)(cb)` takes its table).
    fn climbs(&self, parent: Node<'_>, child: Node<'_>) -> bool {
        let (spec, kind) = (self.spec, parent.kind());
        let through = spec.wrappers.contains(&kind) || spec.declarations.contains(&kind);
        let method = parent
            .child_by_field_name("function")
            .is_some_and(|callee| callee.kind() == "member_expression");
        (through || lookup(spec.bindings, kind).is_some())
            && match kind {
                "arguments" => self.passes_arguments(parent, child),
                "call_expression" if method => {
                    (parent.parent()).is_some_and(|outer| self.climbs(outer, parent))
                }
                _ => true,
            }
    }

    /// Recognizes calls such as `memo(...)` or `React.forwardRef(...)` that wrap a component.
    fn is_wrapper_call(&self, call: Node<'_>) -> bool {
        let function = call.child_by_field_name("function");
        let callee = function.and_then(|f| self.text(f)).unwrap_or_default();
        let name = callee.rsplit('.').next().unwrap_or_default();
        self.spec.wrapper_calls.contains(&name)
    }

    /// Adds the increment of a structure, `if`/`else`, labelled jump or logical run to `unit`
    /// and returns the fields it nests.
    fn increment(&mut self, node: Node<'_>, unit: Option<usize>, nesting: u32) -> Fields {
        let (spec, kind) = (self.spec, node.kind());
        let [if_kind, else_kind] = spec.conditional;
        let parent_is = |kind| node.parent().is_some_and(|p| p.kind() == kind);
        let first_is = |kind| {
            let mut cursor = node.walk();
            let mut statements = node.named_children(&mut cursor).filter(|c| !c.is_extra());
            statements.next().is_some_and(|c| c.kind() == kind)
        };
        let (increment, nested): (u32, Fields) = match lookup(spec.structures, kind) {
            Some(nested) => (1 + nesting, nested),
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
        let first = jump.named_child(0).map(|child| child.kind());
        jump.child_by_field_name("label").is_some() || first == Some("label")
    }

    /// Counts runs of like logical operators in the expression rooted here (`a && b || c` is 2).
    fn logical_sequences(&self, node: Node<'_>) -> u32 {
        if !self.is_logical(node) {
            return 0;
        }
        let mut ancestors = std::iter::successors(node.parent(), |node| node.parent());
        let parent = ancestors.find(|parent| parent.kind() != "parenthesized_expression");
        if parent.is_some_and(|parent| self.is_logical(parent)) {
            return 0;
        }
        let mut operators = Vec::new();
        self.flatten(node, &mut operators);
        operators.dedup();
        operators.len() as u32
    }

    /// Lists the logical operators of a logical expression, through parentheses, in source
    /// order.
    fn flatten(&self, node: Node<'_>, operators: &mut Vec<&'static str>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if self.spec.logical.contains(&child.kind()) {
                operators.push(child.kind());
            } else if self.is_logical(child) || child.kind() == "parenthesized_expression" {
                self.flatten(child, operators);
            }
        }
    }

    /// Recognizes a logical expression; a Rust let chain is a run of `&&`.
    fn is_logical(&self, node: Node<'_>) -> bool {
        let operator = node.child_by_field_name("operator");
        let logical = operator.is_some_and(|operator| self.spec.logical.contains(&operator.kind()));
        node.kind() == "let_chain" || logical
    }
}
