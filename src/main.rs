mod adapters;
mod metrics;
mod report;
mod table;
mod visit;
mod visitor;

use std::{env, fs};

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

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: assay <file.rs|file.py|file.ts> [...]");
        std::process::exit(1);
    }
    let files: Vec<_> = args.iter().filter_map(|p| analyse(p)).collect();
    report::print_table(&files);
}
