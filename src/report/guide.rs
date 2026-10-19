//! Prints the metric thresholds used by assessment reports.

use crate::table::{self, GREEN, RED, YELLOW};

/// Prints the rating thresholds for each metric and scope.
pub fn print_guide(colour: bool) {
    let headers = vec![
        "Metric".to_owned(),
        "Scope".to_owned(),
        "Green".to_owned(),
        "Yellow".to_owned(),
        "Red".to_owned(),
    ];
    let rows = vec![
        guide_row("NMI", "file", ">= 20", "< 20", "< 10"),
        guide_row("LoC", "file", "< 300", ">= 300", ">= 350"),
        guide_row("LoC", "function", "<= 60", "> 60", "> 70"),
        guide_row("Docs", "file", "present", "N/A", "missing"),
        guide_row("Docs", "function", "present", "N/A", "missing"),
        guide_row("Cyclomatic complexity", "file", "< 10", ">= 10", ">= 15"),
        guide_row(
            "Cyclomatic complexity",
            "function",
            "< 10",
            ">= 10",
            ">= 15",
        ),
        guide_row("Cognitive complexity", "file", "< 10", ">= 10", ">= 15"),
        guide_row("Cognitive complexity", "function", "< 10", ">= 10", ">= 15"),
        guide_row("Halstead volume", "file", "< 500", ">= 500", ">= 1000"),
        guide_row("Halstead volume", "function", "< 500", ">= 500", ">= 1000"),
    ];
    table::print(&headers, &rows, colour, false);
}

fn guide_row(
    metric: &str,
    scope: &str,
    green: &str,
    yellow: &str,
    red: &str,
) -> (String, Vec<(String, &'static str)>, bool) {
    (
        metric.to_owned(),
        vec![
            (scope.to_owned(), GREEN),
            (green.to_owned(), GREEN),
            (yellow.to_owned(), YELLOW),
            (red.to_owned(), RED),
        ],
        true,
    )
}
