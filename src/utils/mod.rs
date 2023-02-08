mod exe_version;
mod log;

pub use self::exe_version::get_exe_version;
pub use self::log::{log, log_ok_last};

use anyhow::Result;
use path_clean::PathClean;
use std::env::current_dir;
use std::path::{Path, PathBuf};

pub fn is_debug_mode() -> bool {
    envmnt::get_or("DEBUG", "false") == String::from("true")
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

    log(format!(
        "Debug:Parse relative path '{}' into '{}'",
        &relative,
        &absolute_path.display()
    ));
    Ok(absolute_path)
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
