// Column widths — total line width = 78 columns
pub const LABEL_W:  usize = 25;
pub const STATUS_W: usize = 6;
pub const LOC_W:    usize = 5;
pub const CC_W:     usize = 4;
pub const COG_W:    usize = 4;
pub const HV_W:     usize = 6;
pub const NMI_W:    usize = 6;

pub const RED: &str    = "\x1b[0;31m";
pub const YELLOW: &str = "\x1b[1;33m";
pub const GREEN: &str  = "\x1b[0;32m";
pub const RESET: &str  = "\x1b[0m";

pub fn status(colour: &str) -> &str {
    match colour {
        c if c == RED    => "RED   ",
        c if c == YELLOW => "YELLOW",
        _                => "GREEN ",
    }
}

pub fn truncate_left(s: &str, width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= width {
        format!("{s:<width$}")
    } else {
        let tail: String = chars[chars.len() - (width - 3)..].iter().collect();
        format!("...{tail}")
    }
}

pub fn top() -> String {
    format!("┌{:─<w$}┬{:─<s$}┬{:─<l$}┬{:─<c$}┬{:─<o$}┬{:─<h$}┬{:─<n$}┐",
        "", "", "", "", "", "", "",
        w=LABEL_W+2, s=STATUS_W+2, l=LOC_W+2, c=CC_W+2, o=COG_W+2, h=HV_W+2, n=NMI_W+2)
}

pub fn mid() -> String {
    format!("├{:─<w$}┼{:─<s$}┼{:─<l$}┼{:─<c$}┼{:─<o$}┼{:─<h$}┼{:─<n$}┤",
        "", "", "", "", "", "", "",
        w=LABEL_W+2, s=STATUS_W+2, l=LOC_W+2, c=CC_W+2, o=COG_W+2, h=HV_W+2, n=NMI_W+2)
}

pub fn dbl() -> String {
    format!("╞{:═<w$}╡", "", w = 76)
}

pub fn bot() -> String {
    format!("└{:─<w$}┴{:─<s$}┴{:─<l$}┴{:─<c$}┴{:─<o$}┴{:─<h$}┴{:─<n$}┘",
        "", "", "", "", "", "", "",
        w=LABEL_W+2, s=STATUS_W+2, l=LOC_W+2, c=CC_W+2, o=COG_W+2, h=HV_W+2, n=NMI_W+2)
}

pub fn row(label: &str, status_col: &str, loc: &str, cc: &str, cog: &str, hv: &str, nmi: &str) {
    let lbl = truncate_left(label, LABEL_W);
    println!(
        "│ {lbl} │ {sc}{st}{RESET} │ {loc:>LOW$} │ {cc:>CCW$} │ {cog:>CGW$} │ {hv:>HVW$} │ {nmi:>NMW$} │",
        sc=status_col, st=status(status_col),
        LOW=LOC_W, CCW=CC_W, CGW=COG_W, HVW=HV_W, NMW=NMI_W,
    );
}
