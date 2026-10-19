//! Maps metric values to the documented status bands.

use crate::table::{GREEN, RED, YELLOW};

/// Assigns status bands using inclusive caution and failure thresholds.
pub(super) fn rate(value: u32, yellow: u32, red: u32) -> &'static str {
    if value >= red {
        RED
    } else if value >= yellow {
        YELLOW
    } else {
        GREEN
    }
}
/// Marks whether a documentation description is present.
pub(super) fn doc_colour(has_docstring: bool) -> &'static str {
    if has_docstring {
        GREEN
    } else {
        RED
    }
}
