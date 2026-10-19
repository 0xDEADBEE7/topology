//! Formats assessments and explains the thresholds used to rate metrics.

mod file;
mod function;
mod guide;
mod options;
mod rating;
use crate::{metrics::FileMetrics, table};
use file::file_value;
use function::fn_value;
pub use guide::print_guide;
pub use options::{parse_metrics, Metric, Options};

/// Labels each assessment dimension in report headers.
fn metric_name(metric: Metric) -> &'static str {
    match metric {
        Metric::Nmi => "NMI",
        Metric::Loc => "LoC",
        Metric::Docstring => "Docs",
        Metric::Cc => "CC",
        Metric::Cognitive => "Cog",
        Metric::Halstead => "HV",
    }
}

/// Renders file summaries and optional per-function measurements.
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
