//! Parses language-specific import clauses into names and module references.

/// Expands grouped Rust imports into individual imported names.
pub(super) fn rust_entries(text: &str) -> Vec<(String, Option<String>, Option<String>)> {
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

/// Reads the imported name from a Python from statement.
pub(super) fn python_from(text: &str) -> Vec<(String, Option<String>, Option<String>)> {
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

/// Reads a Python module import.
pub(super) fn python_import(text: &str) -> Vec<(String, Option<String>, Option<String>)> {
    text.strip_prefix("import ")
        .and_then(|value| value.split_whitespace().next())
        .map(|value| {
            let local = value.rsplit('.').next().unwrap_or(value).to_owned();
            vec![(value.to_owned(), Some(local), Some(value.to_owned()))]
        })
        .unwrap_or_default()
}

/// Reads the source module from a JavaScript-style import.
pub(super) fn javascript_import(text: &str) -> Vec<(String, Option<String>, Option<String>)> {
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
