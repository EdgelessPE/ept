use crate::types::{ExSemVer, GlobalPackage};
use crate::utils::{get_exe_version, log, parse_relative_path};
use anyhow::{anyhow, Result};
use std::path::Path;
use std::{
    fs::{write, File},
    io::Read,
};

use super::parse_author;

pub fn parse_package(p: String, located: Option<String>) -> Result<GlobalPackage> {
    let package_path = Path::new(&p);
    if !package_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find package.toml path : {}", p));
    }

    let mut text = String::new();
    File::open(&p)?.read_to_string(&mut text)?;
    let mut pkg: GlobalPackage = toml::from_str(&text)
        .map_err(|res| anyhow!("Error:Can't validate package.toml at {} : {}", p, res))?;

    // 逐一解析作者
    for (i, raw) in pkg.package.authors.clone().into_iter().enumerate() {
        let author = parse_author(raw)?;
        // 第一作者必须提供邮箱
        if i == 0 && author.email == None {
            return Err(anyhow!("Error:Can't validate package.toml : first author in field 'package.authors' should have email (e.g. \"Cno <cno@edgeless.top>\")"));
        }
    }

    // 跟随主程序 exe 文件版本号更新版本号
    let software = pkg.software.clone().unwrap();
    if located.is_some() && pkg.software.is_some() && software.main_program.is_some() {
        let located = located.unwrap();
        // 获取主程序相对路径
        let mp_relative_path = Path::new(&located).join(&software.main_program.unwrap());
        let file_path = parse_relative_path(mp_relative_path.to_string_lossy().to_string())?;

        // 读取包申明版本号
        let pkg_version = pkg.package.version.clone();
        let ex_sv_declared = ExSemVer::parse(pkg_version)?;

        // 读取主程序版本号
        let exe_file_str = file_path.to_string_lossy().to_string();
        let mp_version = get_exe_version(file_path)?;
        let mut ex_sv_latest = ExSemVer::parse(mp_version)?;
        ex_sv_latest.set_reserved(0);

        // 判断是否更新
        if ex_sv_declared.semver_instance != ex_sv_latest.semver_instance {
            log(format!(
                "Warning:Updated '{}' version from '{}' to '{}' according to '{}'",
                &pkg.package.name, ex_sv_declared, ex_sv_latest, exe_file_str
            ));
            pkg.package.version = ex_sv_latest.to_string();
            let new_pkg_text = toml::to_string_pretty(&pkg)?;
            write(package_path, new_pkg_text)?;
        }
    }

    Ok(pkg)
}
