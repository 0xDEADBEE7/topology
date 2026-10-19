//! Renders repository, file, and symbol topology, and finds declarations by name.

use crate::metrics::{FileInfo, Unit};
use std::{cmp::Reverse, collections::BTreeMap, fs, path::Path};

/// Displays repository, file, or symbol detail according to the target path, returning
/// false when a symbol is missing or ambiguous.
pub fn inspect(target: &str) -> Result<bool, String> {
    if let Some((path, symbol)) = target.split_once("::") {
        return show_symbol(path, symbol);
    }
    match Path::new(target).is_file() {
        true => show_file(target)?,
        false => run_directory(target)?,
    }
    Ok(true)
}

/// Displays a repository tree with available descriptions.
fn run_directory(root: &str) -> Result<(), String> {
    let canonical = fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
    let files = crate::paths::analyse(&[&canonical], &canonical)?;
    let files = files.map(|file| (file.path, file.doc)).collect();
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
    print_entry(&format!("{root_name}/"), root_doc, width);
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
        print_entry(&prefix, description.clone(), width);
    }
    Ok(())
}

/// Reads a directory's description: the first line of its `.docstring` file.
fn read_docstring(directory: &Path) -> Option<String> {
    let content = fs::read_to_string(directory.join(".docstring")).ok()?;
    let line = content.lines().next()?.trim();
    (!line.is_empty()).then(|| line.to_owned())
}

/// Displays a file description and its declaration outline.
fn show_file(path: &str) -> Result<(), String> {
    let (_, file) = read(path)?;
    let mut units: Vec<_> = file.units().collect();
    units.sort_by_key(|(u, kind)| (u.line_start, Reverse(u.line_end), *kind == "function"));
    let lines = |u: &Unit| (u.line_start, u.line_end);
    let rows: Vec<_> = (units.iter())
        .map(|(s, _)| {
            let contains =
                |(o, _): &&(&Unit, _)| o.line_start <= s.line_start && s.line_end <= o.line_end;
            let containers = units.iter().filter(contains);
            let mut ranges: Vec<_> = containers.map(|(o, _)| lines(o)).collect();
            ranges.dedup();
            let depth = ranges.len();
            let indent = "  ".repeat(depth);
            let row = format!("{indent}{} (L{}–{})", s.name, s.line_start, s.line_end);
            (row, s.doc.clone())
        })
        .collect();
    let width = align_width(rows.iter().map(|(prefix, _)| prefix.chars().count()));
    print_entry(path, file.doc, width);
    for (prefix, description) in rows {
        print_entry(&prefix, description, width);
    }
    Ok(())
}

/// Picks the description column, ignoring entries too long to align with the rest.
fn align_width(widths: impl Iterator<Item = usize>) -> usize {
    widths.filter(|width| *width <= 40).max().unwrap_or(0) + 2
}

/// Displays the source of every declaration named exactly `name` in the named file only,
/// or of the one whose last segment it is; lists the candidates when several or none match.
fn show_symbol(path: &str, name: &str) -> Result<bool, String> {
    let (source, file) = read(path)?;
    let named: Vec<_> = (file.units().map(|(unit, _)| unit))
        .filter(|unit| is_named(&unit.name, name))
        .collect();
    let exact = named.iter().any(|unit| unit.name == name);
    let shown: Vec<_> = named
        .iter()
        .filter(|unit| !exact || unit.name == name)
        .collect();
    if !exact && shown.len() != 1 {
        println!("{} matches for '{path}::{name}':", shown.len());
        for unit in shown {
            println!(
                "{path}::{} (L{}–{})",
                unit.name, unit.line_start, unit.line_end
            );
        }
        return Ok(false);
    }
    for unit in shown {
        let (start, end) = (unit.line_start, unit.line_end);
        println!("{path}::{} (L{start}–{end})", unit.name);
        for line in source.lines().skip(start - 1).take(end + 1 - start) {
            println!("{line}");
        }
    }
    Ok(true)
}

/// Reads and analyses one named file.
fn read(path: &str) -> Result<(String, FileInfo), String> {
    let source = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let file = crate::walk::analyse(path, &source).map_err(|e| format!("{path}: {e}"))?;
    Ok((source, file))
}

/// Prints the declarations under `directory` that `query` names, returning false when none
/// does.
pub fn find(query: &str, directory: &str) -> Result<bool, String> {
    let mut found = Vec::new();
    for file in crate::paths::analyse(&[directory], Path::new("."))? {
        let named = |(unit, _): &(&Unit, _)| matches(&file.path, &unit.name, query);
        let mut units: Vec<_> = file.units().filter(named).collect();
        units.sort_by_key(|(unit, _)| unit.line_start);
        found.extend(units.iter().map(|(unit, kind)| {
            let (start, end) = (unit.line_start, unit.line_end);
            format!("{}::{} [{kind}] (L{start}–{end})", file.path, unit.name)
        }));
    }
    println!("{} matches for '{query}':", found.len());
    for row in &found {
        println!("{row}");
    }
    Ok(!found.is_empty())
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

/// Fits a description beside a tree entry, cutting it to the line.
fn print_entry(prefix: &str, description: Option<String>, target_width: usize) {
    let Some(description) = description else {
        return println!("{prefix}");
    };
    let width = prefix.chars().count();
    let separator = "─".repeat(target_width.saturating_sub(width).max(2));
    let available = 78usize.saturating_sub(width + separator.chars().count() + 4);
    let description = truncate_right(&description, available.max(24));
    println!("{prefix}  #{separator} {description}");
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
    let later = |depth| has_later_sibling(index, &parts[..depth], entries);
    let rails = (1..parts.len()).map(|depth| if later(depth) { "│  " } else { "   " });
    let branch = if later(parts.len()) {
        "├─ "
    } else {
        "└─ "
    };
    rails.chain([branch]).collect()
}

fn has_later_sibling(index: usize, path: &[String], entries: &[Entry]) -> bool {
    let (name, parent) = path.split_last().expect("paths have a final component");
    entries[index + 1..].iter().any(|(parts, _)| {
        parts.starts_with(parent) && parts.get(parent.len()).is_some_and(|next| next != name)
    })
}
