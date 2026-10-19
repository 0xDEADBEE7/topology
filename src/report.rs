//! Rates functions against the metric bands and prints score tables and the guide.

use crate::{
    table::{self, GREEN, RED, YELLOW},
    walk::{FileInfo, Unit},
};
use std::io::IsTerminal;

/// Rated function metrics: name, largest green value, and largest yellow value.
pub const BANDS: [(&str, usize, usize); 2] = [("Lines", 40, 75), ("Cognitive", 10, 15)];

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
pub fn function_band(f: &Unit) -> Band {
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
pub fn print_table(files: &[FileInfo], options: &Options) {
    let headers = ["File", "Lines", "Cog", "Doc"];
    let mut rows = Vec::new();
    for file in files {
        let partial = if file.parse_errors > 0 { " ~" } else { "" };
        let cells = vec![plain(file.loc), plain(""), doc(file.doc.is_some())];
        rows.push((format!("{}{partial}", file.path), cells, true));
        for f in &file.fns {
            if options.detail || function_band(f) != Band::Green {
                let label = format!("  {} (L{}–{})", f.name, f.line_start, f.line_end);
                rows.push((label, function_cells(f), false));
            }
        }
    }
    table::print(&headers, &rows, options.colour, options.detail);
    println!("{}", summary(files));
}

fn function_cells(f: &Unit) -> Vec<(String, &'static str)> {
    let (loc, cog) = (f.loc, f.cognitive as usize);
    vec![
        (loc.to_string(), Band::of(loc, BANDS[0]).colour()),
        (cog.to_string(), Band::of(cog, BANDS[1]).colour()),
        doc(f.doc.is_some()),
    ]
}

fn plain(value: impl ToString) -> (String, &'static str) {
    (value.to_string(), "")
}

fn doc(present: bool) -> (String, &'static str) {
    plain(if present { "yes" } else { "no" })
}

/// Summarizes the risk profile, documentation coverage, and partially parsed files.
fn summary(files: &[FileInfo]) -> String {
    let fns: Vec<&Unit> = files.iter().flat_map(|file| &file.fns).collect();
    let lines = fns.iter().map(|f| f.loc).sum::<usize>().max(1) as f64;
    let count_and_share = |band| {
        let banded: Vec<_> = fns.iter().filter(|f| function_band(f) == band).collect();
        let share = 100.0 * banded.iter().map(|f| f.loc).sum::<usize>() as f64 / lines;
        (banded.len(), share)
    };
    let (yellow, yellow_share) = count_and_share(Band::Yellow);
    let (red, red_share) = count_and_share(Band::Red);
    let documented = fns.iter().filter(|f| f.doc.is_some()).count();
    let modules = files.iter().filter(|file| file.doc.is_some()).count();
    let partial = match files.iter().filter(|file| file.parse_errors > 0).count() {
        0 => String::new(),
        n => format!("; ~ marks {n} partially parsed"),
    };
    format!(
        "{0} files, {1} functions; {yellow} yellow and {red} red functions hold {yellow_share:.1}% \
         and {red_share:.1}% of function lines; docs on {modules}/{0} files and {documented}/{1} \
         functions{partial}",
        files.len(),
        fns.len(),
    )
}

/// Prints the bands that rate each function.
pub fn print_guide(colour: bool) {
    let headers = ["Function metric", "Green", "Yellow", "Red"];
    let rows = BANDS.map(|(name, green, yellow)| {
        let cells = vec![
            (format!("<= {green}"), GREEN),
            (format!("{}-{yellow}", green + 1), YELLOW),
            (format!("> {yellow}"), RED),
        ];
        (name.to_owned(), cells, true)
    });
    table::print(&headers, &rows, colour, false);
    println!("A function takes its worst band. File lines and docs are shown, not rated.");
}
