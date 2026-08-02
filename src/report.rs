use crate::metrics::{FileMetrics, FnMetrics};
use crate::table::{self, GREEN, RED, YELLOW};

fn rate_u32(v: u32, yellow: u32, red: u32) -> &'static str {
    if v >= red {
        RED
    } else if v >= yellow {
        YELLOW
    } else {
        GREEN
    }
}

fn rate_nmi(v: f64) -> &'static str {
    if v < 10.0 {
        RED
    } else if v < 20.0 {
        YELLOW
    } else {
        GREEN
    }
}

fn worst(colours: &[&'static str]) -> &'static str {
    colours
        .iter()
        .copied()
        .max_by_key(|&c| {
            if c == RED {
                2
            } else if c == YELLOW {
                1
            } else {
                0
            }
        })
        .unwrap_or(GREEN)
}

pub fn file_row(f: &FileMetrics) {
    // File status is driven by NMI alone — CC, Cog, HV are inputs to NMI, not independent signals.
    let col = rate_nmi(f.nmi);
    let label = format!("{} [{}]", f.path, f.language);
    table::row(
        &label,
        col,
        &f.loc.to_string(),
        &f.cc.to_string(),
        &f.cognitive.to_string(),
        &format!("{:.0}", f.halstead),
        &format!("{:.1}", f.nmi),
    );
}

pub fn fn_row(m: &FnMetrics) {
    let col = worst(&[
        rate_u32(m.loc as u32, 30, 50),
        rate_u32(m.cc, 10, 15),
        rate_u32(m.cognitive, 10, 15),
        rate_u32(m.halstead as u32, 500, 1001),
    ]);
    let label = format!("{} (L{}–{})", m.name, m.line_start, m.line_end);
    table::row(
        &label,
        col,
        &m.loc.to_string(),
        &m.cc.to_string(),
        &m.cognitive.to_string(),
        &format!("{:.0}", m.halstead),
        "─",
    );
}

pub fn print_table(files: &[FileMetrics]) {
    use crate::table::{CC_W, COG_W, HV_W, LABEL_W, LOC_W, NMI_W, STATUS_W};
    println!("{}", table::top());
    println!(
        "│ {:<LABEL_W$} │ {:<STATUS_W$} │ {:>LOC_W$} │ {:>CC_W$} │ {:>COG_W$} │ {:>HV_W$} │ {:>NMI_W$} │",
        "Label", "Status", "LoC", "CC", "Cog", "HV", "NMI",
    );
    for (fi, f) in files.iter().enumerate() {
        if fi == 0 {
            println!("{}", table::mid());
        }
        file_row(f);
        for m in &f.fns {
            println!("{}", table::mid());
            fn_row(m);
        }
        if fi < files.len() - 1 {
            println!("{}", table::dbl());
        } else {
            println!("{}", table::bot());
        }
    }
}
