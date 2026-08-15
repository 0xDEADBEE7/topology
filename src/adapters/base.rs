//! Defines the contract implemented by each supported language adapter.

//! Defines the contract implemented by each supported language adapter.

use crate::metrics::FileMetrics;

pub trait Adapter {
    fn language(&self) -> &'static str;
    fn analyse(&self, path: &str, source: &str) -> Result<FileMetrics, String>;
}
