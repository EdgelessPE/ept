use std::path::Path;

use anyhow::{Result, anyhow};

use crate::{types::{Info, InfoDiff, GlobalPackage}, parsers::parse_package};

use super::validator::installed_validator;

pub fn info_local(package_name:String)->Result<(GlobalPackage,InfoDiff)>{
    let local_path=Path::new("./apps").join(&package_name);
    if !local_path.exists(){
        return Err(anyhow!("Error:Can't find app '{}' locally",package_name));
    }
    let local_str=local_path.to_string_lossy().to_string();
    // 检查是否为标准的已安装目录
    let ctx_str=installed_validator(local_str.clone())?;
    let ctx_path=Path::new(&ctx_str);
    // 读入包信息
    let pkg_path=ctx_path.join("package.toml");
    let global=parse_package(pkg_path.to_string_lossy().to_string())?;
    // 写本地信息
    let local=InfoDiff{
        version:global.package.version.clone(),
        authors:global.package.authors.clone(),
    };
    Ok((global.clone(),local))
}

pub fn info(package_name:String)->Result<Info>{
    // 创建结果结构体
    let mut info=Info{
        name:package_name.clone(),
        template:String::from("Software"),
        licence: None,
        local: None,
        online: None,
        software: None,
    };
    
    // 扫描本地安装目录
    let local_path=Path::new("./apps").join(&package_name);
    if local_path.exists() {
        let (global,local)=info_local(package_name.clone())?;
        info.licence=global.package.licence;
        info.local=Some(local);
        info.software=global.software;
    }

    Ok(info)
}

#[test]
fn test_info(){
    let res=info("vscode".to_string());
    println!("{:?}",res);
}