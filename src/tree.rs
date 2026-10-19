//! Renders repository, file, and symbol topology, and finds declarations by name.

use crate::metrics::{FileInfo, Unit};
use std::{cmp::Reverse, collections::BTreeMap, fs, path::Path};

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
    let canonical = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let files = crate::paths::analyse(&[&canonical], &canonical)?;
    let files = files
        .into_iter()
        .map(|file| (file.path, file.doc))
        .collect();
    let root_name = Path::new(root)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(root);
    let entries = tree_entries(root, files);
    let width = align_width(entries.iter().enumerate().map(|(index, (parts, _))| {
        render_prefix(index, parts, &entries).chars().count()
            + parts.last().map_or(0, |name| name.chars().count())
    }));
    let root_doc = read_docstring(Path::new(root));
    print_entry(&format!("{root_name}/"), root_doc, width, full_docstring);
    for (index, (parts, description)) in entries.iter().enumerate() {
        let directory = entries
            .get(index + 1)
            .is_some_and(|(next, _)| next.starts_with(parts));
        let prefix = format!(
            "{}{}{}",
            render_prefix(index, parts, &entries),
            parts.last().map_or("", String::as_str),
            if directory { "/" } else { "" }
        );
        print_entry(&prefix, description.clone(), width, full_docstring);
    }
    Ok(())
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
    let (_, file) = read(path)?;
    let mut units: Vec<_> = file.units().collect();
    units.sort_by_key(|(u, kind)| (u.line_start, Reverse(u.line_end), *kind == "function"));
    let lines = |u: &Unit| (u.line_start, u.line_end);
    let rows: Vec<_> = (units.iter())
        .map(|(s, _)| {
            let contains =
                |(o, _): &&(&Unit, _)| o.line_start <= s.line_start && s.line_end <= o.line_end;
            let containers = units.iter().filter(contains);
            let depth = 1 + containers.filter(|(o, _)| lines(o) != lines(s)).count();
            let indent = "  ".repeat(depth);
            let row = format!("{indent}{} (L{}–{})", s.name, s.line_start, s.line_end);
            (row, s.doc.clone())
        })
        .collect();
    let width = align_width(rows.iter().map(|(prefix, _)| prefix.chars().count()));
    print_entry(path, file.doc, width, full_docstring);
    for (prefix, description) in rows {
        print_entry(&prefix, description, width, full_docstring);
    }
    Ok(())
}

/// Picks the description column, ignoring entries too long to align with the rest.
fn align_width(widths: impl Iterator<Item = usize>) -> usize {
    widths.filter(|width| *width <= 40).max().unwrap_or(0) + 2
}

/// Displays the source of a declaration found in the named file only.
fn show_symbol(path: &str, name: &str) -> Result<(), String> {
    let (source, file) = read(path)?;
    let (start, end) = (file.units().map(|(unit, _)| unit))
        .filter(|unit| is_named(&unit.name, name))
        .min_by_key(|unit| unit.name != name)
        .map(|unit| (unit.line_start, unit.line_end))
        .ok_or_else(|| format!("symbol not found: {path}::{name}"))?;
    println!("{path}::{name} (L{start}–{end})");
    for line in source
        .lines()
        .skip(start.saturating_sub(1))
        .take(end + 1 - start)
    {
        println!("{line}");
    }
    Ok(())
}

/// Reads and analyses one named file.
fn read(path: &str) -> Result<(String, FileInfo), String> {
    let source = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let file = crate::walk::analyse(path, &source).map_err(|e| format!("{path}: {e}"))?;
    Ok((source, file))
}

/// Prints the declarations under the working directory that `query` names.
pub fn find(query: &str) -> Result<(), String> {
    let mut found = Vec::new();
    for file in crate::paths::analyse(&["."], Path::new("."))? {
        let named = |(unit, _): &(&Unit, _)| matches(&file.path, &unit.name, query);
        let mut units: Vec<_> = file.units().filter(named).collect();
        units.sort_by_key(|(unit, _)| unit.line_start);
        found.extend(units.iter().map(|(unit, kind)| {
            let (start, end) = (unit.line_start, unit.line_end);
            format!("{}::{} [{kind}] (L{start}–{end})", file.path, unit.name)
        }));
    }
    println!("{} matches for '{query}':", found.len());
    for row in found {
        println!("{row}");
    }
    Ok(())
}

/// Matches a declaration by name, or by `path::name` with or without the extension.
fn matches(path: &str, name: &str, query: &str) -> bool {
    let stem = path.rsplit_once('.').map_or(path, |(stem, _)| stem);
    is_named(name, query)
        || [path, stem]
            .iter()
            .any(|path| query == format!("{path}::{name}"))
}

/// Matches a symbol by its full name or by the last segment of `View.render` or `Type::method`.
fn is_named(symbol: &str, query: &str) -> bool {
    symbol == query || symbol.rsplit(['.', ':']).next() == Some(query)
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
    if text.chars().count() <= width {
        return text.to_owned();
    }
    text.chars().take(width - 3).collect::<String>() + "..."
}

/// Path components and description of one file or directory.
type Entry = (Vec<String>, Option<String>);

/// Builds one entry per source file and containing directory, ordered by path components.
fn tree_entries(root: &str, files: Vec<(String, Option<String>)>) -> Vec<Entry> {
    let mut descriptions = BTreeMap::new();
    for (path, description) in files {
        let parts: Vec<String> = path.split('/').map(str::to_owned).collect();
        for index in 1..parts.len() {
            let directory = &parts[..index];
            descriptions
                .entry(directory.to_vec())
                .or_insert_with(|| read_docstring(&Path::new(root).join(directory.join("/"))));
        }
        descriptions.insert(parts, description);
    }
    descriptions.into_iter().collect()
}

/// Renders branch connectors based on later siblings.
fn render_prefix(index: usize, parts: &[String], entries: &[Entry]) -> String {
    let mut prefix = String::new();
    for depth in 1..parts.len() {
        prefix.push_str(if has_later_sibling(index, &parts[..depth], entries) {
            "│  "
        } else {
            "   "
        });
    }
    prefix.push_str(if has_later_sibling(index, parts, entries) {
        "├─ "
    } else {
        "└─ "
    });
    prefix
}

fn has_later_sibling(index: usize, path: &[String], entries: &[Entry]) -> bool {
    let (name, parent) = path.split_last().expect("paths have a final component");
    entries[index + 1..].iter().any(|(parts, _)| {
        parts.starts_with(parent) && parts.get(parent.len()).is_some_and(|next| next != name)
    })
}
