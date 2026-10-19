//! Provides approximate line-based metrics for scripting-language adapters.

mod counts;
mod syntax;
use crate::metrics::{ClassMetrics, FileMetrics, FnMetrics};
use counts::{close_count, decision_count, open_count};
use syntax::{function_name, has_file_docstring, has_python_docstring};

/// Analyse source with the lightweight line-based adapter shared by scripting languages.
pub fn analyse_text(language: &'static str, path: &str, source: &str) -> FileMetrics {
    FileMetrics {
        language,
        path: path.to_owned(),
        loc: source.lines().count(),
        has_docstring: has_file_docstring(language, source),
        parse_errors: 0,
        fns: collect(language, source),
        classes: classes(language, source),
    }
}

/// Collects approximate function counts from declaration-delimited line spans.
fn collect(language: &str, source: &str) -> Vec<FnMetrics> {
    let mut raw = Vec::new();
    let mut current: Option<FnMetrics> = None;
    let mut depth = 0_u32;

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(name) = function_name(language, trimmed) {
            if let Some(mut function) = current.take() {
                function.line_end = index;
                raw.push(function);
            }
            current = Some(FnMetrics {
                name,
                line_start: index + 1,
                line_end: index + 1,
                cognitive: 0,
                has_docstring: language == "python" && has_python_docstring(source, index),
                nested_lines: 0,
            });
        }
        if let Some(function) = current.as_mut() {
            function.line_end = index + 1;
            function.cognitive += decision_count(language, trimmed) * (1 + depth);
            depth = depth.saturating_add(open_count(trimmed));
            depth = depth.saturating_sub(close_count(trimmed));
        }
    }
    if let Some(function) = current {
        raw.push(function);
    }
    raw
}

/// Locates class declarations for the Python outline.
fn classes(language: &str, source: &str) -> Vec<ClassMetrics> {
    if language != "python" {
        return Vec::new();
    }
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let rest = line.trim_start().strip_prefix("class ")?;
            let name = rest.split(['(', ':']).next()?.trim();
            (!name.is_empty()).then(|| ClassMetrics {
                name: name.to_owned(),
                line_start: index + 1,
                line_end: index + 1,
            })
        })
        .collect()
}
