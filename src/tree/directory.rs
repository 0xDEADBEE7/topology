//! Reads directory descriptions and renders repository trees.

use super::{
    display::print_entry,
    entries::{render_prefix, tree_entries},
};
use std::{fs, path::Path};

/// Displays a repository tree with available descriptions.
pub(super) fn run_directory(root: &str, full_docstring: bool) -> Result<(), String> {
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

/// Reads the description attached to a containing directory.
pub(super) fn directory_doc(root: &str, parts: &[&str], index: usize) -> Option<String> {
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
