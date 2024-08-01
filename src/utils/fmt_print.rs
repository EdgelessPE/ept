use chrono::DateTime;
use colored::{ColoredString, Colorize};
use std::time::SystemTime;

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
        "  {:>20}/{:<50} {:<20} {}\n",
        scope.cyan().italic(),
        name.cyan().bold(),
        format!("({version})"),
        mirror.unwrap_or_default().as_str().truecolor(100, 100, 100)
    )
}

pub fn fmt_mirror_line(name: String, updated_at: SystemTime) -> String {
    let date_time: DateTime<chrono::Local> = updated_at.into();
    let time_str = date_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let update_str = format!("updated at {time_str}");
    format!(
        "  {name:<20} {str}\n",
        str = update_str.as_str().truecolor(100, 100, 100)
    )
}
