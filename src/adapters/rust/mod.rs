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
        let (raw, classes, file_has_docstring) = visit::collect(&file);
        Ok(metrics::file(
            self.language(),
            path,
            &raw,
            classes,
            file_has_docstring,
            source.lines().count(),
        ))
    }
}
