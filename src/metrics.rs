//! Computes file and function health metrics from raw syntax counts.

use crate::visit::RawCounts;

#[derive(Debug)]
pub struct FnMetrics {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub loc: usize,
    pub cc: u32,
    pub cognitive: u32,
    pub halstead: f64,
    pub has_docstring: bool,
}

#[derive(Debug)]
pub struct ClassMetrics {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
}

pub struct FileMetrics {
    pub language: &'static str,
    pub path: String,
    pub loc: usize,
    pub cc: u32,
    pub cognitive: u32,
    pub halstead: f64,
    pub nmi: f64,
    pub has_docstring: bool,
    pub fns: Vec<FnMetrics>,
    pub classes: Vec<ClassMetrics>,
}

fn halstead_volume(n1: u32, n2: u32, dn1: u32, dn2: u32) -> f64 {
    let big_n = (n1 + n2) as f64;
    let small_n = (dn1 + dn2).max(2) as f64;
    big_n * small_n.log2()
}

/// Compute function-level complexity metrics from raw visitor counts.
pub fn compute(r: &RawCounts) -> FnMetrics {
    let loc = r.line_end.saturating_sub(r.line_start) + 1;
    let halstead = halstead_volume(r.n1, r.n2, r.dn1, r.dn2);
    FnMetrics {
        name: r.name.clone(),
        line_start: r.line_start,
        line_end: r.line_end,
        loc,
        cc: r.decisions,
        cognitive: r.cognitive,
        halstead,
        has_docstring: r.has_docstring,
    }
}

/// Combine per-function measurements into a file-level health summary.
pub fn aggregate(
    language: &'static str,
    path: &str,
    raw: &[RawCounts],
    fns: Vec<FnMetrics>,
    classes: Vec<ClassMetrics>,
    file_has_docstring: bool,
    file_loc: usize,
) -> FileMetrics {
    let n = fns.len() as u32;

    // File CC = sum(fn CCs) - (N - 1): removes per-function base paths, keeps one for the file.
    let cc: u32 = fns
        .iter()
        .map(|f| f.cc)
        .sum::<u32>()
        .saturating_sub(n.saturating_sub(1));

    // Cognitive: informational total only — no standard file-level formula exists.
    let cognitive: u32 = fns.iter().map(|f| f.cognitive).sum();

    // HV: computed from file-level totals, not summed sub-volumes.
    let n1: u32 = raw.iter().map(|r| r.n1).sum();
    let n2: u32 = raw.iter().map(|r| r.n2).sum();
    let dn1: u32 = raw.iter().map(|r| r.dn1).sum();
    let dn2: u32 = raw.iter().map(|r| r.dn2).sum();
    let halstead = halstead_volume(n1, n2, dn1, dn2);

    // NMI is the sole file-level health indicator — derived from HV, CC, and LoC.
    let nmi = {
        let mi = 171.0
            - 5.2 * halstead.max(1.0).ln()
            - 0.23 * cc as f64
            - 16.2 * (file_loc.max(1) as f64).ln();
        (mi * 100.0 / 171.0).clamp(0.0, 100.0)
    };
    FileMetrics {
        language,
        path: path.to_owned(),
        loc: file_loc,
        cc,
        cognitive,
        halstead,
        nmi,
        has_docstring: file_has_docstring,
        fns,
        classes,
    }
}
