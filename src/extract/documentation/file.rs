//! Reads leading documentation comments and docstrings.

/// Block comments and docstrings: opener, terminator, and whether the block documents code.
const BLOCKS: [(&str, &str, bool); 7] = [
    ("/*!", "*/", true),
    ("/**", "*/", true),
    ("\"\"\"", "\"\"\"", true),
    ("'''", "'''", true),
    ("r\"\"\"", "\"\"\"", true),
    ("r'''", "'''", true),
    ("/*", "*/", false),
];

/// Recognizes the start of a documentation block comment or docstring.
pub(super) fn opens_block(line: &str) -> bool {
    BLOCKS
        .iter()
        .any(|(open, _, documents)| *documents && line.trim().starts_with(open))
}

/// Summarizes the first leading documentation block, skipping license headers and pragmas.
pub(super) fn leading_docs(lines: &[&str]) -> Option<String> {
    let mut index = 0;
    while let Some(line) = lines.get(index).map(|line| line.trim()) {
        index += 1;
        let text = if let Some(first) = line.strip_prefix("//!") {
            line_docs(lines, &mut index, first)
        } else if is_preamble(line) {
            continue;
        } else if let Some((open, close, documents)) =
            BLOCKS.iter().find(|(open, ..)| line.starts_with(open))
        {
            block_docs(lines, &mut index, &line[open.len()..], close)
                .filter(|_| *documents)
                .unwrap_or_default()
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
        || line.starts_with("/**/")
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

/// Collects a block comment or docstring body up to its terminator, if it has one.
fn block_docs<'a>(
    lines: &[&'a str],
    index: &mut usize,
    first: &'a str,
    close: &str,
) -> Option<Vec<&'a str>> {
    let mut text = Vec::new();
    let mut rest = first;
    while !rest.contains(close) {
        text.push(rest);
        rest = lines.get(*index)?.trim();
        *index += 1;
    }
    text.push(rest.split(close).next().unwrap_or(rest));
    Some(text)
}

/// Joins the first paragraph up to its tags (`@param`), unless it is a license or a pragma.
fn summary(text: &[&str]) -> Option<String> {
    let lines: Vec<&str> = text
        .iter()
        .map(|line| line.trim().trim_start_matches('*').trim())
        .collect();
    let paragraph = lines
        .split(|line| line.is_empty())
        .find(|paragraph| !paragraph.is_empty() && !is_heading(paragraph))?;
    let lower = paragraph.join(" ").to_ascii_lowercase();
    let license = ["copyright", "spdx-license", "@license", "licensed under"]
        .iter()
        .any(|marker| lower.contains(marker));
    let prose: Vec<&str> = (paragraph.iter())
        .take_while(|line| !line.starts_with('@'))
        .copied()
        .collect();
    (!license && !prose.is_empty()).then(|| prose.join(" "))
}

/// Recognizes a reStructuredText title: one line of text above an underline.
fn is_heading(paragraph: &[&str]) -> bool {
    matches!(paragraph, [_, underline] if underline.chars().all(|c| "=-~^".contains(c)))
}
