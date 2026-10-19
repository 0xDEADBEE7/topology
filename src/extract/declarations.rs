//! Determines declaration visibility and export status.

/// Infer source visibility from the declaration line.
pub(super) fn visibility(language: &str, source: &str, line: usize) -> &'static str {
    let text = source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim_start();
    if (language == "rust" && text.starts_with("pub ")) || text.starts_with("export ") {
        "public"
    } else if language == "python" && text.contains("__") {
        "private"
    } else if language == "python" {
        "public"
    } else {
        "private"
    }
}

/// Determine whether a declaration is part of the language's public surface.
pub(super) fn is_exported(language: &str, source: &str, line: usize) -> bool {
    let text = source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim_start();
    match language {
        "rust" => text.starts_with("pub "),
        "typescript" | "javascript" => {
            text.starts_with("export ")
                || source
                    .lines()
                    .any(|l| l.contains("export") && l.contains(text))
        }
        "python" => true,
        _ => false,
    }
}
