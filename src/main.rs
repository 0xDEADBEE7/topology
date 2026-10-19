//! Dispatches progressive inspection, scoring, and extraction commands.

mod adapters;
mod cli;
mod extract;
mod metrics;
mod paths;
mod report;
mod table;
mod tree;
mod visit;
mod visitor;

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
        Some("find") => find(args.get(1).ok_or_else(cli::usage)?),
        Some("score") => cli::score(&args[1..]),
        Some("tree") => tree::inspect_with_options(target, full_docstring),
        Some(target) => tree::inspect_with_options(target, full_docstring),
        None => Err(cli::usage()),
    }
}

/// Prints matching declarations with their source locations.
fn find(query: &str) -> Result<(), String> {
    let symbols = extract::find(".", query)?;
    println!("{} matches for '{query}':", symbols.len());
    for symbol in symbols {
        println!(
            "{}::{} [{}] (L{}–{})",
            symbol.path, symbol.name, symbol.kind, symbol.lines[0], symbol.lines[1]
        );
    }
    Ok(())
}
