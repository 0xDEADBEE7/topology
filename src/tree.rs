//! Renders repository, file, and symbol topology.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Debug)]
struct Entry {
    parts: Vec<String>,
    description: Option<String>,
    directory: bool,
}

pub fn inspect_with_options(target: &str, full_docstring: bool) -> Result<(), String> {
    if let Some((path, symbol)) = target.split_once("::") {
        return show_symbol(path, symbol);
    }
    if Path::new(target).is_file() {
        return show_file(target, full_docstring);
    }
    run_directory(target, full_docstring)
}

fn tree_entries(root: &str, files: Vec<(String, Option<String>)>) -> Vec<Entry> {
    let mut descriptions = BTreeMap::new();
    let mut paths = BTreeSet::new();
    for (path, description) in files {
        let parts: Vec<String> = path.split('/').map(str::to_owned).collect();
        for index in 0..parts.len() {
            let path = parts[..=index].join("/");
            paths.insert(path.clone());
            if index + 1 == parts.len() {
                descriptions.insert(path, description.clone());
            } else {
                descriptions.entry(path).or_insert_with(|| {
                    directory_doc(
                        root,
                        &parts.iter().map(String::as_str).collect::<Vec<_>>(),
                        index,
                    )
                });
            }
        }
    }
    let path_list: Vec<_> = paths.iter().cloned().collect();
    paths
        .into_iter()
        .map(|path| Entry {
            directory: path_list.iter().any(|candidate| {
                candidate != &path && candidate.starts_with(&(path.clone() + "/"))
            }),
            parts: path.split('/').map(str::to_owned).collect(),
            description: descriptions.remove(&path).flatten(),
        })
        .collect()
}

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

fn run_directory(root: &str, full_docstring: bool) -> Result<(), String> {
    let files = crate::extract::file_descriptions(root)?;
    let root_name = Path::new(root)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(root);
    let entries = tree_entries(root, files);
    let max_prefix = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            render_prefix(index, entry, &entries).chars().count()
                + entry.parts.last().map_or(0, String::len)
        })
        .max()
        .unwrap_or_else(|| root_name.chars().count() + 1);
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

fn directory_doc(root: &str, parts: &[&str], index: usize) -> Option<String> {
    let path = Path::new(root).join(parts[..=index].join("/"));
    read_docstring(&path)
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

fn show_file(path: &str, full_docstring: bool) -> Result<(), String> {
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
