mod exe_version;
#[macro_use]
mod log;
mod path;
mod process;
mod term;

pub use self::exe_version::get_exe_version;
pub use self::log::{fn_log, fn_log_ok_last};
pub use self::path::{find_scope_with_name_locally, parse_relative_path, read_sub_dir};
pub use self::process::kill_with_name;
pub use self::term::ask_yn;

use std::path::PathBuf;

#[macro_export]
macro_rules! p2s {
    ($x:expr) => {
        $x.to_string_lossy().to_string()
    };
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

pub fn get_bare_apps() -> PathBuf {
    parse_relative_path("apps".to_string()).unwrap()
}

pub fn get_path_apps(scope: &String, name: &String) -> PathBuf {
    parse_relative_path("apps".to_string())
        .unwrap()
        .join(scope)
        .join(name)
}

pub fn get_path_temp(name: &String) -> PathBuf {
    parse_relative_path("temp".to_string()).unwrap().join(name)
}

pub fn get_path_bin() -> PathBuf {
    parse_relative_path("bin".to_string()).unwrap()
}
