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
pub fn print(headers: &[&str], rows: &[Row], colour: bool, separate_files: bool) {
    let widths = column_widths(headers, rows);
    let border = |left: &str, join: &str, right: &str| {
        let lines: Vec<_> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
        format!("{left}{}{right}", lines.join(join))
    };
    let middle = border("├", "┼", "┤");
    println!("{}", border("┌", "┬", "┐"));
    let header: Vec<_> = (headers.iter().zip(&widths))
        .map(|(header, width)| format!("{header:<width$}"))
        .collect();
    println!("│ {} │", header.join(" │ "));
    println!("{middle}");
    for (row_index, row) in rows.iter().enumerate() {
        if separate_files && row_index > 0 && row.2 {
            println!("{middle}");
        }
        println!("│ {} │", cells(row, &widths, colour).join(" │ "));
    }
    println!("{}", border("└", "┴", "┘"));
}

/// Formats one row to the column widths, colouring metric values when asked.
fn cells((label, values, _): &Row, widths: &[usize], colour: bool) -> Vec<String> {
    let mut cells = vec![truncate_left(label, widths[0])];
    for ((value, colour_code), width) in values.iter().zip(&widths[1..]) {
        let padded = format!("{value:>width$}");
        cells.push(if colour {
            format!("{colour_code}{padded}{RESET}")
        } else {
            padded
        });
    }
    cells
}

/// Fits the label column around the space required by metric values.
fn column_widths(headers: &[&str], rows: &[Row]) -> Vec<usize> {
    let mut widths: Vec<usize> = headers.iter().map(|header| header.len()).collect();
    for (label, values, _) in rows {
        widths[0] = widths[0].max(label.chars().count());
        for (width, (value, _)) in widths[1..].iter_mut().zip(values) {
            *width = (*width).max(value.len());
        }
    }
    const MAX_TABLE_WIDTH: usize = 78;
    let metric_width: usize = widths.iter().skip(1).sum();
    let label_limit = MAX_TABLE_WIDTH.saturating_sub(3 * headers.len() + metric_width + 1);
    widths[0] = widths[0].min(label_limit.max(3));
    widths
}
