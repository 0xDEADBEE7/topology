//! Serializes function and class declarations into shared symbol records.

use super::metadata;
use crate::metrics::{ClassMetrics, FnMetrics};
use serde_json::{json, Value};

/// Builds a declaration record while preserving its metric shape.
pub(super) fn symbol(
    path: &str,
    repo: &str,
    language: &str,
    source: &str,
    item: &impl Item,
    kind: &'static str,
) -> Value {
    json!({
        "id": format!("{}#{}:{}:{}", path, kind, item.name(), item.start()),
        "type": kind,
        "repo": repo,
        "path": path,
        "name": item.name(),
        "kind": kind,
        "qualified_name": metadata::qualified_name(path, item.name()),
        "language": metadata::language_name(language),
        "visibility": metadata::visibility(language, source, item.start()),
        "parent": null,
        "signature": metadata::signature(language, source, item.start()),
        "lines": [item.start(), item.end()],
        "description": metadata::docstring(language, source, item.start()),
        "metrics": item.metrics(),
    })
}

/// Supplies source and metric fields shared by function and class records.
pub(super) trait Item {
    fn name(&self) -> &str;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn metrics(&self) -> Value;
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
    fn metrics(&self) -> Value {
        json!({
            "loc": self.loc,
            "cognitive": self.cognitive,
            "band": crate::report::function_band(self).name(),
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
    fn metrics(&self) -> Value {
        Value::Null
    }
}
