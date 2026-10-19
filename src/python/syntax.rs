//! Recognizes scripting declarations and documentation boundaries.

/// Recognizes leading documentation in supported scripting syntax.
pub(super) fn has_file_docstring(language: &str, source: &str) -> bool {
    if language == "python" {
        return source
            .lines()
            .find(|line| !line.trim().is_empty())
            .is_some_and(|line| {
                let line = line.trim();
                line.starts_with("\"\"\"") || line.starts_with("''' ") || line == "'''"
            });
    }
    source
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.trim_start().starts_with("/**"))
}

/// Recognizes a docstring after a Python declaration.
pub(super) fn has_python_docstring(source: &str, index: usize) -> bool {
    let lines: Vec<&str> = source.lines().skip(index + 1).collect();
    lines
        .iter()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| {
            let line = line.trim_start();
            line.starts_with("\"\"\"") || line.starts_with("'''")
        })
}
/// Extracts names from supported line-oriented declarations.
pub(super) fn function_name(language: &str, line: &str) -> Option<String> {
    let line = line.trim_start();
    let marker = if language == "python" {
        if line.starts_with("async def ") {
            "async def "
        } else {
            "def "
        }
    } else if line.starts_with("export function ") {
        "export function "
    } else if line.starts_with("function ") {
        "function "
    } else {
        return None;
    };
    let start = line.find(marker)? + marker.len();
    let name: String = line[start..]
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}
