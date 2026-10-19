//! Defines report options and metric selection syntax.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Selects an assessment dimension for tabular output.
pub enum Metric {
    Nmi,
    Loc,
    Docstring,
    Cc,
    Cognitive,
    Halstead,
}
#[derive(Clone, Debug)]
/// Controls report detail, selected metrics, and terminal presentation.
pub struct Options {
    pub detail: bool,
    pub colour: bool,
    pub metrics: Vec<Metric>,
    pub guide: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            detail: false,
            colour: true,
            metrics: vec![Metric::Nmi],
            guide: false,
        }
    }
}

/// Parses the CLI's comma-separated metric selection, removing duplicates.
pub fn parse_metrics(value: &str) -> Result<Vec<Metric>, String> {
    let mut result = Vec::new();
    for name in value.split(',').map(|s| s.trim().to_ascii_lowercase()) {
        let metric = metric_from_name(&name)?;
        if !result.contains(&metric) {
            result.push(metric);
        }
    }
    if result.is_empty() {
        Err("--metrics requires at least one metric".into())
    } else {
        Ok(result)
    }
}

/// Resolves the documented metric aliases.
fn metric_from_name(name: &str) -> Result<Metric, String> {
    let metric = match name {
        "nmi" => Metric::Nmi,
        "loc" | "lines" => Metric::Loc,
        "doc" | "docstring" | "docs" => Metric::Docstring,
        "cc" | "cyclomatic" => Metric::Cc,
        "cognitive" | "cog" => Metric::Cognitive,
        "halstead" | "hv" => Metric::Halstead,
        _ => return Err(format!("unknown metric '{name}'")),
    };
    Ok(metric)
}
