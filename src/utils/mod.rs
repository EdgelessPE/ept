pub mod exe_version;
#[macro_use]
pub mod log;
pub mod arch;
pub mod cfg;
pub mod command;
pub mod conditions;
pub mod env;
pub mod fs;
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
use std::fs::{create_dir_all, remove_dir_all};
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
        crate::utils::format_path(&$x.to_string_lossy().to_string())
    };
}

fn ensure_exist(p: PathBuf) -> Result<PathBuf> {
    if !p.exists() {
        create_dir_all(p.clone()).map_err(|e| anyhow!("Error:Failed to create directory : {e}"))?;
    }
    Ok(p)
}

pub fn is_debug_mode() -> bool {
    envmnt::get_or("DEBUG", "false") == String::from("true")
}

pub fn is_qa_mode() -> bool {
    envmnt::get_or("QA", "false") == String::from("true")
}

pub fn is_confirm_mode() -> bool {
    envmnt::get_or("CONFIRM", "false") == String::from("true")
}

pub fn is_strict_mode() -> bool {
    envmnt::get_or("STRICT", "false") == String::from("true")
}

pub fn format_path(raw: &String) -> String {
    let tmp = raw.replace(r"\", "/");
    if tmp.starts_with("./") {
        tmp[2..].to_string()
    } else {
        tmp
    }
}

pub fn get_bare_apps() -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base(&"apps".to_string())?)
}

/// 不确保目录存在，可选确保 scope 目录存在
pub fn get_path_apps(scope: &String, name: &String, ensure_scope: bool) -> Result<PathBuf> {
    let scope_p = parse_relative_path_with_base(&"apps".to_string())?.join(scope);
    Ok(if ensure_scope {
        ensure_exist(scope_p)?
    } else {
        scope_p
    }
    .join(name))
}

pub fn parse_bare_temp() -> Result<PathBuf> {
    parse_relative_path_with_base(&"temp".to_string())
}

pub fn get_path_temp(name: &String, keep_clear: bool, sub_dir: bool) -> Result<PathBuf> {
    let random_name = name.to_owned() + "_" + &random_short_string();
    let p = parse_relative_path_with_base(&"temp".to_string())?.join(random_name);
    if keep_clear && p.exists() {
        remove_dir_all(p.clone()).map_err(|_| {
            anyhow!(
                "Error:Can't keep temp directory '{dir}' clear, manually delete it then try again",
                dir = p2s!(p.as_os_str())
            )
        })?;
    }
    if sub_dir {
        ensure_exist(p.join("Outer"))?;
        ensure_exist(p.join("Inner"))?;
    }
    ensure_exist(p)
}

pub fn get_path_bin() -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base(&"bin".to_string())?)
}

pub fn get_path_mirror() -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base(&"mirror".to_string())?)
}

pub fn get_system_drive() -> Result<String> {
    let root = var("SystemRoot")?;
    Ok(root[0..2].to_string())
}

pub fn is_url(text: &String) -> bool {
    URL_RE.is_match(text)
}

pub fn is_starts_with_inner_value(p: &String) -> bool {
    p.starts_with("${") || p.starts_with("\"${")
}

pub fn launch_clean() -> Result<()> {
    // 删除 temp 目录
    let p = parse_relative_path_with_base(&"temp".to_string())?;
    if p.exists() {
        try_recycle(p)?;
    }

    Ok(())
}

#[test]
fn test_get_system_drive() {
    assert_eq!(get_system_drive().unwrap(), "C:".to_string())
}
