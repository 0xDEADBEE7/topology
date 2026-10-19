//! Discovers supported source files while excluding dependencies and build output.

use std::{
    fs,
    path::{Path, PathBuf},
};

/// Collects source files deterministically without following symbolic links.
pub fn collect(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = fs::symlink_metadata(root).map_err(|e| format!("{}: {e}", root.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        if crate::adapters::for_path(&root.to_string_lossy()).is_ok() {
            paths.push(root.to_owned());
        }
        return Ok(());
    }
    let mut children = fs::read_dir(root)
        .map_err(|e| format!("{}: {e}", root.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    children.sort();
    for path in children {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if excluded(name) {
            continue;
        }
        if let Err(error) = collect(&path, paths) {
            eprintln!("warning: {error}; skipped");
        }
    }
    Ok(())
}

/// Keeps generated and third-party code out of repository assessments.
fn excluded(name: &str) -> bool {
    name.starts_with('.')
        || matches!(
            name,
            "target" | "node_modules" | "vendor" | "dist" | "build" | "bin" | "coverage"
        )
}
