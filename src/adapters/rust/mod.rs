use super::Adapter;
use crate::{metrics, visit};

pub struct RustAdapter;

impl Adapter for RustAdapter {
    fn language(&self) -> &'static str {
        "rust"
    }

    /// Analyse Rust syntax using syn and the block visitor.
    fn analyse(&self, path: &str, source: &str) -> Result<metrics::FileMetrics, String> {
        let file = syn::parse_file(source).map_err(|error| format!("parse error: {error}"))?;
        let (raw, classes, file_has_docstring) = visit::collect(&file);
        let functions = raw.iter().map(metrics::compute).collect();
        Ok(metrics::aggregate(
            self.language(),
            path,
            &raw,
            functions,
            classes,
            file_has_docstring,
            source.lines().count(),
        ))
    }
}
