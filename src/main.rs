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

/// Every command form, printed by `--help` and with a usage error.
const USAGE: &str = "usage: topo [<path> | <file>::<symbol>]
       topo find <name> [<dir>]
       topo score [--detail] [--guide] [<path>...]
       topo extract [<path>]
       topo --help | --version";

/// Runs one command and exits 0 on success, 1 when nothing matched and 2 on an error.
fn main() {
    // SAFETY: restores the default action (SIG_DFL) for SIGPIPE (13) before any output,
    // so a closed pipe such as `topo extract . | head` ends the process quietly.
    #[cfg(unix)]
    unsafe {
        signal(13, 0);
    }
    let status = match run(std::env::args().skip(1).collect()) {
        Ok(found) => i32::from(!found),
        Err(error) => {
            eprintln!("error: {error}");
            2
        }
    };
    std::process::exit(status);
}

/// Routes arguments to a command, returning false when it found nothing.
fn run(args: Vec<String>) -> Result<bool, String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        _ if args.contains(&"--help") || args.contains(&"-h") => println!("{USAGE}"),
        ["--version" | "-V"] => println!("topo {}", env!("CARGO_PKG_VERSION")),
        ["find", name] | ["find", name, _] => return tree::find(name, args.get(2).unwrap_or(&".")),
        ["score", rest @ ..] => score(rest)?,
        ["extract"] => extract::run(".")?,
        ["extract", target] => extract::run(target)?,
        [] => return tree::inspect("."),
        [target] if !target.starts_with('-') && *target != "find" => return tree::inspect(target),
        _ => return Err(USAGE.to_owned()),
    }
    Ok(true)
}

/// Scores the given paths (the working directory by default), skipping unreadable files.
fn score(args: &[&str]) -> Result<(), String> {
    let mut options = Options::default();
    let mut paths = Vec::new();
    for arg in args {
        match *arg {
            "--detail" => options.detail = true,
            "--guide" => options.guide = true,
            flag if flag.starts_with('-') => return Err(format!("unknown option: {flag}")),
            path => paths.push(path),
        }
    }
    if paths.is_empty() {
        paths.push(".");
    }
    let analysed: Vec<_> = paths::analyse(&paths, Path::new("."))?.collect();
    if analysed.is_empty() {
        return Err("no supported source files could be analysed".to_owned());
    }
    report::print_table(&analysed, &options);
    if options.guide {
        report::print_guide(options.colour);
    }
    Ok(())
}
