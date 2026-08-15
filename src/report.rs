use crate::metrics::{FileMetrics, FnMetrics};
use crate::table::{self, GREEN, RED, YELLOW};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    Nmi,
    Loc,
    Docstring,
    Cc,
    Cognitive,
    Halstead,
}
#[derive(Clone, Debug)]
pub struct Options {
    pub detail: bool,
    pub colour: bool,
    pub metrics: Vec<Metric>,
    pub guide: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            detail: false,
            colour: true,
            metrics: vec![Metric::Nmi],
            guide: false,
        }
    }
}

pub fn parse_metrics(value: &str) -> Result<Vec<Metric>, String> {
    let mut result = Vec::new();
    for name in value.split(',').map(|s| s.trim().to_ascii_lowercase()) {
        let metric = match name.as_str() {
            "nmi" => Metric::Nmi,
            "loc" | "lines" => Metric::Loc,
            "doc" | "docstring" | "docs" => Metric::Docstring,
            "cc" | "cyclomatic" => Metric::Cc,
            "cognitive" | "cog" => Metric::Cognitive,
            "halstead" | "hv" => Metric::Halstead,
            _ => return Err(format!("unknown metric '{name}'")),
        };
        if !result.contains(&metric) {
            result.push(metric);
        }
    }
    if result.is_empty() {
        Err("--metrics requires at least one metric".into())
    } else {
        Ok(result)
    }
}

fn metric_name(m: Metric) -> &'static str {
    match m {
        Metric::Nmi => "NMI",
        Metric::Loc => "LoC",
        Metric::Docstring => "Docs",
        Metric::Cc => "CC",
        Metric::Cognitive => "Cog",
        Metric::Halstead => "HV",
    }
}
fn rate(value: u32, yellow: u32, red: u32) -> &'static str {
    if value >= red {
        RED
    } else if value >= yellow {
        YELLOW
    } else {
        GREEN
    }
}
fn doc_colour(has_docstring: bool) -> &'static str {
    if has_docstring {
        GREEN
    } else {
        RED
    }
}

fn file_value(f: &FileMetrics, m: Metric) -> (String, &'static str) {
    match m {
        Metric::Nmi => (
            format!("{:.1}", f.nmi),
            if f.nmi < 10.0 {
                RED
            } else if f.nmi < 20.0 {
                YELLOW
            } else {
                GREEN
            },
        ),
        Metric::Loc => (f.loc.to_string(), rate(f.loc as u32, 300, 350)),
        Metric::Docstring => (
            (if f.has_docstring { "yes" } else { "no" }).into(),
            doc_colour(f.has_docstring),
        ),
        Metric::Cc => (f.cc.to_string(), rate(f.cc, 10, 15)),
        Metric::Cognitive => (f.cognitive.to_string(), rate(f.cognitive, 10, 15)),
        Metric::Halstead => (
            format!("{:.0}", f.halstead),
            rate(f.halstead as u32, 500, 1000),
        ),
    }
}
fn fn_value(f: &FnMetrics, m: Metric) -> (String, &'static str) {
    match m {
        Metric::Nmi => ("-".into(), GREEN),
        Metric::Loc => (f.loc.to_string(), rate(f.loc as u32, 61, 71)),
        Metric::Docstring => (
            (if f.has_docstring { "yes" } else { "no" }).into(),
            doc_colour(f.has_docstring),
        ),
        Metric::Cc => (f.cc.to_string(), rate(f.cc, 10, 15)),
        Metric::Cognitive => (f.cognitive.to_string(), rate(f.cognitive, 10, 15)),
        Metric::Halstead => (
            format!("{:.0}", f.halstead),
            rate(f.halstead as u32, 500, 1000),
        ),
    }
}

pub fn print_guide(colour: bool) {
    let headers = vec![
        "Metric".to_owned(),
        "Scope".to_owned(),
        "Green".to_owned(),
        "Yellow".to_owned(),
        "Red".to_owned(),
    ];
    let rows = vec![
        guide_row("NMI", "file", ">= 20", "< 20", "< 10"),
        guide_row("LoC", "file", "< 300", ">= 300", ">= 350"),
        guide_row("LoC", "function", "<= 60", "> 60", "> 70"),
        guide_row("Docs", "file", "present", "N/A", "missing"),
        guide_row("Docs", "function", "present", "N/A", "missing"),
        guide_row("Cyclomatic complexity", "file", "< 10", ">= 10", ">= 15"),
        guide_row(
            "Cyclomatic complexity",
            "function",
            "< 10",
            ">= 10",
            ">= 15",
        ),
        guide_row("Cognitive complexity", "file", "< 10", ">= 10", ">= 15"),
        guide_row("Cognitive complexity", "function", "< 10", ">= 10", ">= 15"),
        guide_row("Halstead volume", "file", "< 500", ">= 500", ">= 1000"),
        guide_row("Halstead volume", "function", "< 500", ">= 500", ">= 1000"),
    ];
    table::print(&headers, &rows, colour, false);
}

fn guide_row(
    metric: &str,
    scope: &str,
    green: &str,
    yellow: &str,
    red: &str,
) -> (String, Vec<(String, &'static str)>, bool) {
    (
        metric.to_owned(),
        vec![
            (scope.to_owned(), GREEN),
            (green.to_owned(), GREEN),
            (yellow.to_owned(), YELLOW),
            (red.to_owned(), RED),
        ],
        true,
    )
}
pub fn print_table(files: &[FileMetrics], options: &Options) {
    let mut headers = vec!["File".to_owned()];
    headers.extend(options.metrics.iter().map(|&m| metric_name(m).to_owned()));
    let mut rows = Vec::new();
    for f in files {
        rows.push((
            f.path.clone(),
            options.metrics.iter().map(|&m| file_value(f, m)).collect(),
            true,
        ));
        if options.detail {
            for m in &f.fns {
                rows.push((
                    format!("  {} (L{}–{})", m.name, m.line_start, m.line_end),
                    options.metrics.iter().map(|&x| fn_value(m, x)).collect(),
                    false,
                ));
            }
        }
    }
    table::print(&headers, &rows, options.colour, options.detail);
}
