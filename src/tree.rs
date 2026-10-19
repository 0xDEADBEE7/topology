//! Renders repository, file, and symbol topology.

use std::{collections::BTreeMap, fs, path::Path};

/// Displays repository, file, or symbol detail according to the target path.
pub fn inspect_with_options(target: &str, full_docstring: bool) -> Result<(), String> {
    if let Some((path, symbol)) = target.split_once("::") {
        return show_symbol(path, symbol);
    }
    if Path::new(target).is_file() {
        return show_file(target, full_docstring);
    }
    run_directory(target, full_docstring)
}

/// Displays a repository tree with available descriptions.
fn run_directory(root: &str, full_docstring: bool) -> Result<(), String> {
    let files = crate::extract::file_descriptions(root)?;
    let root_name = Path::new(root)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(root);
    let entries = tree_entries(root, files);
    let max_prefix = align_width(entries.iter().enumerate().map(|(index, entry)| {
        render_prefix(index, entry, &entries).chars().count()
            + entry.parts.last().map_or(0, |name| name.chars().count())
    }));
    if let Some(description) = read_docstring(Path::new(root)) {
        print_entry(
            &format!("{root_name}/"),
            Some(description),
            max_prefix + 2,
            full_docstring,
        );
    } else {
        println!("{root_name}/");
    }
    for (index, entry) in entries.iter().enumerate() {
        let prefix = format!(
            "{}{}{}",
            render_prefix(index, entry, &entries),
            entry.parts.last().map_or("", String::as_str),
            if entry.directory { "/" } else { "" }
        );
        print_entry(
            &prefix,
            entry.description.clone(),
            max_prefix + 2,
            full_docstring,
        );
    }
    Ok(())
}

/// Reads the description attached to a containing directory.
fn directory_doc(root: &str, parts: &[String]) -> Option<String> {
    read_docstring(&Path::new(root).join(parts.join("/")))
}

fn read_docstring(directory: &Path) -> Option<String> {
    let content = fs::read_to_string(directory.join(".docstring")).ok()?;
    content
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
}

/// Displays a file description and its declaration outline.
fn show_file(path: &str, full_docstring: bool) -> Result<(), String> {
    let (description, symbols) = crate::extract::file_outline(path)?;
    let rows: Vec<_> = symbols
        .into_iter()
        .map(|s| {
            (
                format!("  {} (L{}–{})", s.name, s.lines[0], s.lines[1]),
                s.description,
            )
        })
        .collect();
    let width = align_width(rows.iter().map(|(prefix, _)| prefix.chars().count()));
    print_entry(path, description, width, full_docstring);
    for (prefix, description) in rows {
        print_entry(&prefix, description, width, full_docstring);
    }
    Ok(())
}

/// Picks the description column, ignoring entries too long to align with the rest.
fn align_width(widths: impl Iterator<Item = usize>) -> usize {
    widths.filter(|width| *width <= 40).max().unwrap_or(0)
}

/// Displays the source of a declaration found in the named file only.
fn show_symbol(path: &str, name: &str) -> Result<(), String> {
    let (_, symbols) = crate::extract::file_outline(path)?;
    let symbol = symbols
        .iter()
        .find(|symbol| symbol.name == name)
        .ok_or_else(|| format!("symbol not found: {path}::{name}"))?;
    let [start, end] = symbol.lines;
    println!("{path}::{name} (L{start}–{end})");
    let source = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    for line in source
        .lines()
        .skip(start.saturating_sub(1))
        .take(end + 1 - start)
    {
        println!("{line}");
    }
    Ok(())
}

/// Fits a description beside a tree entry unless full text is requested.
fn print_entry(
    prefix: &str,
    description: Option<String>,
    target_width: usize,
    full_docstring: bool,
) {
    match description {
        Some(description) => {
            let separator_width = target_width.saturating_sub(prefix.chars().count()).max(2);
            let separator = format!("  #{} ", "─".repeat(separator_width));
            let available = 78usize
                .saturating_sub(prefix.chars().count() + separator.chars().count())
                .max(24);
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

#[derive(Debug)]
struct Entry {
    parts: Vec<String>,
    description: Option<String>,
    directory: bool,
}

/// Builds one entry per source file and containing directory, ordered by path components.
fn tree_entries(root: &str, files: Vec<(String, Option<String>)>) -> Vec<Entry> {
    let mut descriptions = BTreeMap::new();
    for (path, description) in files {
        let parts: Vec<String> = path.split('/').map(str::to_owned).collect();
        for index in 1..parts.len() {
            descriptions
                .entry(parts[..index].to_vec())
                .or_insert_with(|| directory_doc(root, &parts[..index]));
        }
        descriptions.insert(parts, description);
    }
    let paths: Vec<_> = descriptions.into_iter().collect();
    let directories: Vec<bool> = paths
        .windows(2)
        .map(|pair| pair[1].0.starts_with(&pair[0].0))
        .chain([false])
        .collect();
    paths
        .into_iter()
        .zip(directories)
        .map(|((parts, description), directory)| Entry {
            parts,
            description,
            directory,
        })
        .collect()
}

/// Renders branch connectors based on later siblings.
fn render_prefix(index: usize, entry: &Entry, entries: &[Entry]) -> String {
    let parent_len = entry.parts.len().saturating_sub(1);
    let mut prefix = String::new();
    for depth in 0..parent_len {
        prefix.push_str(
            if has_later_branch(index, &entry.parts[..=depth], entries) {
                "│  "
            } else {
                "   "
            },
        );
    }
    prefix.push_str(if has_later_branch(index, &entry.parts, entries) {
        "├─ "
    } else {
        "└─ "
    });
    prefix
}

fn has_later_branch(index: usize, path: &[String], entries: &[Entry]) -> bool {
    entries[index + 1..].iter().any(|candidate| {
        candidate.parts.len() > path.len().saturating_sub(1)
            && candidate.parts[..path.len().saturating_sub(1)]
                == path[..path.len().saturating_sub(1)]
            && candidate.parts.get(path.len().saturating_sub(1)) != path.last()
    })
}
