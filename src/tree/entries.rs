//! Builds ordered tree entries and branch prefixes.

use super::directory::directory_doc;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(super) struct Entry {
    pub(super) parts: Vec<String>,
    pub(super) description: Option<String>,
    pub(super) directory: bool,
}

/// Builds one sorted entry per source file and containing directory.
pub(super) fn tree_entries(root: &str, files: Vec<(String, Option<String>)>) -> Vec<Entry> {
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

/// Renders branch connectors based on later siblings.
pub(super) fn render_prefix(index: usize, entry: &Entry, entries: &[Entry]) -> String {
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
