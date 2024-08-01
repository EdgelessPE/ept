use colored::{ColoredString, Colorize};

pub fn fmt_log(head: ColoredString, msg: &str) -> String {
    format!("{head:>8} {msg}")
}

pub fn fmt_log_in_step(step: &str, head: ColoredString, msg: &str) -> String {
    format!("{head:>8} {s:<9} {msg}", s = step.truecolor(100, 100, 100))
}
