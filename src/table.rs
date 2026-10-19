//! Renders width-constrained metric tables with optional status colours.

/// Marks a metric in the red band.
pub const RED: &str = "\x1b[0;31m";
/// Marks a metric in the caution band.
pub const YELLOW: &str = "\x1b[1;33m";
/// Marks a metric in the healthy band.
pub const GREEN: &str = "\x1b[0;32m";
/// Restores terminal styling after a coloured value.
pub const RESET: &str = "\x1b[0m";

/// Holds a label, coloured metric cells, and a file-boundary marker.
pub type Row = (String, Vec<(String, &'static str)>, bool);

fn truncate_left(s: &str, width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= width {
        format!("{s:<width$}")
    } else {
        let tail: String = chars[chars.len() - (width - 3)..].iter().collect();
        format!("...{tail}")
    }
}

/// Prints aligned rows, optionally separating file groups.
pub fn print(headers: &[String], rows: &[Row], colour: bool, separate_files: bool) {
    let widths = column_widths(headers, rows);
    let border = |left: &str, join: &str, right: &str| {
        format!(
            "{left}{}{right}",
            widths
                .iter()
                .map(|w| format!("{}{}", "─".repeat(*w + 2), join))
                .collect::<String>()
                .trim_end_matches(join)
        )
    };
    let top = border("┌", "┬", "┐");
    let middle = border("├", "┼", "┤");
    let bottom = border("└", "┴", "┘");
    println!("{top}");
    println!(
        "│ {} │",
        headers
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{h:<width$}", width = widths[i]))
            .collect::<Vec<_>>()
            .join(" │ ")
    );
    println!("{middle}");
    for (row_index, (label, values, file_start)) in rows.iter().enumerate() {
        if separate_files && row_index > 0 && *file_start {
            println!("{middle}");
        }
        let mut cells = vec![truncate_left(label, widths[0])];
        for (i, (value, colour_code)) in values.iter().enumerate() {
            let padded = format!("{value:>width$}", width = widths[i + 1]);
            cells.push(if colour {
                format!("{colour_code}{padded}{RESET}")
            } else {
                padded
            });
        }
        while cells.len() < widths.len() {
            cells.push(" ".repeat(widths[cells.len()]));
        }
        println!("│ {} │", cells.join(" │ "));
    }
    println!("{bottom}");
}

/// Fits the label column around the space required by metric values.
fn column_widths(headers: &[String], rows: &[Row]) -> Vec<usize> {
    let mut widths: Vec<usize> = (0..headers.len())
        .map(|i| {
            let values = rows.iter().map(|r| {
                if i == 0 {
                    r.0.len()
                } else {
                    r.1.get(i - 1).map(|v| v.0.len()).unwrap_or(0)
                }
            });
            headers[i].len().max(values.max().unwrap_or(0))
        })
        .collect();
    const MAX_TABLE_WIDTH: usize = 78;
    let metric_width: usize = widths.iter().skip(1).sum();
    let label_limit = MAX_TABLE_WIDTH.saturating_sub(3 * headers.len() + metric_width + 1);
    widths[0] = widths[0].min(label_limit.max(3));
    widths
}
