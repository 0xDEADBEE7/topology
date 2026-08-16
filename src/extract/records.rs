//! Builds serialized file, import, and symbol records for one source file.

use super::models::{Export, FileRecord, Metrics, SymbolMetrics, SymbolRecord};
use super::{imports, metadata};
use crate::metrics::{ClassMetrics, FnMetrics};
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
    let adapter = crate::adapters::for_path(path.to_string_lossy().as_ref())?;
    let analysed = adapter.analyse(&relative, &source)?;
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
    let file = FileRecord {
        id: relative.clone(),
        record_type: "file",
        repo: repo.to_owned(),
        path: relative.clone(),
        description: metadata::docstring(language, &source, None),
        imports: import_ids,
        exports,
        metrics: Metrics {
            language,
            loc: analysed.loc,
            cc: analysed.cc,
            cognitive: analysed.cognitive,
            halstead: analysed.halstead,
            nmi: analysed.nmi,
        },
    };
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
        description: metadata::docstring(language, source, Some(start)),
    })
}

fn symbol<T>(
    path: &str,
    repo: &str,
    language: &str,
    source: &str,
    item: &T,
    kind: &'static str,
) -> Result<serde_json::Value, String>
where
    T: Item,
{
    serde_json::to_value(SymbolRecord {
        id: format!("{}#{}:{}:{}", path, kind, item.name(), item.start()),
        record_type: kind,
        repo: repo.to_owned(),
        path: path.to_owned(),
        name: item.name().to_owned(),
        kind,
        qualified_name: metadata::qualified_name(path, item.name()),
        language: metadata::language_name(language),
        visibility: metadata::visibility(language, source, item.start()),
        parent: None,
        signature: metadata::signature(language, source, item.start()),
        lines: [item.start(), item.end()],
        description: metadata::docstring(language, source, Some(item.start())),
        metrics: item.metrics(),
    })
    .map_err(|e| e.to_string())
}

trait Item {
    fn name(&self) -> &str;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn metrics(&self) -> Option<SymbolMetrics>;
}

impl Item for FnMetrics {
    fn name(&self) -> &str {
        &self.name
    }
    fn start(&self) -> usize {
        self.line_start
    }
    fn end(&self) -> usize {
        self.line_end
    }
    fn metrics(&self) -> Option<SymbolMetrics> {
        Some(SymbolMetrics {
            loc: self.loc,
            cc: self.cc,
            cognitive: self.cognitive,
            halstead: self.halstead,
        })
    }
}
impl Item for ClassMetrics {
    fn name(&self) -> &str {
        &self.name
    }
    fn start(&self) -> usize {
        self.line_start
    }
    fn end(&self) -> usize {
        self.line_end
    }
    fn metrics(&self) -> Option<SymbolMetrics> {
        None
    }
}
