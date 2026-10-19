//! Formats function-level metrics using the report rating thresholds.

use super::{
    rating::{doc_colour, rate},
    Metric,
};
use crate::{metrics::FnMetrics, table::GREEN};

/// Formats a function metric together with its status colour.
pub(super) fn fn_value(f: &FnMetrics, m: Metric) -> (String, &'static str) {
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
