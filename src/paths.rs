//! Discovers and analyses supported source files, excluding dependencies and build output.

use crate::{lang::Language, walk::FileInfo};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Folders of build output and of installed or vendored dependencies, which scans skip.
const BUILT: [&str; 4] = ["target", "dist", "coverage", "__pycache__"];
const INSTALLED: [&str; 4] = ["node_modules", "vendor", "venv", "site-packages"];

/// Folders and file name parts that mark tests, test helpers and stories.
const TESTS: [&str; 5] = ["test", "tests", "__tests__", "__mocks__", "e2e"];
const TEST_NAMES: [&str; 4] = [".test.", ".spec.", ".stories.", ".story."];

/// Analyses every supported file under `targets` as it is iterated, once each however it
/// was spelt (the first spelling wins), naming each relative to `root` and skipping, with a
/// warning, any that cannot be read or is not supported. Scans leave out tests unless
/// `tests` is set.
pub fn analyse<'a>(
    targets: &[impl AsRef<Path>],
    root: &'a Path,
    tests: bool,
) -> Result<impl Iterator<Item = FileInfo> + 'a, String> {
    let mut paths = Vec::new();
    for target in targets.iter().map(AsRef::as_ref) {
        let metadata = fs::metadata(target).map_err(|e| format!("{}: {e}", target.display()))?;
        match metadata.is_file() {
            true => paths.push(target.to_owned()),
            false => paths.extend(scan(target, tests)?),
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

/// Lists the sources in a directory: the files git tracks or would track there, or every
/// file a walk finds outside a repository, without hidden or excluded folders, TypeScript
/// declarations, minified scripts and, unless `tests`, tests.
fn scan(directory: &Path, tests: bool) -> Result<Vec<PathBuf>, String> {
    let mut files = git_files(directory);
    if files.is_empty() {
        walk(directory, &mut files)?;
    }
    let source = |file: &PathBuf| {
        let relative = file.strip_prefix(directory).unwrap_or(file);
        let names: Vec<_> = relative.iter().filter_map(|name| name.to_str()).collect();
        let (name, folders) = names.split_last()?;
        let generated = name.ends_with(".d.ts") || name.ends_with(".min.js");
        let hidden = names.iter().any(|name| name.starts_with('.'));
        let excluded = folders.iter().any(|folder| excluded(folder));
        let test = folders.iter().any(|folder| TESTS.contains(folder))
            || TEST_NAMES.iter().any(|part| name.contains(part))
            || name.starts_with("test_")
            || name.ends_with("_test.py")
            || *name == "conftest.py";
        Some(!(generated || hidden || excluded || test && !tests))
    };
    files.retain(|file| Language::from_path(file).is_some() && source(file) == Some(true));
    Ok(files)
}

/// Lists the regular files git tracks or would track under `directory`; none outside a
/// repository or without git. The repository's file monitor hook is not run.
fn git_files(directory: &Path) -> Vec<PathBuf> {
    let mut git = Command::new("git");
    git.args(["-c", "core.fsmonitor=0", "-C"]).arg(directory);
    git.args(["ls-files", "-co", "--exclude-standard", "-z"]);
    let listed = git.output().map(|output| output.stdout).unwrap_or_default();
    let names = listed.split(|byte| *byte == 0);
    let files = names.map(|name| directory.join(String::from_utf8_lossy(name).as_ref()));
    let regular = |file: &PathBuf| fs::symlink_metadata(file).is_ok_and(|m| m.is_file());
    files.filter(regular).collect()
}

/// Collects the regular files under a directory, skipping symbolic links and hidden or
/// excluded folders.
fn walk(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let children = fs::read_dir(directory)
        .map_err(|e| format!("{}: {e}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for path in children {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let warn = |e| eprintln!("warning: {}: {e}; skipped", path.display());
        let Ok(kind) = fs::symlink_metadata(&path)
            .map(|m| m.file_type())
            .map_err(warn)
        else {
            continue;
        };
        if kind.is_dir() && !name.starts_with('.') && !excluded(name) {
            if let Err(error) = walk(&path, files) {
                eprintln!("warning: {error}; skipped");
            }
        } else if kind.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

/// Recognizes build output and installed or vendored dependencies by folder name.
fn excluded(folder: &str) -> bool {
    BUILT.contains(&folder) || INSTALLED.contains(&folder)
}
