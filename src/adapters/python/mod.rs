//! Adapts Python source to the shared line-oriented analyser.

use super::{common, Adapter};

pub struct PythonAdapter;

impl Adapter for PythonAdapter {
    fn language(&self) -> &'static str {
        "python"
    }

    fn analyse(&self, path: &str, source: &str) -> Result<crate::metrics::FileMetrics, String> {
        Ok(common::analyse_text(self.language(), path, source))
    }
}
