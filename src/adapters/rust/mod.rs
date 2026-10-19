//! Adapts Rust syntax trees to shared file and function metrics.

use super::Adapter;
use crate::{metrics, visit};

/// Measures Rust functions and structures through syn visitors.
pub struct RustAdapter;

impl Adapter for RustAdapter {
    fn language(&self) -> &'static str {
        "rust"
    }

    /// Analyse Rust syntax using syn and the block visitor.
    fn analyse(&self, path: &str, source: &str) -> Result<metrics::FileMetrics, String> {
        let file = syn::parse_file(source).map_err(|error| format!("parse error: {error}"))?;
        let (fns, classes, has_docstring) = visit::collect(&file);
        Ok(metrics::FileMetrics {
            language: self.language(),
            path: path.to_owned(),
            loc: source.lines().count(),
            has_docstring,
            parse_errors: 0,
            fns,
            classes,
        })
    }
}
