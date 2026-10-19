//! Extracts file and symbol descriptions from source comments.

mod file;
mod preceding;
use file::{leading_docs, opens_block};
use preceding::preceding_docs;

/// Describe a file by its leading module documentation, in any supported language.
pub(crate) fn module_doc(source: &str) -> Option<String> {
    leading_docs(&source.lines().collect::<Vec<_>>())
}

/// Extract documentation for the declaration that starts on `line`.
pub(crate) fn docstring(language: &str, source: &str, line: usize) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    if language == "python" {
        let start = lines
            .iter()
            .skip(line)
            .take(12)
            .position(|text| opens_block(text))?;
        return leading_docs(&lines[line + start..]);
    }
    preceding_docs(&lines, line, language)
}
