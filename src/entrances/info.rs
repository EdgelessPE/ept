use std::path::Path;

use anyhow::{anyhow, Result};

use crate::{
    p2s,
    parsers::parse_package,
    types::{
        info::{Info, InfoDiff},
        mirror::TreeItem,
        package::GlobalPackage,
    },
    utils::{
        fs::read_sub_dir,
        get_path_apps, get_path_mirror,
        mirror::{filter_release, read_local_mirror_pkg_software},
        path::find_scope_with_name,
    },
};

use super::utils::validator::installed_validator;

pub fn info_local(scope: &String, package_name: &String) -> Result<(GlobalPackage, InfoDiff)> {
    let local_path = get_path_apps(scope, package_name, false)?;
    if !local_path.exists() {
        return Err(anyhow!("Error:Can't find package '{package_name}' locally"));
    }
    let local_str = p2s!(local_path);
    // 检查是否为标准的已安装目录
    let ctx_str = installed_validator(&local_str)?;
    let ctx_path = Path::new(&ctx_str);
    // 读入包信息
    let pkg_path = ctx_path.join("package.toml");
    let global = parse_package(&p2s!(pkg_path), &local_str, true)?;
    // 写本地信息
    let authors = global.package.authors.clone();
    let local = InfoDiff {
        version: global.package.version.clone(),
        authors,
    };
    Ok((global.clone(), local))
}

// 第二个参数为 URL 模板
pub fn info_online(
    scope: &String,
    package_name: &String,
    mirror: Option<String>,
) -> Result<(TreeItem, String)> {
    // 定义匹配函数
    let item_matcher = |mirror_name: &String| {
        let pkg_software = read_local_mirror_pkg_software(mirror_name)?;
        if let Some(entry) = pkg_software.tree.get(scope) {
            for item in entry {
                if &item.name == package_name {
                    return Ok((item.to_owned(), pkg_software.url_template));
                }
            }
        }
        Err(anyhow!(
            "Error:Can't find such package in mirror '{mirror_name}'"
        ))
    };
    if let Some(mirror_name) = mirror {
        return item_matcher(&mirror_name);
    } else {
        // 遍历 mirror 目录，读出软件包树并进行查找
        let p = get_path_mirror()?;
        let mirror_names = read_sub_dir(p)?;
        for name in mirror_names {
            if let Ok(res) = item_matcher(&name) {
                return Ok(res);
            }
        }
    }

    Err(anyhow!(
        "Error:Package '{package_name}' in scope '{scope}' not found"
    ))
}

pub fn info(scope: Option<String>, package_name: &String) -> Result<Info> {
    // 查找 scope 并使用 scope 更新纠正大小写
    let (scope, package_name) = find_scope_with_name(package_name, scope)?;

    // 创建结果结构体
    let mut info = Info {
        name: package_name.clone(),
        template: String::from("Software"),
        license: None,
        local: None,
        online: None,
        software: None,
    };

    // 扫描本地安装目录
    let local_path = get_path_apps(&scope, &package_name, false)?;
    if local_path.exists() {
        let (global, local) = info_local(&scope, &package_name)?;
        info.license = global.package.license;
        info.local = Some(local);
        info.software = global.software;
    }

    // 在线检查
    if let Ok((item, _)) = info_online(&scope, &package_name, None) {
        let latest = filter_release(item.releases, None, false)?;
        info.online = Some(InfoDiff {
            version: latest.version.to_string(),
            authors: Vec::new(),
        })
    }

    // 检查到底有没有这个包
    if info.local.is_some() || info.online.is_some() {
        Ok(info)
    } else {
        Err(anyhow!("Error:Unknown package '{package_name}'"))
    }
}

// #[test]
// fn test_info() {
// use crate::utils::test::_ensure_testing_vscode,
// _ensure_testing_vscode();
// let res = info(Some("Microsoft".to_string()), &"VSCode".to_string()).unwrap();
// println!("{res:#?}");
// let res = info(None, &"vscode".to_string()).unwrap();
// println!("{res:#?}");
// }
