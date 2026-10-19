//! Prints one JSON record per file, import and declaration, resolving relative imports.

use crate::{
    lang::{Language, EXTENSIONS},
    metrics::{FileInfo, Import, Unit},
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Prints the records of every supported file under `target`, one JSON object per line.
pub fn run(target: &str) -> Result<(), String> {
    let target = fs::canonicalize(target).map_err(|e| format!("{target}: {e}"))?;
    let root = target.parent().filter(|_| target.is_file());
    let root = root.unwrap_or(&target);
    for file in crate::paths::analyse(&[&target], root)? {
        for record in records(root, &file) {
            println!("{record}");
        }
    }
    Ok(())
}

/// Builds a file's record, then its imports and declarations, with ids unique in the file.
fn records(root: &Path, file: &FileInfo) -> Vec<Value> {
    let path = &file.path;
    let metrics =
        json!({"language": file.language, "loc": file.loc, "parse_errors": file.parse_errors});
    let mut records = vec![json!({
        "id": path, "type": "file", "path": path, "description": file.doc, "metrics": metrics,
    })];
    for (ordinal, import) in file.imports.iter().enumerate() {
        records.push(json!({
            "id": format!("{path}#import:{}:{ordinal}", import.line),
            "type": "import", "path": path, "line": import.line,
            "source": import.source, "imported_name": import.name,
            "resolved_path": resolve_import(root, path, import),
        }));
    }
    records.extend(
        file.units()
            .map(|(unit, kind)| unit_record(file, unit, kind)),
    );
    let mut seen = HashMap::new();
    for record in &mut records {
        let id = record["id"].as_str().unwrap_or_default().to_owned();
        let repeats: &mut usize = seen.entry(id.clone()).or_default();
        if *repeats > 0 {
            record["id"] = format!("{id}~{repeats}").into();
        }
        *repeats += 1;
    }
    records
}

/// Describes a function, with its measures and band, or a class.
fn unit_record(file: &FileInfo, unit: &Unit, kind: &str) -> Value {
    let band = crate::report::function_band(unit).name();
    let metrics = json!({"band": band, "cognitive": unit.cognitive, "loc": unit.loc});
    json!({
        "id": format!("{}#{kind}:{}:{}", file.path, unit.name, unit.line_start),
        "type": kind, "path": file.path, "name": unit.name, "language": file.language,
        "exported": unit.exported, "parent": unit.parent, "signature": unit.signature,
        "lines": [unit.line_start, unit.line_end], "description": unit.doc,
        "metrics": (kind == "function").then_some(metrics),
    })
}

/// Resolves a relative script specifier or Python module to a file under `root`.
fn resolve_import(root: &Path, current: &str, import: &Import) -> Option<String> {
    let directory = root.join(current).parent()?.to_owned();
    let candidates = match Language::from_path(current)? {
        _ if !import.source.starts_with('.') => return None,
        Language::Python => python_candidates(directory, import),
        _ => script_candidates(&directory.join(&import.source)),
    };
    candidates.into_iter().find_map(|candidate| {
        let path = candidate
            .is_file()
            .then(|| candidate.canonicalize().ok())??;
        Some(path.strip_prefix(root).ok()?.to_string_lossy().into_owned())
    })
}

/// Lists files `from ..a.b import c` may name: each dot after the first climbs a package,
/// and `c` may be a submodule.
fn python_candidates(mut base: PathBuf, import: &Import) -> Vec<PathBuf> {
    let module = import.source.trim_start_matches('.');
    for _ in 1..import.source.len() - module.len() {
        base.pop();
    }
    base.extend(module.split('.').filter(|part| !part.is_empty()));
    let named = import.name.as_ref().map(|name| base.join(name));
    let modules = named.into_iter().chain([base]);
    modules
        .flat_map(|module| [module.join("__init__.py"), module.with_extension("py")])
        .collect()
}

/// Lists files a relative script import may name: the path itself, the path with each
/// script extension or index file, and TypeScript sources imported by their `.js` names.
fn script_candidates(base: &Path) -> Vec<PathBuf> {
    let scripts = || {
        EXTENSIONS
            .iter()
            .filter(|(_, language)| !matches!(language, Language::Rust | Language::Python))
    };
    let mut candidates = vec![base.to_owned()];
    for (extension, _) in scripts() {
        let mut file = base.as_os_str().to_owned();
        file.push(format!(".{extension}"));
        candidates.push(file.into());
    }
    candidates.extend(scripts().map(|(extension, _)| base.join(format!("index.{extension}"))));
    if Language::from_path(base) == Some(Language::JavaScript) {
        let typescript = scripts().filter(|(_, language)| *language != Language::JavaScript);
        candidates.extend(typescript.map(|(extension, _)| base.with_extension(extension)));
    }
    candidates
}
