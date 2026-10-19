//! Formats file-level metrics using the report rating thresholds.

use super::{
    rating::{doc_colour, rate},
    Metric,
};
use crate::{
    metrics::FileMetrics,
    table::{GREEN, RED, YELLOW},
};

/// Formats a file metric together with its status colour.
pub(super) fn file_value(f: &FileMetrics, m: Metric) -> (String, &'static str) {
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
