//! Registers language adapters and their supported source extensions.

pub mod base;
pub mod common;
pub mod python;
pub mod rust;
pub mod typescript;

use crate::metrics::FileMetrics;
pub use base::Adapter;

/// Extensions measured with the TypeScript grammars; JavaScript reuses TSX.
pub const ECMA_EXTENSIONS: [&str; 8] = ["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"];

/// Select the language adapter from a source path.
pub fn for_path(path: &str) -> Result<Box<dyn Adapter>, String> {
    match std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("rs") => Ok(Box::new(rust::RustAdapter)),
        Some("py") => Ok(Box::new(python::PythonAdapter)),
        Some(extension) if ECMA_EXTENSIONS.contains(&extension) => {
            Ok(Box::new(typescript::TypeScriptAdapter))
        }
        Some(extension) => Err(format!("unsupported language: .{extension}")),
        None => Err("cannot determine language: file has no extension".to_owned()),
    }
}

/// Measures one source file, warning when only part of it could be parsed.
pub fn analyse(path: &str, source: &str) -> Result<FileMetrics, String> {
    let metrics = for_path(path)?.analyse(path, source)?;
    if metrics.parse_errors > 0 {
        eprintln!(
            "warning: {path}: {} syntax errors; measured the recoverable code",
            metrics.parse_errors
        );
    }
    Ok(metrics)
}
