use std::path::Path;

use anyhow::{anyhow, Result};

use crate::{
    p2s,
    parsers::parse_package,
    types::{
        info::{Info, InfoDiff},
        package::GlobalPackage,
    },
    utils::{find_scope_with_name_locally, get_path_apps},
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
    let global = parse_package(&p2s!(pkg_path), Some(local_str))?;
    // 写本地信息
    let authors = global.package.authors.clone();
    let local = InfoDiff {
        version: global.package.version.clone(),
        authors,
    };
    Ok((global.clone(), local))
}

pub fn info(scope: Option<String>, package_name: &String) -> Result<Info> {
    // 查找 scope
    let scope = scope.unwrap_or(find_scope_with_name_locally(package_name)?);

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
    let local_path = get_path_apps(&scope, package_name, false)?;
    if local_path.exists() {
        let (global, local) = info_local(&scope, package_name)?;
        info.license = global.package.license;
        info.local = Some(local);
        info.software = global.software;
    }

    // 检查到底有没有这个包
    if info.local.is_some() || info.online.is_some() {
        Ok(info)
    } else {
        Err(anyhow!("Error:Unknown package '{package_name}'"))
    }
}

#[test]
fn test_info() {
    let res = info(Some("Microsoft".to_string()), &"VSCode".to_string()).unwrap();
    println!("{res:#?}");
    let res = info(None, &"vscode".to_string()).unwrap();
    println!("{res:#?}");
}
