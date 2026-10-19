//! Displays file outlines, symbol bodies, and bounded descriptions.

use std::fs;

/// Displays a file description and its declaration outline.
pub(super) fn show_file(path: &str, full_docstring: bool) -> Result<(), String> {
    let (description, symbols) = crate::extract::file_outline(path)?;
    let name = path;
    print_entry(name, description, name.len(), full_docstring);
    for symbol in symbols {
        let prefix = format!(
            "  {} (L{}–{})",
            symbol.name, symbol.lines[0], symbol.lines[1]
        );
        print_entry(
            &prefix,
            symbol.description,
            name.len() + 2 + symbol.name.len(),
            full_docstring,
        );
    }
    Ok(())
}

/// Displays the signature and body of a selected declaration.
pub(super) fn show_symbol(path: &str, name: &str) -> Result<(), String> {
    let path = path.strip_prefix("./").unwrap_or(path);
    let matches = crate::extract::find(".", &format!("{path}::{name}"))?;
    let symbol = matches
        .first()
        .ok_or_else(|| format!("symbol not found: {path}::{name}"))?;
    println!(
        "{}::{} (L{}–{})",
        symbol.path, symbol.name, symbol.lines[0], symbol.lines[1]
    );
    println!("{}", symbol.signature);
    let source = fs::read_to_string(&symbol.path).map_err(|e| e.to_string())?;
    for line in source
        .lines()
        .skip(symbol.lines[0])
        .take(symbol.lines[1].saturating_sub(symbol.lines[0]))
    {
        println!("    {line}");
    }
    Ok(())
}

/// Fits a description beside a tree entry unless full text is requested.
pub(super) fn print_entry(
    prefix: &str,
    description: Option<String>,
    target_width: usize,
    full_docstring: bool,
) {
    match description {
        Some(description) => {
            let separator_width = target_width.saturating_sub(prefix.chars().count()).max(2);
            let separator = format!("  #{} ", "─".repeat(separator_width));
            let available =
                78usize.saturating_sub(prefix.chars().count() + separator.chars().count());
            let description = if full_docstring {
                description
            } else {
                truncate_right(&description, available)
            };
            println!("{prefix}{separator}{description}");
        }
        None => println!("{prefix}"),
    }
}

fn truncate_right(text: &str, width: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= width {
        return text.to_owned();
    }
    if width <= 3 {
        return "…".repeat(width);
    }
    chars[..width - 3].iter().collect::<String>() + "..."
}
