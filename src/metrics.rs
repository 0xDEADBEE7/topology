//! Holds the per-function and per-file measurements produced for every language.

/// Reports a function's source extent, documentation, and cognitive complexity.
pub struct FnMetrics {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    /// Code lines the function owns: those no smaller unit's range holds.
    pub loc: usize,
    pub cognitive: u32,
    pub has_docstring: bool,
}

/// Locates a class or structure in its source file.
pub struct ClassMetrics {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// Collects a file's declarations with its size, documentation, and parse status.
pub struct FileMetrics {
    pub language: &'static str,
    pub path: String,
    /// Code lines: lines with text that is neither a comment nor a Python string statement.
    pub loc: usize,
    pub has_docstring: bool,
    /// Counts syntax-error nodes; nonzero means only the recoverable code was measured.
    pub parse_errors: usize,
    pub fns: Vec<FnMetrics>,
    pub classes: Vec<ClassMetrics>,
    pub imports: Vec<Import>,
}

/// A module a file imports, as written, and the Python name taken from it.
pub struct Import {
    pub line: usize,
    pub source: String,
    pub name: Option<String>,
}
