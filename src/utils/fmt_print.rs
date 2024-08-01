use colored::{ColoredString, Colorize};

pub fn fmt_log(head: ColoredString, msg: &str) -> String {
    format!("{head:>8} {msg}")
}

pub fn fmt_log_in_step(step: &str, head: ColoredString, msg: &str) -> String {
    format!("{head:>8} {s:<9} {msg}", s = step.truecolor(100, 100, 100))
}

pub fn fmt_package_line(
    scope: String,
    name: String,
    version: String,
    mirror: Option<String>,
) -> String {
    format!(
        "  {:>20}/{:<50}    {:<20}   {}\n",
        scope.cyan().italic(),
        name.cyan().bold(),
        format!("({version})"),
        mirror.unwrap_or_default().as_str().truecolor(100, 100, 100)
    )
}
