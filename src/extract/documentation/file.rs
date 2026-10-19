//! Reads leading documentation comments and docstrings.

/// Openers and terminators of block comments and docstrings that carry documentation.
const BLOCKS: [(&str, &str); 6] = [
    ("/*!", "*/"),
    ("/**", "*/"),
    ("\"\"\"", "\"\"\""),
    ("'''", "'''"),
    ("r\"\"\"", "\"\"\""),
    ("r'''", "'''"),
];

/// Recognizes the start of a documentation block comment or docstring.
pub(super) fn opens_block(line: &str) -> bool {
    BLOCKS.iter().any(|(open, _)| line.trim().starts_with(open))
}

/// Summarizes the first leading documentation block, skipping license headers and pragmas.
pub(super) fn leading_docs(lines: &[&str]) -> Option<String> {
    let mut index = 0;
    while let Some(line) = lines.get(index).map(|line| line.trim()) {
        index += 1;
        let text = if let Some(first) = line.strip_prefix("//!") {
            line_docs(lines, &mut index, first)
        } else if let Some((open, close)) = BLOCKS.iter().find(|(open, _)| line.starts_with(open)) {
            block_docs(lines, &mut index, &line[open.len()..], close)
        } else if is_preamble(line) {
            continue;
        } else {
            return None;
        };
        if let Some(summary) = summary(&text) {
            return Some(summary);
        }
    }
    None
}

/// Recognizes lines that may precede module documentation: blanks, comments,
/// shebangs, attributes and directives such as `'use client'`.
fn is_preamble(line: &str) -> bool {
    line.is_empty()
        || line.starts_with("//")
        || line.starts_with('#')
        || line.starts_with("'use ")
        || line.starts_with("\"use ")
}

/// Collects a run of `//!` lines that starts with `first`.
fn line_docs<'a>(lines: &[&'a str], index: &mut usize, first: &'a str) -> Vec<&'a str> {
    let mut text = vec![first];
    while let Some(next) = lines.get(*index).and_then(|l| l.trim().strip_prefix("//!")) {
        text.push(next);
        *index += 1;
    }
    text
}

/// Collects a block comment or docstring body up to its terminator.
fn block_docs<'a>(
    lines: &[&'a str],
    index: &mut usize,
    first: &'a str,
    close: &str,
) -> Vec<&'a str> {
    let mut text = Vec::new();
    let mut rest = first;
    while !rest.contains(close) && *index < lines.len() {
        text.push(rest);
        rest = lines[*index].trim();
        *index += 1;
    }
    text.push(rest.split(close).next().unwrap_or(rest));
    text
}

/// Joins the first descriptive paragraph unless it is a license header or a pragma.
fn summary(text: &[&str]) -> Option<String> {
    let lines: Vec<&str> = text
        .iter()
        .map(|line| line.trim().trim_start_matches('*').trim())
        .collect();
    let summary = lines
        .split(|line| line.is_empty())
        .find(|paragraph| !paragraph.is_empty() && !is_heading(paragraph))?
        .join(" ");
    let lower = summary.to_ascii_lowercase();
    let license = [
        "copyright",
        "spdx-license-identifier",
        "@license",
        "licensed under",
    ]
    .iter()
    .any(|marker| lower.contains(marker));
    (!license && !summary.starts_with('@')).then_some(summary)
}

/// Recognizes a reStructuredText title: one line of text above an underline.
fn is_heading(paragraph: &[&str]) -> bool {
    matches!(paragraph, [_, underline] if underline.chars().all(|c| "=-~^".contains(c)))
}
