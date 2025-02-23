use anyhow::Result;
use chrono::DateTime;
use colored::{ColoredString, Colorize};
use std::{fmt::Display, time::SystemTime};


fn ellipsis(raw: &str, limit: usize) -> String {
    let len = raw.len();
    if len <= limit {
        return raw.to_string();
    }
    format!("{}...", &raw[0..limit - 3])
}

pub fn fmt_log(head: ColoredString, msg: &str) -> String {
    format!("{head:>8} {msg}")
}

pub fn fmt_log_in_step(step: &str, head: ColoredString, msg: &str) -> String {
    format!("{head:>8} {s:<9} {msg}", s = step.truecolor(100, 100, 100))
}

#[deprecated]
pub fn fmt_package_line(
    scope: &str,
    name: &str,
    version: &str,
    description: Option<String>,
) -> String {
    format!(
        "  {:>15}/{:<30} {:<22} {}\n",
        ellipsis(scope, 15).truecolor(100, 100, 100).italic(),
        ellipsis(name, 30).cyan().bold(),
        format!("({version})"),
        description
            .unwrap_or_default()
            .as_str()
            .truecolor(100, 100, 100)
    )
}

#[deprecated]
pub fn fmt_mirror_line(name: &str, updated_at: SystemTime) -> String {
    let date_time: DateTime<chrono::Local> = updated_at.into();
    let time_str = date_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let update_str = format!("updated at {time_str}");
    format!(
        "  {name:<20} {str}\n",
        str = update_str.as_str().truecolor(100, 100, 100)
    )
}

#[test]
fn test_ellipsis() {
    assert_eq!(ellipsis("VSCode", 10), "VSCode".to_string());
    assert_eq!(ellipsis("Visual Studio Code", 10), "Visual ...".to_string());
    assert_eq!(
        ellipsis("Visual Studio Code", 16),
        "Visual Studio...".to_string()
    );
}

#[test]
fn test_fmt() {
    println!("{}", fmt_log("Test".purple(), "This is a fmt test message"));
    println!(
        "{}",
        fmt_log_in_step("Test", "Test".purple(), "This is a fmt test message")
    );
    print!(
        "{}",
        fmt_package_line("Scope", "Name", "1.1.4.5", Some("mock-server".to_string()),)
    );
    print!(
        "{}",
        fmt_package_line(
            "Portable-Apps-Foundation",
            "Firefox-Special-Edition-For-Developers",
            "1145.1419.1981.0000",
            Some("mock-server".to_string()),
        )
    );
    print!("{}", fmt_mirror_line("mock-server", SystemTime::now()));
}

pub enum PackageSource {
    Mirror(String),
    LocalPath(String),
    Url(String),
}

impl Display for PackageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageSource::Mirror(mirror) => write!(f, "Mirror '{}'", mirror),
            PackageSource::LocalPath(path) => write!(f, "Path '{}'", path),
            PackageSource::Url(url) => write!(f, "URL '{}'", url),
        }
    }
}

pub enum FmtPrintCaller {
    Info,
    Install(PackageSource),
    Update(PackageSource),
    Uninstall,
}

pub trait FmtPrint {
    fn fmt_print(&self, fmt_caller: FmtPrintCaller) -> Result<String>;
}
