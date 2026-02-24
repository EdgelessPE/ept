use anyhow::Result;
use chrono::DateTime;
use colored::{ColoredString, Colorize};
use std::fmt::Display;

use crate::types::mirror::MirrorInfo;

use super::parse_inputs::ParseInputResEnum;

fn _ellipsis(raw: &str, limit: usize) -> String {
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

#[test]
fn test_ellipsis() {
    assert_eq!(_ellipsis("VSCode", 10), "VSCode".to_string());
    assert_eq!(
        _ellipsis("Visual Studio Code", 10),
        "Visual ...".to_string()
    );
    assert_eq!(
        _ellipsis("Visual Studio Code", 16),
        "Visual Studio...".to_string()
    );
}

pub enum PackageSource {
    Mirror(String),
    LocalPath(String),
    Url(String),
}

impl From<ParseInputResEnum> for PackageSource {
    fn from(value: ParseInputResEnum) -> Self {
        match value {
            ParseInputResEnum::Url(url, _) => PackageSource::Url(url),
            ParseInputResEnum::LocalPath(path, _) => PackageSource::LocalPath(path),
            ParseInputResEnum::PackageMatcher(res) => PackageSource::Mirror(res.mirror),
        }
    }
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
    fn fmt_print(&self, fmt_caller: FmtPrintCaller, show_emojis: bool) -> Result<String>;
    fn fmt_brief_print(&self, fmt_caller: FmtPrintCaller) -> Result<String>;
}

pub fn fmt_print_mirror_line(mirror_info: MirrorInfo) -> String {
    let date_time: DateTime<chrono::Local> = mirror_info.updated_at.into();
    let time_str = date_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let root_str = format!("Root URL: '{}'", mirror_info.root_url);
    let update_str = format!("Last Update: {time_str}");
    format!(
        "· {}\n  {}\n  {}\n",
        mirror_info.name.bold(),
        root_str.as_str().truecolor(100, 100, 100),
        update_str.as_str().truecolor(100, 100, 100)
    )
}

#[test]
fn test_fmt_log() {
    use colored::Colorize;
    let result = fmt_log("Info".blue(), "test message");
    assert!(result.contains("test message"));
}

#[test]
fn test_fmt_log_in_step() {
    use colored::Colorize;
    let result = fmt_log_in_step("Copy", "Info".blue(), "test message");
    assert!(result.contains("test message"));
    assert!(result.contains("Copy"));
}

#[test]
fn test_package_source_display() {
    assert_eq!(
        PackageSource::Mirror("test".to_string()).to_string(),
        "Mirror 'test'"
    );
    assert_eq!(
        PackageSource::LocalPath("path".to_string()).to_string(),
        "Path 'path'"
    );
    assert_eq!(
        PackageSource::Url("http://test.com".to_string()).to_string(),
        "URL 'http://test.com'"
    );
}
