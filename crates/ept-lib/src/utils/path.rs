use anyhow::{anyhow, Result};
use path_clean::PathClean;
use std::path::{Path, PathBuf};

use crate::{p2s, types::cfg::Cfg};

use super::{
    format_path, fs::read_sub_dir, get_bare_apps, get_path_mirror, mirror::read_quick_maps,
};

pub fn split_parent(raw: &str, located: &str) -> (PathBuf, String) {
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

/// 使用给定的 base 解析相对路径
pub fn parse_relative_path_with_base(relative: &str, base: &str) -> Result<PathBuf> {
    let relative = format_path(relative);
    let path = Path::new(&relative);

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(base).join(&relative)
    }
    .clean();

    // log!(
    //     "Debug:Parse relative path '{relative}' into '{p}'",
    //     p = p2s!(absolute_path)
    // );
    Ok(absolute_path)
}

/// 使用给定的 located 解析相对路径
pub fn parse_relative_path_with_located(relative: &str, located: &str) -> PathBuf {
    // debug_assert!(Path::new(located).is_absolute());
    debug_assert!(located.is_empty() || Path::new(located).exists());

    let relative = format_path(relative);
    let located = format_path(located);
    let path = Path::new(&relative);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(&located).join(relative)
    }
}

/// name 大小写不敏感
fn find_scope_with_name_locally(
    cfg: &Cfg,
    name: &str,
    scope: Option<&str>,
) -> Result<(String, String)> {
    let app_dir = get_bare_apps(cfg)?;

    for scope_dir_name in read_sub_dir(app_dir.clone())? {
        if let Some(s) = scope {
            if scope_dir_name.to_lowercase() != s.to_lowercase() {
                continue;
            }
        }
        for dir_name in read_sub_dir(app_dir.join(&scope_dir_name))? {
            if dir_name.eq_ignore_ascii_case(name) {
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

fn find_scope_with_name_online(
    cfg: &Cfg,
    name: &str,
    scope: Option<&str>,
) -> Result<(String, String)> {
    // 遍历 mirrors
    let p = get_path_mirror(cfg)?;
    let mirror_names = read_sub_dir(p)?;
    if mirror_names.is_empty() {
        return Err(anyhow!("Error:No mirror added yet"));
    }
    for mirror_name in mirror_names {
        let quick_maps = read_quick_maps(cfg, &mirror_name)?;
        if let Some((possible_scopes, true_name)) = quick_maps.scope_map.get(&name.to_lowercase()) {
            if let Some(dirty_scope) = scope {
                for s in possible_scopes {
                    if s.to_lowercase() == dirty_scope.to_lowercase() {
                        return Ok((s.clone(), name.to_string()));
                    }
                }
            } else if possible_scopes.len() == 1 {
                return Ok((possible_scopes[0].clone(), true_name.clone()));
            } else {
                return Err(anyhow!("Error:Multiple scopes found for '{name}' : {}. Use explicit scope like '{}/{name}' to specify exact package",possible_scopes.join(","),possible_scopes[0]));
            }
        }
    }
    Err(if let Some(s) = scope {
        anyhow!("Error:Can't locate '{name}' with scope '{s}' online")
    } else {
        anyhow!("Error:Can't find scope for '{name}' online")
    })
}

pub fn find_scope_with_name(
    cfg: &Cfg,
    name: &str,
    scope: Option<&str>,
) -> Result<(String, String)> {
    if let Ok(res) = find_scope_with_name_locally(cfg, name, scope) {
        return Ok(res);
    }
    find_scope_with_name_online(cfg, name, scope)
}

#[test]
fn test_parse_relative_path() {
    use crate::utils::test::_default_test_cfg;
    let cfg = _default_test_cfg();
    let p1 = String::from("./VSCode/VSCode.exe");
    let p2 = String::from(r"D:\Desktop\Projects\") + "./code.exe";
    let p3 = p2s!(std::env::current_dir().unwrap().join("./code.exe"));

    println!("{:?}", parse_relative_path_with_base(&p1, &cfg.local.base));
    println!("{:?}", parse_relative_path_with_base(&p2, &cfg.local.base));
    println!("{:?}", parse_relative_path_with_base(&p3, &cfg.local.base));
}

#[test]
fn test_find_scope_with_name() {
    use crate::utils::flags::{set_flag, Flag};
    use crate::utils::test::_default_test_cfg;
    use crate::utils::test::{
        _ensure_testing_vscode, _mount_custom_mirror, _unmount_custom_mirror,
    };

    set_flag(Flag::Debug, true);
    set_flag(Flag::Confirm, true);
    _ensure_testing_vscode();
    let tup = _mount_custom_mirror();

    let cfg = _default_test_cfg();

    // 本地信息
    let name = String::from("vscode");
    let res = find_scope_with_name(&cfg, &name, None).unwrap();
    assert_eq!(res, ("Microsoft".to_string(), "VSCode".to_string()));

    // 在线信息
    let name = String::from("Notepad");
    let res = find_scope_with_name(&cfg, &name, None).unwrap();
    assert_eq!(res, ("Microsoft".to_string(), "Notepad".to_string()));

    // 别名
    let name = String::from("code");
    let res = find_scope_with_name(&cfg, &name, None).unwrap();
    assert_eq!(res, ("Microsoft".to_string(), "VSCode".to_string()));

    // 命名冲突
    assert!(find_scope_with_name(&cfg, "NameA", None).is_err());
    assert!(find_scope_with_name(&cfg, "NameA", Some("ScopeA")).is_ok());
    assert!(find_scope_with_name(&cfg, "NameA", Some("ScopeB")).is_ok());

    // 命名和别名冲突
    assert!(find_scope_with_name(&cfg, "NameB", None).is_err());
    assert!(find_scope_with_name(&cfg, "NameB", Some("ScopeA")).is_ok());
    assert!(find_scope_with_name(&cfg, "NameB", Some("ScopeB")).is_ok());

    _unmount_custom_mirror(tup);
}
