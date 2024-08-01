use chrono::DateTime;
use colored::{ColoredString, Colorize};
use std::time::SystemTime;

pub fn fmt_log(head: ColoredString, msg: &str) -> String {
    format!("{head:>8} {msg}")
}

pub fn fmt_log_in_step(step: &str, head: ColoredString, msg: &str) -> String {
    format!("{head:>8} {s:<9} {msg}", s = step.black())
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
        mirror.unwrap_or_default().as_str().black()
    )
}

pub fn fmt_mirror_line(name: String, updated_at: SystemTime) -> String {
    let date_time: DateTime<chrono::Local> = updated_at.into();
    let time_str = date_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let update_str = format!("updated at {time_str}");
    format!("  {name:<20} {str}\n", str = update_str.as_str().black())
}

#[test]
fn test_fmt() {
    println!("{}", fmt_log("Test".black(), "This is a fmt test message"));
    println!(
        "{}",
        fmt_log_in_step("Test", "Test".black(), "This is a fmt test message")
    );
    print!(
        "{}",
        fmt_package_line(
            "Scope".to_string(),
            "Name".to_string(),
            "1.1.4.5".to_string(),
            Some("mock-server".to_string()),
        )
    );
    print!(
        "{}",
        fmt_mirror_line("mock-server".to_string(), SystemTime::now())
    );
}
