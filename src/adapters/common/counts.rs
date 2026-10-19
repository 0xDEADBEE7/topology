//! Supplies approximate line-oriented token and decision counts.

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

/// Estimates operation counts from source text.
pub(super) fn operator_count(line: &str) -> u32 {
    ["=", "+", "-", "*", "/", "==", "=>", "<", ">", "&&", "||"]
        .iter()
        .map(|operator| line.matches(operator).count() as u32)
        .sum()
}

/// Counts identifier-like tokens as approximate operands.
pub(super) fn operand_count(line: &str) -> u32 {
    line.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|part| !part.is_empty())
        .count() as u32
}

/// Counts opening braces for approximate nesting.
pub(super) fn open_count(line: &str) -> u32 {
    line.matches('{').count() as u32
}
/// Counts closing braces for approximate nesting.
pub(super) fn close_count(line: &str) -> u32 {
    line.matches('}').count() as u32
}
