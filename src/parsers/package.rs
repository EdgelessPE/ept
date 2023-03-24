use crate::types::{ExSemVer, GlobalPackage, Software};
use crate::utils::{get_exe_version, parse_relative_path};
use crate::{log, p2s};
use anyhow::{anyhow, Result};
use std::path::Path;
use std::{
    fs::{write, File},
    io::Read,
};

use super::parse_author;

fn update_main_program(
    pkg: &mut GlobalPackage,
    software: Software,
    located: Option<String>,
    package_path: &Path,
) -> Result<()> {
    let located = located.unwrap();
    // 获取主程序相对路径
    let mp_relative_path = Path::new(&located).join(&software.main_program.unwrap());
    let file_path = parse_relative_path(&p2s!(mp_relative_path))?;

    // 读取包申明版本号
    let ex_sv_declared = ExSemVer::parse(&pkg.package.version)?;

    // 读取主程序版本号
    let exe_file_str = p2s!(file_path);
    let mp_version = get_exe_version(file_path)?;
    let mut ex_sv_latest = ExSemVer::parse(&mp_version)?;
    ex_sv_latest.set_reserved(0);

    // 判断是否更新
    if ex_sv_declared.semver_instance != ex_sv_latest.semver_instance {
        log!(
            "Warning:Updated '{}' version from '{}' to '{}' according to '{}'",
            &pkg.package.name,
            ex_sv_declared,
            ex_sv_latest,
            exe_file_str
        );
        pkg.package.version = ex_sv_latest.to_string();
        let new_pkg_text = toml::to_string_pretty(&pkg)?;
        write(package_path, new_pkg_text)?;
    };

    Ok(())
}

/// p 输入 package.toml 所在位置，如需自动更新主程序版本号则传入 located 为包安装后的所在路径
pub fn parse_package(p: &String, located: Option<String>) -> Result<GlobalPackage> {
    let package_path = Path::new(p);
    if !package_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find package.toml path : {}", p));
    }

    let mut text = String::new();
    File::open(p)?.read_to_string(&mut text)?;
    let dirty_toml: toml::Value = toml::from_str(&text)
        .map_err(|res| anyhow!("Error:Invalid toml file '{}' : {}", p, res))?;

    // 检查 nep 版本号是否符合
    let ver_opt = dirty_toml.get("nep");
    if let Some(val) = ver_opt {
        let ver = val.as_str().unwrap_or("0.0");
        let s_ver = &env!("CARGO_PKG_VERSION")[0..ver.len()];
        if ver != s_ver {
            return Err(anyhow!(
                "Error:Can't parse nep version with '{}', current ept only accept version '{}'",
                ver,
                s_ver
            ));
        }
    } else {
        return Err(anyhow!("Error:Field 'nep' undefined in '{}'", p));
    }

    // 序列化
    let mut pkg: GlobalPackage = dirty_toml
        .try_into()
        .map_err(|res| anyhow!("Error:Can't validate package.toml at '{}' : {}", p, res))?;

    // 逐一解析作者
    for (i, raw) in pkg.package.authors.clone().into_iter().enumerate() {
        let author = parse_author(&raw)?;
        // 第一作者必须提供邮箱
        if i == 0 && author.email == None {
            return Err(anyhow!("Error:Can't validate package.toml : first author in field 'package.authors' should have email (e.g. \"Cno <cno@edgeless.top>\")"));
        }
    }

    // 跟随主程序 exe 文件版本号更新版本号
    let software = pkg.software.clone().unwrap();
    if located.is_some() && pkg.software.is_some() && software.main_program.is_some() {
        let u_res = update_main_program(&mut pkg, software, located, package_path);
        if u_res.is_err() {
            log!(
                "Warning:Failed to update main program version for {} : {}",
                &pkg.package.name,
                u_res.unwrap_err()
            );
        }
    }

    Ok(pkg)
}
