//! Command-line entry point for per-file metrics and repository extraction.

mod adapters;
mod extract;
mod metrics;
mod report;
mod table;
mod tree;
mod visit;
mod visitor;

use std::{env, fs};

fn collect_paths(path: &str, files: &mut Vec<String>) {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => {
            eprintln!("error reading {path}: {error}");
            return;
        }
    };
    if !metadata.is_dir() {
        files.push(path.to_owned());
        return;
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("error reading {path}: {error}");
            return;
        }
    };
    let mut children: Vec<_> = entries.filter_map(Result::ok).collect();
    children.sort_by_key(|entry| entry.path());
    for entry in children {
        let child = entry.path();
        collect_paths(&child.to_string_lossy(), files);
    }
}

fn analyse(path: &str) -> Option<metrics::FileMetrics> {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {path}: {e}");
            return None;
        }
    };
    let adapter = match adapters::for_path(path) {
        Ok(adapter) => adapter,
        Err(e) => {
            eprintln!("{path}: {e}");
            return None;
        }
    };
    match adapter.analyse(path, &src) {
        Ok(metrics) => Some(metrics),
        Err(e) => {
            eprintln!("{path}: {e}");
            None
        }
    }
}

fn usage() -> ! {
    eprintln!("usage: topo <path|path::symbol>");
    eprintln!("       topo find <symbol>");
    eprintln!(
        "       topo score [--guide] [--detail] [--metrics LIST] [--no-colour] <file> [...]"
    );
    eprintln!("       topo extract <dir>");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("extract") => {
            let root = args.get(1).cloned().unwrap_or_else(|| ".".to_owned());
            if let Err(error) = extract::run(&root) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
            return;
        }
        Some("find") => {
            let query = args.get(1).map(String::as_str).unwrap_or_else(|| usage());
            match extract::find(".", query) {
                Ok(matches) => {
                    println!("{} matches for '{query}':", matches.len());
                    for symbol in matches {
                        println!(
                            "{}::{} [{}] (L{}–{})",
                            symbol.path, symbol.name, symbol.kind, symbol.lines[0], symbol.lines[1]
                        );
                    }
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("score") => score(&args[1..]),
        Some("tree") => {
            let target = args.get(1).map(String::as_str).unwrap_or(".");
            if let Err(error) = tree::inspect(target) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
            return;
        }
        _ => {
            let target = args.first().map(String::as_str).unwrap_or_else(|| usage());
            if let Err(error) = tree::inspect(target) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
            return;
        }
    }
}

fn score(args: &[String]) {
    let mut options = report::Options::default();
    let mut files = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        index += 1;
        match arg.as_str() {
            "--detail" | "--functions" => options.detail = true,
            "--guide" => options.guide = true,
            "--all-metrics" => {
                options.metrics = vec![
                    report::Metric::Nmi,
                    report::Metric::Loc,
                    report::Metric::Docstring,
                    report::Metric::Cc,
                    report::Metric::Cognitive,
                    report::Metric::Halstead,
                ]
            }
            "--no-colour" | "--no-color" => options.colour = false,
            "--metrics" => {
                let value = args.get(index).cloned().unwrap_or_default();
                index += 1;
                options.metrics = report::parse_metrics(&value).unwrap_or_else(|e| {
                    eprintln!("{e}");
                    usage()
                });
            }
            "--help" | "-h" => usage(),
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                usage();
            }
            path => files.push(path.to_owned()),
        }
    }
    if files.is_empty() {
        usage();
    }
    let mut input_files = Vec::new();
    for path in files {
        collect_paths(&path, &mut input_files);
    }
    let analysed: Vec<_> = input_files.iter().filter_map(|p| analyse(p)).collect();
    report::print_table(&analysed, &options);
    if options.guide {
        report::print_guide(options.colour);
    }
}
