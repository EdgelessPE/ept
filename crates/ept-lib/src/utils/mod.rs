pub mod exe_version;
#[macro_use]
pub mod log;
pub mod arch;
pub mod cache;
pub mod cfg;
pub mod command;
pub mod conditions;
pub mod constants;
pub mod download;
pub mod env;
pub mod expand;
pub mod flags;
pub mod fmt_print;
pub mod fs;
pub mod mirror;
pub mod parse_inputs;
pub mod path;
pub mod permissions;
pub mod process;
pub mod random;
pub mod reg_entry;
pub mod request;
pub mod term;
pub mod test;
pub mod upgrade;
pub mod wild_match;

use anyhow::{anyhow, Result};
use cache::clean_cache;
use flags::{get_flag, Flag};
use regex::Regex;

use std::env::var;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use self::path::parse_relative_path_with_base;
use self::random::random_short_string;
use crate::types::cfg::Cfg;

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
    get_flag(Flag::Debug, false)
}

pub fn is_confirm_mode(cfg: &Cfg) -> bool {
    cfg.interaction.auto_confirm_all
}

pub fn format_path(raw: &str) -> String {
    let tmp = raw.replace('\\', "/");
    tmp.strip_prefix("./").map(|s| s.to_string()).unwrap_or(tmp)
}

pub fn get_bare_apps(cfg: &Cfg) -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base("apps", &cfg.local.base)?)
}

/// 不确保目录存在，可选确保 scope 目录存在
pub fn get_path_apps(cfg: &Cfg, scope: &str, name: &str, ensure_scope: bool) -> Result<PathBuf> {
    let scope_p = parse_relative_path_with_base("apps", &cfg.local.base)?.join(scope);
    Ok(if ensure_scope {
        ensure_exist(scope_p)?
    } else {
        scope_p
    }
    .join(name))
}

pub fn parse_bare_temp(cfg: &Cfg) -> Result<PathBuf> {
    parse_relative_path_with_base("temp", &cfg.local.base)
}

pub fn allocate_path_temp(cfg: &Cfg, name: &str, sub_dir: bool) -> Result<PathBuf> {
    let random_name = name.to_owned() + "_" + &random_short_string();
    let p = parse_relative_path_with_base("temp", &cfg.local.base)?.join(random_name);
    if sub_dir {
        ensure_exist(p.join("Outer"))?;
        ensure_exist(p.join("Inner"))?;
    }
    ensure_exist(p)
}

pub fn get_path_bin(cfg: &Cfg) -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base("bin", &cfg.local.base)?)
}

pub fn get_path_mirror(cfg: &Cfg) -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base("mirror", &cfg.local.base)?)
}

pub fn get_path_cache(cfg: &Cfg) -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base("cache", &cfg.local.base)?)
}

pub fn get_path_meta(cfg: &Cfg) -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base("meta", &cfg.local.base)?)
}

pub fn get_path_toolchain(cfg: &Cfg) -> Result<PathBuf> {
    ensure_exist(parse_relative_path_with_base("toolchain", &cfg.local.base)?)
}

pub fn get_system_drive() -> Result<String> {
    let root = var("SystemRoot")?;
    root.get(0..2)
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Error:SystemRoot environment variable is too short"))
}

pub fn is_url(text: &str) -> bool {
    URL_RE.is_match(text)
}

pub fn is_starts_with_inner_value(p: &str) -> bool {
    p.starts_with("${") || p.starts_with("\"${")
}

pub fn launch_clean(cfg: &Cfg) -> Result<()> {
    // 删除 temp 目录
    let p = parse_bare_temp(cfg)?;
    if p.exists() {
        std::fs::remove_dir_all(p)
            .map_err(|e| anyhow!("Error:Failed to remove temp directory : {e}"))?;
    }

    // 清理过期缓存
    clean_cache(cfg)?;

    Ok(())
}

use crate::types::constants::{DIR_NEP_CONTEXT, DIR_WORKFLOWS, FILE_PACKAGE};

pub fn get_manifest_path(located: &str) -> Result<PathBuf> {
    let possible_path = vec![
        format!("{located}/{}", FILE_PACKAGE),
        format!("{located}/{}/{}", DIR_NEP_CONTEXT, FILE_PACKAGE),
    ];

    for p in possible_path {
        let p = Path::new(&p);
        if p.exists() {
            return Ok(p.to_path_buf());
        }
    }
    Err(anyhow!(
        "Error:Failed to find '{}' in {located}",
        FILE_PACKAGE
    ))
}

pub fn get_workflows_path(located: &str) -> Result<PathBuf> {
    let possible_path = vec![
        format!("{located}/{}", DIR_WORKFLOWS),
        format!("{located}/{}/{}", DIR_NEP_CONTEXT, DIR_WORKFLOWS),
    ];

    for p in possible_path {
        let p = Path::new(&p);
        if p.exists() {
            return Ok(p.to_path_buf());
        }
    }
    Err(anyhow!(
        "Error:Failed to find workflows directory in {located}"
    ))
}

#[test]
fn test_get_system_drive() {
    assert_eq!(get_system_drive().unwrap(), "C:".to_string())
}
