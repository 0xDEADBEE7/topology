//! Renders repository, file, and symbol topology.

use std::{collections::BTreeSet, fs, path::Path};

/// Inspect a directory, file, or file-qualified symbol.
pub fn inspect(target: &str) -> Result<(), String> {
    if let Some((path, symbol)) = target.split_once("::") {
        return show_symbol(path, symbol);
    }
    if Path::new(target).is_file() {
        return show_file(target);
    }
    run_directory(target)
}

fn run_directory(root: &str) -> Result<(), String> {
    let files = crate::extract::file_descriptions(root)?;
    let root_name = Path::new(root)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(root);
    let max_prefix = files
        .iter()
        .map(|(path, _)| {
            let depth = path.split('/').count().saturating_sub(1);
            root_name.len() + 1 + (depth + 1) * 2 + path.rsplit('/').next().unwrap_or(path).len()
        })
        .max()
        .unwrap_or(0);
    println!("{root_name}/");
    let mut seen = BTreeSet::new();
    for (path, description) in files {
        let parts: Vec<_> = path.split('/').collect();
        let depth = parts.len().saturating_sub(1);
        for index in 0..depth {
            if seen.insert(parts[..=index].join("/")) {
                println!("{}{}/", "  ".repeat(index + 1), parts[index]);
            }
        }
        let prefix = format!(
            "{}{}",
            "  ".repeat(depth + 1),
            parts.last().copied().unwrap_or_default()
        );
        print_entry(&prefix, description, max_prefix);
    }
    Ok(())
}

fn show_file(path: &str) -> Result<(), String> {
    let (description, symbols) = crate::extract::file_outline(path)?;
    let name = path;
    print_entry(name, description, name.len());
    for symbol in symbols {
        let prefix = format!(
            "  {} (L{}–{})",
            symbol.name, symbol.lines[0], symbol.lines[1]
        );
        print_entry(
            &prefix,
            symbol.description,
            name.len() + 2 + symbol.name.len(),
        );
    }
    Ok(())
}

fn show_symbol(path: &str, name: &str) -> Result<(), String> {
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

fn print_entry(prefix: &str, description: Option<String>, width: usize) {
    match description {
        Some(description) => println!(
            "{prefix}  #{} {description}",
            "─".repeat(width.saturating_sub(prefix.len()).max(2))
        ),
        None => println!("{prefix}"),
    }
}
