use anyhow::{anyhow, Result};
use path_clean::PathClean;
use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
};

use crate::p2s;

use super::{get_bare_apps, get_config, read_sub_dir};

pub fn parse_relative_path(relative: &String) -> Result<PathBuf> {
    let path = Path::new(&relative);

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let cfg = get_config();
        let relative = Path::new(&cfg.local.base).join(path);
        let dirty_abs = p2s!(canonicalize(relative)?);
        Path::new(&dirty_abs[4..]).to_path_buf()
    }
    .clean();

    log!(
        "Debug:Parse relative path '{relative}' into '{p}'",
        p = p2s!(absolute_path)
    );
    Ok(absolute_path)
}

/// name 大小写不敏感
pub fn find_scope_with_name_locally(name: &String) -> Result<String> {
    let app_dir = get_bare_apps()?;
    for scope in read_sub_dir(app_dir.clone())? {
        for dir_name in read_sub_dir(app_dir.join(&scope))? {
            if dir_name.to_ascii_lowercase() == name.to_ascii_lowercase() {
                return Ok(scope);
            }
        }
    }
    Err(anyhow!("Error:Can't find scope for '{name}'"))
}

#[test]
fn test_parse_relative_path() {
    let p1 = String::from("./VSCode/VSCode.exe");
    let p2 = String::from(r"D:\Desktop\Projects\") + "./code.exe";
    let p3 = p2s!(std::env::current_dir().unwrap().join("./code.exe"));

    println!("{:?}", parse_relative_path(&p1));
    println!("{:?}", parse_relative_path(&p2));
    println!("{:?}", parse_relative_path(&p3));
}
