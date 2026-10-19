//! Reads leading module documentation comments.

/// Collects leading module descriptions before source declarations.
pub(super) fn file_docs(lines: &[&str]) -> Option<String> {
    let mut docs = Vec::new();
    for line in lines {
        let line = line.trim();
        if let Some(text) = line.strip_prefix("//!") {
            docs.push(text.trim().to_owned());
        } else if let Some(text) = line.strip_prefix("/**") {
            docs.push(text.trim().trim_end_matches("*/").trim().to_owned());
            break;
        } else if !line.is_empty() {
            break;
        }
    }
    docs.dedup();
    (!docs.is_empty()).then(|| docs.join(" "))
}
