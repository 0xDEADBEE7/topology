//! Dispatches progressive inspection, scoring, and extraction commands.

mod extract;
mod lang;
mod metrics;
mod paths;
mod report;
mod syntax;
mod table;
mod tree;
mod walk;

use report::Options;
use std::path::Path;

#[cfg(unix)]
extern "C" {
    fn signal(signum: i32, handler: usize) -> usize;
}

/// Reports command failures with a nonzero exit status.
fn main() {
    // SAFETY: restores the default action (SIG_DFL) for SIGPIPE (13) before any output,
    // so a closed pipe such as `topo extract . | head` ends the process quietly.
    #[cfg(unix)]
    unsafe {
        signal(13, 0);
    }
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

/// Routes arguments to the requested inspection workflow.
fn run(args: Vec<String>) -> Result<(), String> {
    let full_docstring = args.iter().any(|arg| arg == "--full-docstring");
    let args: Vec<_> = args
        .into_iter()
        .filter(|arg| arg != "--full-docstring")
        .collect();
    let target = args.get(1).map(String::as_str).unwrap_or(".");
    match args.first().map(String::as_str) {
        Some("extract") => extract::run(target),
        Some("find") => tree::find(args.get(1).ok_or_else(usage)?),
        Some("score") => score(&args[1..]),
        Some("tree") => tree::inspect_with_options(target, full_docstring),
        Some(target) => tree::inspect_with_options(target, full_docstring),
        None => Err(usage()),
    }
}

/// Describes the available command forms.
fn usage() -> String {
    "usage: topo [--full-docstring] <path|path::symbol>\n       topo find <symbol>\n       topo score [--detail] [--guide] [--no-colour] <path> [...]\n       topo extract <dir>".to_owned()
}

/// Scores supported project sources, skipping files that cannot be read.
fn score(args: &[String]) -> Result<(), String> {
    let (options, inputs) = parse_score_args(args)?;
    let analysed: Vec<_> = paths::analyse(&inputs, Path::new(""))?.collect();
    if analysed.is_empty() {
        return Err("no supported source files could be analysed".to_owned());
    }
    report::print_table(&analysed, &options);
    if options.guide {
        report::print_guide(options.colour);
    }
    Ok(())
}

/// Separates scoring options from input paths.
fn parse_score_args(args: &[String]) -> Result<(Options, Vec<String>), String> {
    let mut options = Options::default();
    let mut paths = Vec::new();
    for arg in args {
        if !flag(arg, &mut options)? {
            paths.push(arg.clone());
        }
    }
    if paths.is_empty() {
        return Err(usage());
    }
    Ok((options, paths))
}

/// Applies one score flag, returning false for a path.
fn flag(arg: &str, options: &mut Options) -> Result<bool, String> {
    match arg {
        "--detail" => options.detail = true,
        "--guide" => options.guide = true,
        "--no-colour" | "--no-color" => options.colour = false,
        "--help" | "-h" => return Err(usage()),
        value if value.starts_with('-') => return Err(format!("unknown option: {value}")),
        _ => return Ok(false),
    }
    Ok(true)
}
