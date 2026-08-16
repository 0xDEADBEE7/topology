//! Serialized structures shared by extraction record builders.

use serde::Serialize;

#[derive(Clone, Serialize)]
pub(super) struct ImportRecord {
    pub(super) id: String,
    #[serde(rename = "type")]
    pub(super) record_type: &'static str,
    pub(super) repo: String,
    pub(super) path: String,
    pub(super) language: &'static str,
    pub(super) line: usize,
    pub(super) source: String,
    pub(super) local_name: Option<String>,
    pub(super) imported_name: Option<String>,
    pub(super) resolved_path: Option<String>,
    pub(super) resolved_symbol: Option<String>,
    pub(super) resolution: &'static str,
}

#[derive(Serialize)]
pub(super) struct FileRecord {
    pub(super) id: String,
    #[serde(rename = "type")]
    pub(super) record_type: &'static str,
    pub(super) repo: String,
    pub(super) path: String,
    pub(super) description: Option<String>,
    pub(super) imports: Vec<String>,
    pub(super) exports: Vec<Export>,
    pub(super) metrics: Metrics,
}

#[derive(Serialize)]
pub(super) struct SymbolRecord {
    pub(super) id: String,
    #[serde(rename = "type")]
    pub(super) record_type: &'static str,
    pub(super) repo: String,
    pub(super) path: String,
    pub(super) name: String,
    pub(super) kind: &'static str,
    pub(super) qualified_name: String,
    pub(super) language: &'static str,
    pub(super) visibility: &'static str,
    pub(super) parent: Option<String>,
    pub(super) signature: String,
    pub(super) lines: [usize; 2],
    pub(super) description: Option<String>,
    pub(super) metrics: Option<SymbolMetrics>,
}

#[derive(Serialize)]
pub(super) struct Export {
    pub(super) signature: String,
    pub(super) lines: [usize; 2],
    pub(super) description: Option<String>,
}

#[derive(Serialize)]
pub(super) struct SymbolMetrics {
    pub(super) loc: usize,
    pub(super) cc: u32,
    pub(super) cognitive: u32,
    pub(super) halstead: f64,
}

#[derive(Serialize)]
pub(super) struct Metrics {
    pub(super) language: &'static str,
    pub(super) loc: usize,
    pub(super) cc: u32,
    pub(super) cognitive: u32,
    pub(super) halstead: f64,
    pub(super) nmi: f64,
}
