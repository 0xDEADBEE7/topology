//! Defines the contract implemented by each supported language adapter.

use crate::metrics::FileMetrics;

/// Converts one source file into language-neutral assessment metrics.
pub trait Adapter {
    /// Identifies the language used in reports and extraction records.
    fn language(&self) -> &'static str;
    /// Measures source or explains why it could not be parsed.
    fn analyse(&self, path: &str, source: &str) -> Result<FileMetrics, String>;
}
