//! Applies scoring switches and diagnoses unknown options.

use crate::report::{self, Options};

/// Applies a single flag, returning false for positional paths.
pub(super) fn flag(arg: &str, options: &mut Options) -> Result<bool, String> {
    match arg {
        "--detail" | "--functions" => options.detail = true,
        "--guide" => options.guide = true,
        "--all-metrics" => {
            options.metrics = report::parse_metrics("nmi,loc,docs,cc,cognitive,halstead")?
        }
        "--no-colour" | "--no-color" => options.colour = false,
        "--help" | "-h" => return Err(crate::cli::usage()),
        value if value.starts_with('-') => return Err(format!("unknown option: {value}")),
        _ => return Ok(false),
    }
    Ok(true)
}
