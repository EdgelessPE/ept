use anyhow::{anyhow, Result};
use path_clean::PathClean;
use std::path::{Path, PathBuf};

use crate::{p2s, utils::cfg::get_config};

use super::{
    format_path, fs::read_sub_dir, get_bare_apps, get_path_mirror,
    mirror::read_local_mirror_pkg_software,
};

pub fn split_parent(raw: &str, located: &String) -> (PathBuf, String) {
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

/// 使用配置文件中指定的 base 解析相对路径
pub fn parse_relative_path_with_base(relative: &str) -> Result<PathBuf> {
    let relative = format_path(relative);
    let path = Path::new(&relative);

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let cfg = get_config();
        Path::new(&cfg.local.base).join(&relative)
    }
    .clean();

    log!(
        "Debug:Parse relative path '{relative}' into '{p}'",
        p = p2s!(absolute_path)
    );
    Ok(absolute_path)
}

/// 使用给定的 located 解析相对路径
pub fn parse_relative_path_with_located(relative: &str, located: &String) -> PathBuf {
    // debug_assert!(Path::new(located).is_absolute());
    debug_assert!(Path::new(located).exists());

    let relative = format_path(relative);
    let located = format_path(located);
    let path = Path::new(&relative);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(&located).join(relative).to_path_buf()
    }
}

/// name 大小写不敏感
fn find_scope_with_name_locally(name: &String, scope: Option<String>) -> Result<(String, String)> {
    let scope_input_str = scope.clone().unwrap_or("".to_string());
    let app_dir = get_bare_apps()?;

    for scope_dir_name in read_sub_dir(app_dir.clone())? {
        if scope.is_some() && scope_dir_name.to_lowercase() != scope_input_str.to_lowercase() {
            continue;
        }
        for dir_name in read_sub_dir(app_dir.join(&scope_dir_name))? {
            if dir_name.to_ascii_lowercase() == name.to_ascii_lowercase() {
                return Ok((scope_dir_name, dir_name));
            }
        }
    }

    Err(if let Some(s) = scope {
        anyhow!("Error:Can't locate '{name}' with scope '{s}' locally")
    } else {
        anyhow!("Error:Can't find scope for '{name}' locally")
    })
}

fn find_scope_with_name_online(name: &String, scope: Option<String>) -> Result<(String, String)> {
    let scope_input_str = scope.clone().unwrap_or("".to_string());
    // 遍历 mirrors
    let p = get_path_mirror()?;
    let mirror_names = read_sub_dir(p)?;
    for mirror_name in mirror_names {
        let pkg_software = read_local_mirror_pkg_software(&mirror_name)?;
        for (scope_real_name, tree) in pkg_software.tree {
            if scope.is_some() && scope_real_name.to_lowercase() != scope_input_str.to_lowercase() {
                continue;
            }
            for node in tree {
                if node.name.to_lowercase() == name.to_lowercase() {
                    return Ok((scope_real_name, node.name));
                }
            }
        }
    }
    Err(if let Some(s) = scope {
        anyhow!("Error:Can't locate '{name}' with scope '{s}' online")
    } else {
        anyhow!("Error:Can't find scope for '{name}' online")
    })
}

pub fn find_scope_with_name(name: &String, scope: Option<String>) -> Result<(String, String)> {
    let local_res = find_scope_with_name_locally(name, scope.clone());
    if local_res.is_ok() {
        return local_res;
    }
    find_scope_with_name_online(name, scope)
}

#[test]
fn test_parse_relative_path() {
    let p1 = String::from("./VSCode/VSCode.exe");
    let p2 = String::from(r"D:\Desktop\Projects\") + "./code.exe";
    let p3 = p2s!(std::env::current_dir().unwrap().join("./code.exe"));

    println!("{:?}", parse_relative_path_with_base(&p1));
    println!("{:?}", parse_relative_path_with_base(&p2));
    println!("{:?}", parse_relative_path_with_base(&p3));
}
