//! Coordinates CLI scoring and provides command usage.

mod options;
use crate::{adapters, paths, report};
use std::{fs, path::Path};

/// Describes the available command forms.
pub fn usage() -> String {
    "usage: topo [--full-docstring] <path|path::symbol>\n       topo find <symbol>\n       topo score [--guide] [--detail] [--metrics LIST] [--no-colour] <file> [...]\n       topo extract <dir>".to_owned()
}

/// Scores supported project sources, skipping files that cannot be read or parsed.
pub fn score(args: &[String]) -> Result<(), String> {
    let (options, inputs) = options::parse(args)?;
    let mut files = Vec::new();
    for input in inputs {
        paths::collect(Path::new(&input), &mut files)?;
    }
    files.sort();
    files.dedup();
    let analysed: Vec<_> = files
        .iter()
        .filter_map(|path| {
            let path = path.to_string_lossy();
            fs::read_to_string(path.as_ref())
                .map_err(|e| e.to_string())
                .and_then(|source| adapters::analyse(&path, &source))
                .map_err(|e| eprintln!("warning: {path}: skipped: {e}"))
                .ok()
        })
        .collect();
    if analysed.is_empty() {
        return Err("no supported source files could be analysed".to_owned());
    }
    report::print_table(&analysed, &options);
    if options.guide {
        report::print_guide(options.colour);
    }
    Ok(())
}
