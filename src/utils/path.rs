use anyhow::{anyhow, Result};
use path_clean::PathClean;
use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
};

use crate::p2s;

use super::{format_path, get_bare_apps, get_config, read_sub_dir};

pub fn split_parent(raw: &String, located: &String) -> (PathBuf, String) {
    // 解析为绝对路径
    let abs_path = parse_relative_path_with_located(raw, located);

    // 拿到 parent
    let parent = abs_path
        .parent()
        .unwrap_or_else(|| Path::new(located))
        .to_path_buf();

    // 拿到 base name
    let base = p2s!(abs_path.file_name().unwrap());

    (parent, base)
}

/// 使用配置文件中指定的 base 解析相对路径，不带路径格式化
pub fn parse_relative_path(relative: &String) -> Result<PathBuf> {
    let path = Path::new(relative);

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let cfg = get_config();
        let relative = Path::new(&cfg.local.base).join(relative);
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

/// 使用给定的 located 解析相对路径，带路径格式化
pub fn parse_relative_path_with_located(relative: &String, located: &String) -> PathBuf {
    let relative = format_path(relative);
    let located=format_path(located);
    let path = Path::new(&relative);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(&located).join(relative).to_path_buf()
    }
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
