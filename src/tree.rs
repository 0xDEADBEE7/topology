//! Renders an extracted file tree with aligned source descriptions.

use std::collections::BTreeSet;

/// Render the supported files beneath `root` as an indented tree.
pub fn run(root: &str) -> Result<(), String> {
    let files = crate::extract::file_descriptions(root)?;
    let root_name = std::path::Path::new(root)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(root)
        .to_owned();
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
            let directory = parts[..=index].join("/");
            if seen.insert(directory) {
                println!("{}{}{}/", "  ".repeat(index + 1), parts[index], "");
            }
        }
        let name = parts.last().copied().unwrap_or_default();
        let prefix = format!("{}{}", "  ".repeat(depth + 1), name);
        match description {
            Some(description) => println!(
                "{prefix} #{} {description}",
                "─".repeat(max_prefix.saturating_sub(prefix.len()).max(2))
            ),
            None => println!("{prefix}"),
        }
    }
    Ok(())
}
