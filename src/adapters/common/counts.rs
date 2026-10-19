//! Supplies approximate line-oriented decision and nesting counts.

/// Estimates branching from control-flow words and logical operators.
pub(super) fn decision_count(language: &str, line: &str) -> u32 {
    let words = if language == "python" {
        ["if ", "elif ", "for ", "while ", "except ", "match "]
    } else {
        ["if ", "else if ", "for ", "while ", "case ", "catch "]
    };
    words
        .iter()
        .map(|word| line.matches(word).count() as u32)
        .sum::<u32>()
        + line.matches("&&").count() as u32
        + line.matches("||").count() as u32
}

/// Counts opening braces for approximate nesting.
pub(super) fn open_count(line: &str) -> u32 {
    line.matches('{').count() as u32
}
/// Counts closing braces for approximate nesting.
pub(super) fn close_count(line: &str) -> u32 {
    line.matches('}').count() as u32
}
