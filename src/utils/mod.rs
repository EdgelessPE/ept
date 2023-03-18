mod exe_version;
#[macro_use]
mod log;
mod path;
mod process;
mod term;

use anyhow::{anyhow,Result};

pub use self::exe_version::get_exe_version;
pub use self::log::{fn_log, fn_log_ok_last};
pub use self::path::{find_scope_with_name_locally, parse_relative_path, read_sub_dir};
pub use self::process::kill_with_name;
pub use self::term::ask_yn;

use std::fs::{create_dir_all, remove_dir_all};
use std::path::PathBuf;

#[macro_export]
macro_rules! p2s {
    ($x:expr) => {
        $x.to_string_lossy().to_string()
    };
}

fn ensure_exist(p: PathBuf) -> Result<PathBuf> {
    if !p.exists() {
        create_dir_all(p.clone())
            .map_err(|e| anyhow!("Error:Failed to create directory : {}", e.to_string()))?;
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

pub fn get_bare_apps() -> Result<PathBuf> {
    ensure_exist(parse_relative_path("apps".to_string())?)
}

pub fn get_path_apps(scope: &String, name: &String) -> Result<PathBuf> {
    ensure_exist(
        parse_relative_path("apps".to_string())?
            .join(scope)
            .join(name),
    )
}

pub fn get_path_temp(name: &String,keep_clear:bool,sub_dir:bool) -> Result<PathBuf> {
    let p=parse_relative_path("temp".to_string())?.join(name);
    if keep_clear&&p.exists() {
        remove_dir_all(p.clone())
        .map_err(|_|anyhow!("Error:Can't keep temp directory '{}' clear, manually delete it then try again",p2s!(p.as_os_str())))?;
    }
    if sub_dir{
        ensure_exist(p.join("Outer"))?;
        ensure_exist(p.join("Inner"))?;
    }
    ensure_exist(p)
}

pub fn get_path_bin() -> Result<PathBuf> {
    ensure_exist(parse_relative_path("bin".to_string())?)
}
