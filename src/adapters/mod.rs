pub mod base;
pub mod common;
pub mod python;
pub mod rust;
pub mod typescript;

pub use base::Adapter;

pub fn for_path(path: &str) -> Result<Box<dyn Adapter>, String> {
    match std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("rs") => Ok(Box::new(rust::RustAdapter)),
        Some("py") => Ok(Box::new(python::PythonAdapter)),
        Some("ts") | Some("tsx") => Ok(Box::new(typescript::TypeScriptAdapter)),
        Some(extension) => Err(format!("unsupported language: .{extension}")),
        None => Err("cannot determine language: file has no extension".to_owned()),
    }
}
