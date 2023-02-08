use crate::types::{GlobalPackage, ExSemVer};
use crate::utils::{get_exe_version, parse_relative_path, log};
use anyhow::{anyhow, Result};
use std::path::Path;
use std::{fs::{File,write}, io::Read};

pub fn parse_package(p: String,located:Option<String>) -> Result<GlobalPackage> {
    let package_path = Path::new(&p);
    if !package_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find package.toml path : {}", p));
    }

    let mut text = String::new();
    File::open(&p)?.read_to_string(&mut text)?;
    let pkg_res = toml::from_str(&text);
    if pkg_res.is_err() {
        return Err(anyhow!(
            "Error:Can't validate package.toml at {} : {}",
            p,
            pkg_res.err().unwrap()
        ));
    }

    let mut pkg:GlobalPackage=pkg_res.unwrap();

    // 跟随主程序 exe 文件版本号更新版本号
    if located.is_some() && pkg.software.is_some() {
        let located=located.unwrap();
        // 获取主程序相对路径
        let software=pkg.software.clone().unwrap();
        let mp_relative_path=Path::new(&located).join(&software.main_program);
        let file_path=parse_relative_path(mp_relative_path.to_string_lossy().to_string())?;

        // 读取包申明版本号
        let pkg_version=pkg.package.version.clone();
        let ex_sv_declared=ExSemVer::parse(pkg_version)?;

        // 读取主程序版本号
        let mp_version=get_exe_version(file_path)?;
        let mut ex_sv_latest=ExSemVer::parse(mp_version)?;
        ex_sv_latest.set_reserved(0);

        // 判断是否更新
        if ex_sv_declared.semver_instance != ex_sv_latest.semver_instance{
            log(format!("Warning:Updated package version from '{}' to '{}'",ex_sv_declared,ex_sv_latest));
            pkg.package.version=ex_sv_latest.to_string();
            let new_pkg_text=toml::to_string_pretty(&pkg)?;
            write(package_path,new_pkg_text)?;
        }
    }

    Ok(pkg)
}
