use super::Adapter;
use crate::{metrics, visit};

pub struct RustAdapter;

impl Adapter for RustAdapter {
    fn language(&self) -> &'static str {
        "rust"
    }

    fn analyse(&self, path: &str, source: &str) -> Result<metrics::FileMetrics, String> {
        let file = syn::parse_file(source).map_err(|error| format!("parse error: {error}"))?;
        let raw = visit::collect(&file);
        let functions = raw.iter().map(metrics::compute).collect();
        Ok(metrics::aggregate(
            self.language(),
            path,
            &raw,
            functions,
            source.lines().count(),
        ))
    }
}
