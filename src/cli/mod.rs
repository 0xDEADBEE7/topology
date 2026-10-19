//! Coordinates CLI scoring and provides command usage.

mod options;
use crate::{adapters, paths, report};
use std::{fs, path::Path};

/// Describes the available command forms.
pub fn usage() -> String {
    "usage: topo [--full-docstring] <path|path::symbol>\n       topo find <symbol>\n       topo score [--guide] [--detail] [--metrics LIST] [--no-colour] <file> [...]\n       topo extract <dir>".to_owned()
}

/// Scores supported project sources and propagates read or parse failures.
pub fn score(args: &[String]) -> Result<(), String> {
    let (options, inputs) = options::parse(args)?;
    let mut files = Vec::new();
    for input in inputs {
        paths::collect(Path::new(&input), &mut files)?;
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        return Err("no supported source files found".to_owned());
    }
    let analysed = files
        .iter()
        .map(|path| {
            let path = path.to_string_lossy();
            let source = fs::read_to_string(path.as_ref()).map_err(|e| format!("{path}: {e}"))?;
            adapters::for_path(&path)?
                .analyse(&path, &source)
                .map_err(|e| format!("{path}: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    report::print_table(&analysed, &options);
    if options.guide {
        report::print_guide(options.colour);
    }
    Ok(())
}
