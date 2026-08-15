use crate::{metrics, visit::RawCounts};

/// Analyse source with the lightweight line-based adapter shared by scripting languages.
pub fn analyse_text(language: &'static str, path: &str, source: &str) -> metrics::FileMetrics {
    let mut raw = Vec::new();
    let mut current: Option<RawCounts> = None;
    let mut depth = 0_u32;

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(name) = function_name(language, trimmed) {
            if let Some(mut function) = current.take() {
                function.line_end = index;
                raw.push(function);
            }
            current = Some(RawCounts {
                name,
                line_start: index + 1,
                line_end: index + 1,
                decisions: 1,
                has_docstring: language == "python" && has_python_docstring(source, index),
                ..RawCounts::default()
            });
        }
        if let Some(function) = current.as_mut() {
            function.line_end = index + 1;
            let decisions = decision_count(language, trimmed);
            function.decisions += decisions;
            function.cognitive += decisions * (1 + depth);
            function.n1 += operator_count(trimmed);
            function.n2 += operand_count(trimmed);
            depth = depth.saturating_add(open_count(trimmed));
            depth = depth.saturating_sub(close_count(trimmed));
        }
    }
    if let Some(function) = current {
        raw.push(function);
    }
    for function in &mut raw {
        function.dn1 = function.n1.max(1);
        function.dn2 = function.n2.max(1);
    }
    let functions = raw.iter().map(metrics::compute).collect();
    let mut classes = Vec::new();
    if language == "python" {
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("class ") {
                let name = rest.split(['(', ':']).next().unwrap_or("").trim();
                if !name.is_empty() {
                    classes.push(metrics::ClassMetrics {
                        name: name.to_owned(),
                        line_start: index + 1,
                        line_end: index + 1,
                    });
                }
            }
        }
    }
    metrics::aggregate(
        language,
        path,
        &raw,
        functions,
        classes,
        has_file_docstring(language, source),
        source.lines().count(),
    )
}

fn has_file_docstring(language: &str, source: &str) -> bool {
    if language == "python" {
        return source
            .lines()
            .find(|line| !line.trim().is_empty())
            .is_some_and(|line| {
                let line = line.trim();
                line.starts_with("\"\"\"") || line.starts_with("''' ") || line == "'''"
            });
    }
    source
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.trim_start().starts_with("/**"))
}

fn has_python_docstring(source: &str, index: usize) -> bool {
    let lines: Vec<&str> = source.lines().skip(index + 1).collect();
    lines
        .iter()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| {
            let line = line.trim_start();
            line.starts_with("\"\"\"") || line.starts_with("'''")
        })
}
fn function_name(language: &str, line: &str) -> Option<String> {
    let line = line.trim_start();
    let marker = if language == "python" {
        if line.starts_with("async def ") {
            "async def "
        } else {
            "def "
        }
    } else if line.starts_with("export function ") {
        "export function "
    } else if line.starts_with("function ") {
        "function "
    } else {
        return None;
    };
    let start = line.find(marker)? + marker.len();
    let name: String = line[start..]
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

fn decision_count(language: &str, line: &str) -> u32 {
    let words = if language == "python" {
        ["if ", "elif ", "for ", "while ", "except ", "match "]
    } else {
        ["if ", "else if ", "for ", "while ", "case ", "catch "]
    };
    words
        .iter()
        .map(|word| line.matches(word).count() as u32)
        .sum::<u32>()
        + line.matches("&&").count() as u32
        + line.matches("||").count() as u32
}

fn operator_count(line: &str) -> u32 {
    ["=", "+", "-", "*", "/", "==", "=>", "<", ">", "&&", "||"]
        .iter()
        .map(|operator| line.matches(operator).count() as u32)
        .sum()
}

fn operand_count(line: &str) -> u32 {
    line.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|part| !part.is_empty())
        .count() as u32
}

fn open_count(line: &str) -> u32 {
    line.matches('{').count() as u32
}
fn close_count(line: &str) -> u32 {
    line.matches('}').count() as u32
}
