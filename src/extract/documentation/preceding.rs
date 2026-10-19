//! Reads documentation immediately above a source declaration.

/// Collects the comment block immediately above a declaration.
pub(super) fn preceding_docs(lines: &[&str], end: usize, language: &str) -> Option<String> {
    let mut docs = Vec::new();
    let mut block = false;
    for line in lines[..end.saturating_sub(1).min(lines.len())].iter().rev() {
        let line = line.trim();
        if line == "*/" {
            block = true;
            continue;
        }
        let text = if language == "rust" {
            line.strip_prefix("///")
        } else if block {
            line.strip_prefix("/**")
                .or_else(|| line.strip_prefix('*'))
                .map(str::trim)
        } else {
            line.strip_prefix("/**").or_else(|| line.strip_prefix("//"))
        };
        let Some(text) = text else {
            break;
        };
        docs.push(text.trim().trim_end_matches("*/").trim().to_owned());
        if line.starts_with("/**") {
            break;
        }
    }
    (!docs.is_empty()).then(|| {
        docs.reverse();
        docs.join(" ")
    })
}
