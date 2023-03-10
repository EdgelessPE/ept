mod exe_version;
#[macro_use]
mod log;
mod term;
mod process;

pub use self::exe_version::get_exe_version;
pub use self::log::{fn_log, fn_log_ok_last};
pub use self::term::ask_yn;
pub use self::process::kill_with_name;

use anyhow::Result;
use path_clean::PathClean;
use std::env::current_dir;
use std::path::{Path, PathBuf};

pub fn is_debug_mode() -> bool {
    envmnt::get_or("DEBUG", "false") == String::from("true")
}

pub fn is_confirm_mode() -> bool {
    envmnt::get_or("CONFIRM", "false") == String::from("true")
}

pub fn is_strict_mode() -> bool {
    envmnt::get_or("STRICT", "false") == String::from("true")
}

pub fn parse_relative_path(relative: String) -> Result<PathBuf> {
    let cr = relative.replace("./", "");
    let path = Path::new(&cr);

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir()?.join(path)
    }
    .clean();

    log!(
        "Debug:Parse relative path '{}' into '{}'",
        &relative,
        &absolute_path.display()
    );
    Ok(absolute_path)
}

pub fn get_path_apps() -> PathBuf {
    parse_relative_path("apps".to_string()).unwrap()
}

pub fn get_path_temp() -> PathBuf {
    parse_relative_path("temp".to_string()).unwrap()
}

pub fn get_path_bin() -> PathBuf {
    parse_relative_path("bin".to_string()).unwrap()
}

#[macro_export]
macro_rules! p2s {
    ($x:expr) => {
        $x.to_string_lossy().to_string()
    };
}

#[test]
fn test_parse_relative_path() {
    let p1 = String::from("./VSCode/VSCode.exe");
    let p2 = String::from(r"D:\Desktop\Projects\") + "./code.exe";
    let p3 = current_dir()
        .unwrap()
        .join("./code.exe")
        .to_string_lossy()
        .to_string();

    println!("{:?}", parse_relative_path(p1));
    println!("{:?}", parse_relative_path(p2));
    println!("{:?}", parse_relative_path(p3));
}
