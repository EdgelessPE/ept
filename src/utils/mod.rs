mod exe_version;
#[macro_use]
mod log;
mod cfg;
pub mod env;
mod fs;
mod path;
mod process;
mod term;
mod wild_match;

use anyhow::{anyhow, Result};
use regex::Regex;

pub use self::cfg::{get_config, set_config, Cfg, Local};
pub use self::exe_version::get_exe_version;
pub use self::fs::{count_sub_files, ensure_dir_exist, read_sub_dir, try_recycle};
pub use self::log::{fn_log, fn_log_ok_last};
pub use self::path::{
    find_scope_with_name_locally, parse_relative_path, parse_relative_path_with_located,
    split_parent,
};
pub use self::process::{is_alive_with_name, kill_with_name};
pub use self::term::ask_yn;
pub use self::wild_match::{
    common_wild_match_verify, contains_wild_match, is_valid_wild_match, parse_wild_match,
};

use std::fs::{create_dir_all, remove_dir_all};
use std::path::PathBuf;

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
    ensure_exist(parse_relative_path(&"apps".to_string())?)
}

/// 不确保目录存在，可选确保 scope 目录存在
pub fn get_path_apps(scope: &String, name: &String, ensure_scope: bool) -> Result<PathBuf> {
    let scope_p = parse_relative_path(&"apps".to_string())?.join(scope);
    Ok(if ensure_scope {
        ensure_exist(scope_p)?
    } else {
        scope_p
    }
    .join(name))
}

pub fn parse_bare_temp() -> Result<PathBuf> {
    parse_relative_path(&"temp".to_string())
}

pub fn get_path_temp(name: &String, keep_clear: bool, sub_dir: bool) -> Result<PathBuf> {
    let p = parse_relative_path(&"temp".to_string())?.join(name);
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
    ensure_exist(parse_relative_path(&"bin".to_string())?)
}

pub fn is_url(text: &String) -> bool {
    URL_RE.is_match(text)
}
