//! Holds the per-function and per-file measurements produced by language adapters.

/// Reports a function's source extent, documentation, and cognitive complexity.
pub struct FnMetrics {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub cognitive: u32,
    pub has_docstring: bool,
}

impl FnMetrics {
    /// Counts the physical lines the function spans.
    pub fn loc(&self) -> usize {
        self.line_end.saturating_sub(self.line_start) + 1
    }
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
    pub loc: usize,
    pub has_docstring: bool,
    /// Counts syntax-error nodes; nonzero means only the recoverable code was measured.
    pub parse_errors: usize,
    pub fns: Vec<FnMetrics>,
    pub classes: Vec<ClassMetrics>,
}
