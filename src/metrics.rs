//! Holds the per-function and per-file measurements produced by language adapters.

use crate::visit::RawCounts;

#[derive(Debug)]
/// Reports a function's source extent, documentation, and cognitive complexity.
pub struct FnMetrics {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub loc: usize,
    pub cognitive: u32,
    pub has_docstring: bool,
}

#[derive(Debug)]
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

/// Builds a file's measurements from the raw counts of its functions.
pub fn file(
    language: &'static str,
    path: &str,
    raw: &[RawCounts],
    classes: Vec<ClassMetrics>,
    has_docstring: bool,
    loc: usize,
) -> FileMetrics {
    let fns = raw
        .iter()
        .map(|r| FnMetrics {
            name: r.name.clone(),
            line_start: r.line_start,
            line_end: r.line_end,
            loc: r.line_end.saturating_sub(r.line_start) + 1,
            cognitive: r.cognitive,
            has_docstring: r.has_docstring,
        })
        .collect();
    FileMetrics {
        language,
        path: path.to_owned(),
        loc,
        has_docstring,
        parse_errors: 0,
        fns,
        classes,
    }
}
