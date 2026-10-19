//! Renders repository, file, and symbol topology.

mod directory;
mod display;
mod entries;
use directory::run_directory;
use display::{show_file, show_symbol};
use std::path::Path;

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
