//! Builds serialized file, import, and symbol records for one source file.

use super::models::{Export, FileRecord, Metrics};
use super::symbols::symbol;
use super::{imports, metadata};

use std::{fs, path::Path};

/// Parse one source file and assemble its file, import, and symbol records.
pub(super) fn records(
    root: &Path,
    repo: &str,
    path: &Path,
) -> Result<Vec<serde_json::Value>, String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    let relative = relative.strip_prefix("./").unwrap_or(&relative).to_owned();
    let analysed = crate::adapters::analyse(&relative, &source)?;
    let language = analysed.language;
    let imports = imports::imports(root, repo, &relative, language, &source);
    let import_ids = imports.iter().map(|import| import.id.clone()).collect();
    let exports = analysed
        .fns
        .iter()
        .filter_map(|item| export(language, &source, item.line_start, item.line_end))
        .chain(
            analysed
                .classes
                .iter()
                .filter_map(|item| export(language, &source, item.line_start, item.line_end)),
        )
        .collect();
    let file = file_record(&relative, repo, &source, &analysed, import_ids, exports);
    let mut output = vec![serde_json::to_value(file).map_err(|e| e.to_string())?];
    output.extend(
        imports
            .into_iter()
            .map(|import| serde_json::to_value(import).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?,
    );
    for function in &analysed.fns {
        output.push(symbol(
            &relative, repo, language, &source, function, "function",
        )?);
    }
    for class in &analysed.classes {
        output.push(symbol(
            &relative,
            repo,
            language,
            &source,
            class,
            metadata::class_kind(language),
        )?);
    }
    Ok(output)
}

fn export(language: &str, source: &str, start: usize, end: usize) -> Option<Export> {
    metadata::is_exported(language, source, start).then(|| Export {
        signature: metadata::signature(language, source, start),
        lines: [start, end],
        description: metadata::docstring(language, source, start),
    })
}

/// Combines file metadata with the adapter's aggregate measurements.
fn file_record(
    relative: &str,
    repo: &str,
    source: &str,
    analysed: &crate::metrics::FileMetrics,
    imports: Vec<String>,
    exports: Vec<Export>,
) -> FileRecord {
    let language = analysed.language;
    FileRecord {
        id: relative.to_owned(),
        record_type: "file",
        repo: repo.to_owned(),
        path: relative.to_owned(),
        description: metadata::module_doc(source),
        imports,
        exports,
        metrics: Metrics {
            language,
            loc: analysed.loc,
            cc: analysed.cc,
            cognitive: analysed.cognitive,
            halstead: analysed.halstead,
            nmi: analysed.nmi,
            parse_errors: analysed.parse_errors,
        },
    }
}
