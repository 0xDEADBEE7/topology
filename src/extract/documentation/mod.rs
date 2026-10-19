//! Extracts file and symbol descriptions from source comments.

mod file;
mod preceding;
use file::file_docs;
use preceding::preceding_docs;

/// Extract documentation immediately preceding a file or symbol.
pub(crate) fn docstring(
    language: &str,
    source: &str,
    symbol_line: Option<usize>,
) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    if language == "python" {
        let start = symbol_line.map_or(0, |line| line.saturating_sub(1));
        return lines[start..]
            .iter()
            .take(12)
            .map(|line| line.trim())
            .skip_while(|line| line.is_empty())
            .find(|line| line.starts_with("\"\"\"") || line.starts_with("'''"))
            .and_then(quoted_doc);
    }
    if symbol_line.is_none() {
        return file_docs(&lines);
    }
    preceding_docs(&lines, symbol_line.unwrap_or(0), language)
}

fn quoted_doc(line: &str) -> Option<String> {
    let quote = if line.starts_with("\"\"\"") {
        "\"\"\""
    } else {
        "'''"
    };
    let text = line.strip_prefix(quote)?;
    Some(text.strip_suffix(quote).unwrap_or(text).trim().to_owned())
}
