//! Extracts file and symbol descriptions from source comments.

mod file;
mod preceding;
use file::{leading_docs, opens_block};
use preceding::preceding_docs;

/// Describe a file by its leading module documentation, in any supported language.
pub(crate) fn module_doc(source: &str) -> Option<String> {
    let source = source.trim_start_matches('\u{feff}');
    leading_docs(&source.lines().collect::<Vec<_>>())
}

/// Extract documentation for the declaration that starts on `line`.
pub(crate) fn docstring(language: &str, source: &str, line: usize) -> Option<String> {
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
