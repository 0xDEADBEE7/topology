use crate::{metrics, visit::RawCounts};

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
    metrics::aggregate(language, path, &raw, functions, source.lines().count())
}

fn function_name(language: &str, line: &str) -> Option<String> {
    let marker = if language == "python" {
        "def "
    } else {
        "function "
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
