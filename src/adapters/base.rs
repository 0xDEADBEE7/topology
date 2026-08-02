use crate::metrics::FileMetrics;

pub trait Adapter {
    fn language(&self) -> &'static str;
    fn analyse(&self, path: &str, source: &str) -> Result<FileMetrics, String>;
}
