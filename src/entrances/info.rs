use std::path::Path;

use anyhow::{anyhow, Result};

use crate::{
    parsers::parse_package,
    types::{GlobalPackage, Info, InfoDiff},
};

use super::validator::installed_validator;

pub fn info_local(package_name: String) -> Result<(GlobalPackage, InfoDiff)> {
    let local_path = Path::new("./apps").join(&package_name);
    if !local_path.exists() {
        return Err(anyhow!(
            "Error:Can't find package '{}' locally",
            package_name
        ));
    }
    let local_str = local_path.to_string_lossy().to_string();
    // 检查是否为标准的已安装目录
    let ctx_str = installed_validator(local_str.clone())?;
    let ctx_path = Path::new(&ctx_str);
    // 读入包信息
    let pkg_path = ctx_path.join("package.toml");
    let global = parse_package(
        pkg_path.to_string_lossy().to_string(),
        Some(local_str.clone()),
    )?;
    // 写本地信息
    let local = InfoDiff {
        version: global.package.version.clone(),
        authors: global.package.authors.clone(),
    };
    Ok((global.clone(), local))
}

pub fn info(package_name: String) -> Result<Info> {
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
    let local_path = Path::new("./apps").join(&package_name);
    if local_path.exists() {
        let (global, local) = info_local(package_name.clone())?;
        info.license = global.package.license;
        info.local = Some(local);
        info.software = global.software;
    }

    // 检查到底有没有这个包
    if info.local.is_some() || info.online.is_some() {
        Ok(info)
    } else {
        Err(anyhow!("Error:Unknown package '{}'", &package_name))
    }
}

#[test]
fn test_info() {
    let res = info("vscode".to_string());
    println!("{:?}", res);
}
