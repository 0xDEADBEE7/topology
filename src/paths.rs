//! Discovers and analyses supported source files, excluding dependencies and build output.

use crate::{lang::Language, metrics::FileInfo};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Analyses every supported file under `targets` as it is iterated, once each however it
/// was spelt (the first spelling wins), naming each relative to `root` and skipping, with a
/// warning, any that cannot be read or is not supported.
pub fn analyse<'a>(
    targets: &[impl AsRef<Path>],
    root: &'a Path,
) -> Result<impl Iterator<Item = FileInfo> + 'a, String> {
    let mut paths = Vec::new();
    for target in targets.iter().map(AsRef::as_ref) {
        let metadata = fs::metadata(target).map_err(|e| format!("{}: {e}", target.display()))?;
        match metadata.is_file() {
            true => paths.push(target.to_owned()),
            false => collect(target, &mut paths)?,
        }
    }
    let canonical = |path: &PathBuf| fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    let mut paths: Vec<_> = paths.into_iter().map(|p| (canonical(&p), p)).collect();
    paths.sort_by(|a, b| a.0.cmp(&b.0));
    paths.dedup_by(|a, b| a.0 == b.0);
    let files = paths.into_iter().filter_map(move |(_, path)| {
        let name = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
        fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|source| crate::walk::analyse(&name, &source))
            .map_err(|e| eprintln!("warning: {name}: skipped: {e}"))
            .ok()
    });
    Ok(files)
}

/// Collects a directory's supported files in name order, skipping hidden entries, symbolic
/// links, excluded directories, TypeScript declarations and minified scripts.
fn collect(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut children = fs::read_dir(directory)
        .map_err(|e| format!("{}: {e}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    children.sort();
    for path in children {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let warn = |e| eprintln!("warning: {}: {e}; skipped", path.display());
        let metadata = fs::symlink_metadata(&path).map_err(warn).ok();
        let visible = metadata.filter(|_| !name.starts_with('.'));
        let Some(kind) = visible.map(|metadata| metadata.file_type()) else {
            continue;
        };
        if kind.is_dir() && !excluded(name) {
            if let Err(error) = collect(&path, paths) {
                eprintln!("warning: {error}; skipped");
            }
        } else if kind.is_file() && Language::from_path(&path).is_some() && !generated(name) {
            paths.push(path);
        }
    }
    Ok(())
}

/// Recognizes TypeScript declarations and minified scripts, which describe or repeat code
/// written elsewhere.
fn generated(name: &str) -> bool {
    name.ends_with(".d.ts") || name.ends_with(".min.js")
}

/// Keeps generated, installed and third-party code out of directory scans.
fn excluded(name: &str) -> bool {
    matches!(
        name,
        "target"
            | "node_modules"
            | "vendor"
            | "dist"
            | "coverage"
            | "venv"
            | "__pycache__"
            | "site-packages"
    )
}
