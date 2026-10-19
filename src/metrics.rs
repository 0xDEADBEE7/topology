//! Holds what one source file declares, imports and measures, read once from its syntax tree.

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
    /// Declared with `export` or `pub`, or a public top-level Python name.
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
        let class = if self.language == "rust" {
            "struct"
        } else {
            "class"
        };
        let fns = self.fns.iter().map(|unit| (unit, "function"));
        fns.chain(self.classes.iter().map(move |unit| (unit, class)))
    }
}
