mod metrics;
mod report;
mod table;
mod visit;
mod visitor;

use std::{env, fs};

fn analyse(path: &str) -> Option<metrics::FileMetrics> {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("error reading {path}: {e}"); return None; }
    };
    let file = match syn::parse_file(&src) {
        Ok(f) => f,
        Err(e) => { eprintln!("parse error in {path}: {e}"); return None; }
    };
    let raw  = visit::collect(&file);
    let fns: Vec<_> = raw.iter().map(metrics::compute).collect();
    Some(metrics::aggregate(path, &raw, fns, src.lines().count()))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: assay <file.rs> [...]");
        std::process::exit(1);
    }
    let files: Vec<_> = args.iter().filter_map(|p| analyse(p)).collect();
    report::print_table(&files);
}
