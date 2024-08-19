pub mod exe_version;
#[macro_use]
pub mod log;
pub mod arch;
pub mod cfg;
pub mod command;
pub mod conditions;
pub mod download;
pub mod env;
pub mod fmt_print;
pub mod fs;
pub mod mirror;
pub mod parse_inputs;
pub mod path;
pub mod process;
pub mod random;
pub mod reg_entry;
pub mod term;
pub mod test;
pub mod wild_match;

use anyhow::{anyhow, Result};
use regex::Regex;

use std::env::var;
use std::fs::create_dir_all;
use std::path::PathBuf;

use self::fs::try_recycle;
use self::path::parse_relative_path_with_base;
use self::random::random_short_string;

lazy_static! {
    static ref URL_RE: Regex = Regex::new(r"^https?://").unwrap();
}

#[macro_export]
macro_rules! p2s {
    ($x:expr) => {
        $crate::utils::format_path(&$x.to_string_lossy().to_string())
    };
}

fn ensure_exist(p: PathBuf) -> Result<PathBuf> {
    if !p.exists() {
        create_dir_all(p.clone()).map_err(|e| anyhow!("Error:Failed to create directory : {e}"))?;
    }
    Ok(p)
}

pub fn is_debug_mode() -> bool {
    envmnt::get_or("DEBUG", "false") == *"true"
}

pub fn is_qa_mode() -> bool {
    envmnt::get_or("QA", "false") == *"true"
}

pub fn is_confirm_mode() -> bool {
    envmnt::get_or("CONFIRM", "false") == *"true"
}

pub fn is_strict_mode() -> bool {
    envmnt::get_or("STRICT", "false") == *"true"
}

pub fn is_no_warning_mode() -> bool {
    envmnt::get_or("NO_WARNING", "false") == *"true"
}

pub fn format_path(raw: &str) -> String {
    let tmp = raw.replace('\\', "/");
    tmp.strip_prefix("./").map(|s| s.to_string()).unwrap_or(tmp)
}

pub fn get_bare_apps() -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base("apps")?)
}

/// 不确保目录存在，可选确保 scope 目录存在
pub fn get_path_apps(scope: &String, name: &String, ensure_scope: bool) -> Result<PathBuf> {
    let scope_p = parse_relative_path_with_base("apps")?.join(scope);
    Ok(if ensure_scope {
        ensure_exist(scope_p)?
    } else {
        scope_p
    }
    .join(name))
}

pub fn parse_bare_temp() -> Result<PathBuf> {
    parse_relative_path_with_base("temp")
}

pub fn allocate_path_temp(name: &String, sub_dir: bool) -> Result<PathBuf> {
    let random_name = name.to_owned() + "_" + &random_short_string();
    let p = parse_relative_path_with_base("temp")?.join(random_name);
    if sub_dir {
        ensure_exist(p.join("Outer"))?;
        ensure_exist(p.join("Inner"))?;
    }
    ensure_exist(p)
}

pub fn get_path_bin() -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base("bin")?)
}

pub fn get_path_mirror() -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base("mirror")?)
}

pub fn get_system_drive() -> Result<String> {
    let root = var("SystemRoot")?;
    Ok(root[0..2].to_string())
}

pub fn is_url(text: &str) -> bool {
    URL_RE.is_match(text)
}

pub fn is_starts_with_inner_value(p: &str) -> bool {
    p.starts_with("${") || p.starts_with("\"${")
}

pub fn launch_clean() -> Result<()> {
    // 删除 temp 目录
    let p = parse_bare_temp()?;
    if p.exists() {
        try_recycle(p)?;
    }

    Ok(())
}

#[test]
fn test_get_system_drive() {
    assert_eq!(get_system_drive().unwrap(), "C:".to_string())
}
