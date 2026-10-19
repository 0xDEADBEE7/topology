//! Rates functions against the metric bands and prints score tables and the guide.

use crate::{
    metrics::{FileMetrics, FnMetrics},
    table::{self, GREEN, RED, YELLOW},
};
use std::io::IsTerminal;

/// Rated function metrics: name, largest green value, and largest yellow value.
pub const BANDS: [(&str, usize, usize); 2] = [("Lines", 60, 70), ("Cognitive", 10, 15)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Status of a rated value; a function takes the worst band of its metrics.
pub enum Band {
    Green,
    Yellow,
    Red,
}

impl Band {
    /// Places a value against one metric's band boundaries.
    fn of(value: usize, (_, green, yellow): (&str, usize, usize)) -> Self {
        if value > yellow {
            Self::Red
        } else if value > green {
            Self::Yellow
        } else {
            Self::Green
        }
    }

    /// Names the band in extracted records.
    pub fn name(self) -> &'static str {
        ["green", "yellow", "red"][self as usize]
    }

    fn colour(self) -> &'static str {
        [GREEN, YELLOW, RED][self as usize]
    }
}

/// Rates a function by the worse of its lines and cognitive complexity bands.
pub fn function_band(f: &FnMetrics) -> Band {
    Band::of(f.loc, BANDS[0]).max(Band::of(f.cognitive as usize, BANDS[1]))
}

/// Controls function detail, the guide, and terminal colour.
pub struct Options {
    pub detail: bool,
    pub colour: bool,
    pub guide: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            detail: false,
            colour: std::io::stdout().is_terminal()
                && std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty()),
            guide: false,
        }
    }
}

/// Prints each file with its yellow and red functions (all functions with `detail`),
/// then the share of function lines in each band.
pub fn print_table(files: &[FileMetrics], options: &Options) {
    let headers = ["File", "Lines", "Cog", "Doc"].map(str::to_owned);
    let mut rows = Vec::new();
    for file in files {
        let partial = if file.parse_errors > 0 { " ~" } else { "" };
        let cells = vec![
            plain(file.loc),
            plain(""),
            plain(yes_no(file.has_docstring)),
        ];
        rows.push((format!("{}{partial}", file.path), cells, true));
        for f in &file.fns {
            if options.detail || function_band(f) != Band::Green {
                rows.push((
                    format!("  {} (L{}–{})", f.name, f.line_start, f.line_end),
                    function_cells(f),
                    false,
                ));
            }
        }
    }
    table::print(&headers, &rows, options.colour, options.detail);
    println!("{}", summary(files));
}

fn function_cells(f: &FnMetrics) -> Vec<(String, &'static str)> {
    let cognitive = f.cognitive as usize;
    vec![
        (f.loc.to_string(), Band::of(f.loc, BANDS[0]).colour()),
        (
            cognitive.to_string(),
            Band::of(cognitive, BANDS[1]).colour(),
        ),
        plain(yes_no(f.has_docstring)),
    ]
}

fn plain(value: impl ToString) -> (String, &'static str) {
    (value.to_string(), "")
}

fn yes_no(present: bool) -> &'static str {
    if present {
        "yes"
    } else {
        "no"
    }
}

/// Summarizes the risk profile, documentation coverage, and partially parsed files.
fn summary(files: &[FileMetrics]) -> String {
    let fns: Vec<&FnMetrics> = files.iter().flat_map(|file| &file.fns).collect();
    let lines = fns.iter().map(|f| f.loc).sum::<usize>().max(1) as f64;
    let share = |band| {
        let banded = fns.iter().filter(|f| function_band(f) == band);
        100.0 * banded.map(|f| f.loc).sum::<usize>() as f64 / lines
    };
    let documented = fns.iter().filter(|f| f.has_docstring).count();
    let modules = files.iter().filter(|file| file.has_docstring).count();
    let partial = files.iter().filter(|file| file.parse_errors > 0).count();
    format!(
        "{} files, {} functions: {:.1}% of function lines in yellow functions, {:.1}% in red; \
         docs on {modules}/{} files and {documented}/{} functions{}",
        files.len(),
        fns.len(),
        share(Band::Yellow),
        share(Band::Red),
        files.len(),
        fns.len(),
        if partial > 0 {
            format!("; ~ marks {partial} partially parsed")
        } else {
            String::new()
        },
    )
}

/// Prints the bands that rate each function.
pub fn print_guide(colour: bool) {
    let headers = ["Function metric", "Green", "Yellow", "Red"].map(str::to_owned);
    let rows: Vec<_> = (BANDS.iter())
        .map(|&(name, green, yellow)| {
            let cells = vec![
                (format!("<= {green}"), GREEN),
                (format!("{}-{yellow}", green + 1), YELLOW),
                (format!("> {yellow}"), RED),
            ];
            (name.to_owned(), cells, true)
        })
        .collect();
    table::print(&headers, &rows, colour, false);
    println!("A function takes its worst band. File lines and docs are shown, not rated.");
}
