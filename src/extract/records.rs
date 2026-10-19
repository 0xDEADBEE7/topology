//! Builds serialized file, import, and symbol records for one source file.

use super::symbols::symbol;
use super::{imports, metadata};
use serde_json::{json, Value};

use std::{fs, path::Path};

/// Parse one source file and assemble its file, import, and symbol records.
pub(super) fn records(root: &Path, repo: &str, path: &Path) -> Result<Vec<Value>, String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    let relative = relative.strip_prefix("./").unwrap_or(&relative).to_owned();
    let analysed = crate::walk::analyse(&relative, &source)?;
    let language = analysed.language;
    let imports = imports::imports(root, repo, &analysed);
    let exports: Vec<_> = (analysed.fns.iter().map(|f| (f.line_start, f.line_end)))
        .chain(analysed.classes.iter().map(|c| (c.line_start, c.line_end)))
        .filter_map(|(start, end)| export(language, &source, start, end))
        .collect();
    let mut output = vec![json!({
        "id": relative,
        "type": "file",
        "repo": repo,
        "path": relative,
        "description": metadata::module_doc(&source),
        "imports": imports.iter().map(|import| import["id"].clone()).collect::<Vec<_>>(),
        "exports": exports,
        "metrics": {
            "language": language,
            "loc": analysed.loc,
            "parse_errors": analysed.parse_errors,
        },
    })];
    output.extend(imports);
    for function in &analysed.fns {
        output.push(symbol(
            &relative, repo, language, &source, function, "function",
        ));
    }
    let kind = metadata::class_kind(language);
    for class in &analysed.classes {
        output.push(symbol(&relative, repo, language, &source, class, kind));
    }
    unique_ids(&mut output);
    Ok(output)
}

/// Suffixes repeated ids (`~1`, `~2`) so every record in a file has its own.
fn unique_ids(records: &mut [Value]) {
    let mut seen = std::collections::HashMap::new();
    for record in records {
        let id = record["id"].as_str().unwrap_or_default().to_owned();
        let repeats = seen.entry(id.clone()).or_insert(0);
        if *repeats > 0 {
            record["id"] = format!("{id}~{repeats}").into();
        }
        *repeats += 1;
    }
}

fn export(language: &str, source: &str, start: usize, end: usize) -> Option<Value> {
    metadata::is_exported(language, source, start).then(|| {
        json!({
            "signature": metadata::signature(language, source, start),
            "lines": [start, end],
            "description": metadata::docstring(language, source, start),
        })
    })
}
