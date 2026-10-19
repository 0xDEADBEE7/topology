//! Discovers and analyses supported source files, excluding dependencies and build output.

use crate::metrics::FileInfo;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Analyses every supported file under `targets`, naming each relative to `root` and
/// skipping, with a warning, any that cannot be read.
pub fn analyse(targets: &[impl AsRef<Path>], root: &Path) -> Result<Vec<FileInfo>, String> {
    let mut paths = Vec::new();
    for target in targets {
        collect(target.as_ref(), &mut paths)?;
    }
    paths.sort();
    paths.dedup();
    let files = paths.iter().filter_map(|path| {
        let name = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
        fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|source| crate::walk::analyse(&name, &source))
            .map_err(|e| eprintln!("warning: {name}: skipped: {e}"))
            .ok()
    });
    Ok(files.collect())
}

/// Collects source files deterministically without following symbolic links.
fn collect(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = fs::symlink_metadata(root).map_err(|e| format!("{}: {e}", root.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        if crate::lang::Language::from_path(root).is_some() {
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
