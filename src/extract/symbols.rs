//! Serializes function and class declarations into shared symbol records.

use super::{
    metadata,
    models::{SymbolMetrics, SymbolRecord},
};
use crate::metrics::{ClassMetrics, FnMetrics};

/// Builds a declaration record while preserving its metric shape.
pub(super) fn symbol<T>(
    path: &str,
    repo: &str,
    language: &str,
    source: &str,
    item: &T,
    kind: &'static str,
) -> Result<serde_json::Value, String>
where
    T: Item,
{
    serde_json::to_value(SymbolRecord {
        id: format!("{}#{}:{}:{}", path, kind, item.name(), item.start()),
        record_type: kind,
        repo: repo.to_owned(),
        path: path.to_owned(),
        name: item.name().to_owned(),
        kind,
        qualified_name: metadata::qualified_name(path, item.name()),
        language: metadata::language_name(language),
        visibility: metadata::visibility(language, source, item.start()),
        parent: None,
        signature: metadata::signature(language, source, item.start()),
        lines: [item.start(), item.end()],
        description: metadata::docstring(language, source, item.start()),
        metrics: item.metrics(),
    })
    .map_err(|e| e.to_string())
}

/// Supplies source and metric fields shared by function and class records.
pub(super) trait Item {
    fn name(&self) -> &str;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn metrics(&self) -> Option<SymbolMetrics>;
}

impl Item for FnMetrics {
    fn name(&self) -> &str {
        &self.name
    }
    fn start(&self) -> usize {
        self.line_start
    }
    fn end(&self) -> usize {
        self.line_end
    }
    fn metrics(&self) -> Option<SymbolMetrics> {
        Some(SymbolMetrics {
            loc: self.loc,
            cognitive: self.cognitive,
            band: crate::report::function_band(self).name(),
        })
    }
}
impl Item for ClassMetrics {
    fn name(&self) -> &str {
        &self.name
    }
    fn start(&self) -> usize {
        self.line_start
    }
    fn end(&self) -> usize {
        self.line_end
    }
    fn metrics(&self) -> Option<SymbolMetrics> {
        None
    }
}
