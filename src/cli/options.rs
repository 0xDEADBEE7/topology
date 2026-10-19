//! Parses scoring flags independently of file assessment.

mod flags;
use crate::report::{self, Options};
use flags::flag;

/// Separates scoring options from input paths.
pub(super) fn parse(args: &[String]) -> Result<(Options, Vec<String>), String> {
    let mut options = Options::default();
    let mut paths = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--metrics" {
            let value = args.next().ok_or("--metrics requires a value")?;
            options.metrics = report::parse_metrics(value)?;
        } else if !flag(arg, &mut options)? {
            paths.push(arg.clone());
        }
    }
    if paths.is_empty() {
        return Err(super::usage());
    }
    Ok((options, paths))
}
