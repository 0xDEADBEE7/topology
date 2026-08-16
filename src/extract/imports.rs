//! Parses imports and enriches them with local resolution metadata.

use super::metadata::{language_name, module_path, resolve_import};
use super::models::ImportRecord;
use std::path::Path;

pub(super) fn imports(
    root: &Path,
    repo: &str,
    relative: &str,
    language: &str,
    source: &str,
) -> Vec<ImportRecord> {
    source
        .lines()
        .enumerate()
        .flat_map(|(index, line)| parse(language, line, index + 1))
        .enumerate()
        .map(|(ordinal, mut import)| {
            import.id = format!("{}#import:{}:{}", relative, import.line, ordinal);
            import.repo = repo.to_owned();
            import.path = relative.to_owned();
            import.language = language_name(language);
            import.resolved_path = resolve_import(root, relative, language, &import.source);
            import.resolved_symbol = import.resolved_path.as_ref().and_then(|path| {
                import
                    .imported_name
                    .as_ref()
                    .map(|name| format!("{}::{name}", module_path(path)))
            });
            import.resolution = if import.resolved_path.is_some() {
                "local"
            } else {
                "unresolved"
            };
            import
        })
        .collect()
}

pub(super) fn parse(language: &str, line: &str, line_number: usize) -> Vec<ImportRecord> {
    let text = line.trim();
    let entries = if language == "rust" && text.starts_with("use ") {
        rust_entries(text)
    } else if language == "python" && text.starts_with("from ") {
        python_from(text)
    } else if language == "python" && text.starts_with("import ") {
        python_import(text)
    } else if (language == "typescript" || language == "javascript") && text.starts_with("import ")
    {
        javascript_import(text)
    } else {
        Vec::new()
    };
    entries
        .into_iter()
        .map(|(source, local_name, imported_name)| ImportRecord {
            id: String::new(),
            record_type: "import",
            repo: String::new(),
            path: String::new(),
            language: language_name(language),
            line: line_number,
            source,
            local_name,
            imported_name,
            resolved_path: None,
            resolved_symbol: None,
            resolution: "unresolved",
        })
        .collect()
}

fn rust_entries(text: &str) -> Vec<(String, Option<String>, Option<String>)> {
    let value = match text.strip_prefix("use ") {
        Some(value) => value.trim_end_matches(';').trim(),
        None => return Vec::new(),
    };
    if let Some((prefix, members)) = value.split_once('{') {
        members
            .trim_end_matches('}')
            .split(',')
            .filter_map(|member| {
                let member = member.trim();
                if member.is_empty() {
                    return None;
                }
                let (name, local) = member.split_once(" as ").unwrap_or((member, member));
                let imported = name.rsplit("::").next().unwrap_or(name);
                let local = if local == member { imported } else { local };
                Some((
                    format!("{}::{name}", prefix.trim_end_matches("::")),
                    Some(local.to_owned()),
                    Some(imported.to_owned()),
                ))
            })
            .collect()
    } else {
        let name = value.rsplit("::").next().unwrap_or(value).to_owned();
        vec![(value.to_owned(), Some(name.clone()), Some(name))]
    }
}

fn python_from(text: &str) -> Vec<(String, Option<String>, Option<String>)> {
    let mut parts = text.split_whitespace();
    let module = match parts.nth(1) {
        Some(value) => value,
        None => return Vec::new(),
    };
    parts
        .next()
        .and_then(|_| parts.next())
        .map(|name| {
            vec![(
                format!("{module}.{name}"),
                Some(name.to_owned()),
                Some(name.to_owned()),
            )]
        })
        .unwrap_or_default()
}

fn python_import(text: &str) -> Vec<(String, Option<String>, Option<String>)> {
    text.strip_prefix("import ")
        .and_then(|value| value.split_whitespace().next())
        .map(|value| {
            let local = value.rsplit('.').next().unwrap_or(value).to_owned();
            vec![(value.to_owned(), Some(local), Some(value.to_owned()))]
        })
        .unwrap_or_default()
}

fn javascript_import(text: &str) -> Vec<(String, Option<String>, Option<String>)> {
    text.rsplit(" from ")
        .next()
        .map(|source| {
            vec![(
                source.trim().trim_matches(['\'', '\"', ';']).to_owned(),
                None,
                None,
            )]
        })
        .unwrap_or_default()
}
