use crate::executor::values_replacer;
use crate::types::interpretable::Interpretable;
use crate::types::{extended_semver::ExSemVer, package::GlobalPackage, software::Software};
use crate::utils::{get_exe_version, get_path_apps, parse_relative_path_with_located};
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
    software: &Software,
    located: Option<String>,
    package_path: &Path,
) -> Result<()> {
    let located = located.unwrap();
    let software = software.clone();
    // 获取主程序相对路径
    let file_path = parse_relative_path_with_located(&software.main_program.unwrap(), &located);

    // 读取包申明版本号
    let ex_sv_declared = ExSemVer::parse(&pkg.package.version)?;

    // 读取主程序版本号（只关心符合 SemVer 规范的前三位）
    let exe_file_str = p2s!(file_path);
    let mp_version = get_exe_version(file_path)?;
    let mut ex_sv_latest = ExSemVer::parse(&mp_version)?;
    ex_sv_latest.set_reserved(0);

    // 判断是否更新
    if ex_sv_declared.semver_instance != ex_sv_latest.semver_instance {
        log!(
            "Warning:Updated '{name}' version from '{ex_sv_declared}' to '{ex_sv_latest}' according to '{exe_file_str}'",
            name = pkg.package.name
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
        return Err(anyhow!("Error:Fatal:Can't find package.toml path : {p}"));
    }

    let mut text = String::new();
    File::open(p)?.read_to_string(&mut text)?;
    let dirty_toml: toml::Value =
        toml::from_str(&text).map_err(|res| anyhow!("Error:Invalid toml file '{p}' : {res}"))?;

    // 检查 nep 版本号是否符合
    let ver_opt = dirty_toml.get("nep");
    if let Some(val) = ver_opt {
        let pkg_ver = val.as_str().unwrap_or("0.0").to_string();
        is_nep_version_compatible(&pkg_ver, &env!("CARGO_PKG_VERSION").to_string())?;
    } else {
        return Err(anyhow!("Error:Field 'nep' undefined in '{p}'"));
    }

    // 序列化
    let mut pkg: GlobalPackage = dirty_toml
        .try_into()
        .map_err(|res| anyhow!("Error:Can't validate package.toml at '{p}' : {res}"))?;

    // 逐一解析作者
    for (i, raw) in pkg.package.authors.clone().into_iter().enumerate() {
        let author = parse_author(&raw)?;
        // 第一作者必须提供邮箱
        if i == 0 && author.email == None {
            return Err(anyhow!("Error:Can't validate package.toml : first author '{name}' in field 'package.authors' should have email (e.g. \"Cno <cno@edgeless.top>\")",name=author.name));
        }
    }

    // 跟随主程序 exe 文件版本号更新版本号
    let software = pkg.software.clone().unwrap();
    if located.is_some() && pkg.software.is_some() && software.main_program.is_some() {
        if let Err(e) = update_main_program(&mut pkg, &software, located.clone(), package_path) {
            log!(
                "Warning:Failed to update main program version for {name} : {e}",
                name = pkg.package.name,
            );
        }
    }

    // 解释
    let scope = &software.scope;
    let located = located.unwrap_or_else(|| {
        let p = get_path_apps(scope, &pkg.package.name, false).unwrap();
        p2s!(p)
    });
    let interpreter = |raw: String| values_replacer(raw, 0, &located);
    let pkg = pkg.interpret(interpreter);

    Ok(pkg)
}

fn is_nep_version_compatible(pkg_str: &String, ept_str: &String) -> Result<()> {
    let pkg_ver = semver::Version::parse(&(pkg_str.clone() + ".0"))
        .map_err(|e| anyhow!("Error:Failed to parse nep package version '{pkg_str}' : '{e}'"))?;
    let ept_ver = semver::Version::parse(ept_str)?;

    if pkg_ver.major != ept_ver.major || pkg_ver.minor != ept_ver.minor {
        // 0 开头的要求 major 和 minor 一致
        if pkg_str.starts_with("0.") || ept_str.to_string().starts_with("0.") {
            return Err(anyhow!(
                "Error:Nep package version '{pkg_str}' incompatible, current ept only accept version '{ept_str}'"
            ));
        } else {
            // 检查 major 是否一致
            if pkg_ver.major != ept_ver.major {
                return Err(anyhow!(
                    "Error:Nep package version '{pkg_str}' incompatible, current ept only accept version starts with '{major}'",
                    major=ept_ver.major
                ));
            }
        }
    }

    Ok(())
}

#[test]
fn test_update_main_program() {
    let located = "examples/Dism++".to_string();
    let mut pkg = parse_package(
        &"examples/Dism++/package.toml".to_string(),
        Some(located.clone()),
    )
    .unwrap();
    pkg.package.version = "10.1.112.1".to_string();
    let software = pkg.clone().software.unwrap();

    update_main_program(
        &mut pkg,
        &software,
        Some("examples/Dism++/Dism++".to_string()),
        &Path::new("test/nul.toml"),
    )
    .unwrap();
    assert!(pkg.package.version == "10.1.1002.0".to_string());
}

#[test]
fn test_is_nep_version_compatible() {
    assert!(is_nep_version_compatible(&"0.2".to_string(), &"0.2.1".to_string()).is_ok());
    assert!(is_nep_version_compatible(&"0.1".to_string(), &"0.2.1".to_string()).is_err());
    assert!(is_nep_version_compatible(&"1.0".to_string(), &"1.10.3".to_string()).is_ok());
    assert!(is_nep_version_compatible(&"1.0".to_string(), &"1.0.30".to_string()).is_ok());
    assert!(is_nep_version_compatible(&"1.2".to_string(), &"1.0.30".to_string()).is_ok());
    assert!(is_nep_version_compatible(&"1.8".to_string(), &"2.0.0".to_string()).is_err());
}

#[test]
fn test_parse_package() {
    let pkg = parse_package(&"examples/VSCode/package.toml".to_string(), None).unwrap();
    let answer = GlobalPackage {
        nep: "0.2".to_string(),
        package: crate::types::package::Package {
            name: "VSCode".to_string(),
            description: "Visual Studio Code".to_string(),
            template: "Software".to_string(),
            version: "1.75.0.0".to_string(),
            authors: vec![
                "Cno <dsyourshy@qq.com>".to_string(),
                "Microsoft".to_string(),
            ],
            license: None,
        },
        software: Some(Software {
            scope: "Microsoft".to_string(),
            upstream: "https://code.visualstudio.com/".to_string(),
            category: "办公编辑".to_string(),
            arch: None,
            language: "Multi".to_string(),
            main_program: None,
            tags: Some(vec!["Electron".to_string()]),
            alias: None,
            installed: Some(
                crate::utils::env::env_appdata() + "/Local/Programs/Microsoft VS Code/Code.exe",
            ),
        }),
    };
    assert_eq!(pkg, answer)
}
