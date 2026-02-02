use crate::executor::values_replacer;
use crate::types::constants::FILE_PACKAGE;
use crate::types::context::VerifiableCtx;
use crate::types::interpretable::Interpretable;
use crate::types::mixed_fs::MixedFS;
use crate::types::verifiable::Verifiable;
use crate::types::{extended_semver::ExSemVer, package::GlobalPackage};
use crate::utils::expand::get_expanded_mixed_fs;
use crate::utils::get_workflows_path;
use crate::utils::reg_entry::get_reg_entry;
use crate::utils::{exe_version::get_exe_version, path::parse_relative_path_with_located};
use crate::{log, p2s};
use anyhow::{anyhow, Result};
use std::path::Path;
use std::{
    fs::{write, File},
    io::Read,
};

use super::parse_author;

// 输入读到的版本号，判断是否需要更新 pkg 并自动写文件系统
fn update_pkg_version(
    pkg: &mut GlobalPackage,
    read_ver: &str,
    package_path: &Path,
    according_to: String,
) -> Result<()> {
    // 解析为 ExSemVeer 实例
    let mut read_ver = ExSemVer::parse(read_ver)?;
    read_ver.set_reserved(0); // 对读到的版本号保留位清 0
    let current_ver = ExSemVer::parse(&pkg.package.version)?;

    // 判断是否更新
    if read_ver.semver_instance > current_ver.semver_instance {
        log!(
            "Warning:Updated '{name}' version from '{current_ver}' to '{read_ver}' according to {according_to}",
            name = pkg.package.name
        );
        pkg.package.version = read_ver.to_string();
        let new_pkg_text = toml::to_string_pretty(&pkg)?;
        write(package_path, new_pkg_text)?;
    };

    Ok(())
}

fn update_ver_with_main_program(
    pkg: &mut GlobalPackage,
    main_program: &str,
    located: &str,
    package_path: &Path,
) -> Result<()> {
    // 解释内置变量
    let interpreted_main_program =
        values_replacer(main_program.to_string(), 0, located, &pkg.package.version);
    // 获取主程序相对路径
    let file_path = parse_relative_path_with_located(&interpreted_main_program, located);

    // 读取主程序版本号
    let exe_file_str = p2s!(file_path);
    let mp_version = get_exe_version(file_path)?;

    update_pkg_version(
        pkg,
        &mp_version,
        package_path,
        format!("main program '{exe_file_str}'"),
    )
}

fn update_ver_with_reg_entry(
    pkg: &mut GlobalPackage,
    entry_id: &str,
    package_path: &Path,
) -> Result<()> {
    let e = get_reg_entry(entry_id);
    if let Some(read_ver) = e.version {
        return update_pkg_version(
            pkg,
            &read_ver,
            package_path,
            format!("registry entry '{entry_id}'"),
        );
    } else {
        log!("Warning:Failed to read version due to registry entry '{entry_id}'");
    }

    Ok(())
}

/// p 输入 package.toml 所在位置
pub fn parse_package(
    cfg: &crate::types::context::RuntimeContext,
    p: &str,
    located: &str,
    need_update_main_program: bool,
) -> Result<GlobalPackage> {
    log!("Debug:Parse package '{p}' with located '{located}'");
    let package_path = Path::new(p);
    if !package_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find {FILE_PACKAGE} path : {p}",));
    }

    let mut text = String::new();
    File::open(p)?.read_to_string(&mut text)?;
    let dirty_toml: toml::Value =
        toml::from_str(&text).map_err(|res| anyhow!("Error:Invalid toml file '{p}' : {res}"))?;

    // 检查 nep 版本号是否符合
    let ver_opt = dirty_toml.get("nep");
    if let Some(val) = ver_opt {
        let pkg_ver = val.as_str().unwrap_or("0.0").to_string();
        is_nep_version_compatible(&pkg_ver, env!("CARGO_PKG_VERSION"))?;
    } else {
        return Err(anyhow!("Error:Field 'nep' undefined in '{p}'"));
    }

    // 序列化
    let pkg: GlobalPackage = dirty_toml
        .try_into()
        .map_err(|res| anyhow!("Error:Can't validate {FILE_PACKAGE} at '{p}' : {res}",))?;
    let software = pkg.software.clone().unwrap();

    // 逐一解析作者
    for (i, raw) in pkg.package.authors.clone().into_iter().enumerate() {
        let author = parse_author(&raw)?;
        // 第一作者必须提供邮箱
        if i == 0 && author.email.is_none() {
            return Err(anyhow!("Error:Can't validate {FILE_PACKAGE} : first author '{name}' in field 'package.authors' should have email (e.g. \"Cno <cno@edgeless.top>\")", name=author.name));
        }
    }

    // 支持智能识别 located 指的 "根目录" 还是 "根目录/名称"
    let mixed_located = if Path::new(&(located.to_owned() + "/" + FILE_PACKAGE)).exists() {
        &format!("{located}/{name}", name = pkg.package.name)
    } else {
        located
    };

    // 校验
    let workflows_path = get_workflows_path(located)?;
    let mixed_fs = get_expanded_mixed_fs(MixedFS::new(mixed_located), workflows_path)?;
    let ctx = VerifiableCtx {
        mixed_fs: &mixed_fs,
        runtime_ctx: cfg,
    };
    pkg.verify_self(&ctx)?;

    // 解释
    let package_version = pkg.package.version.clone();
    let interpreter = |raw: String| values_replacer(raw, 0, located, &package_version);
    let mut pkg = pkg.interpret(interpreter);

    // 跟随主程序 exe 文件版本号或是注册表入口 ID 更新版本号
    if need_update_main_program {
        if let Some(main_program) = software.main_program {
            if let Err(e) =
                update_ver_with_main_program(&mut pkg, &main_program, located, package_path)
            {
                log!(
                    "Warning:Failed to update main program version for '{name}' : {e}",
                    name = pkg.package.name,
                );
            }
        }
    }
    if need_update_main_program {
        if let Some(registry_entry) = software.registry_entry {
            if let Err(e) = update_ver_with_reg_entry(&mut pkg, &registry_entry, package_path) {
                log!(
                "Warning:Failed to update main program version for '{name}' with reg entry '{registry_entry}' : {e}",
                name = pkg.package.name,
            );
            }
        }
    }

    Ok(pkg)
}

fn is_nep_version_compatible(pkg_str: &str, ept_str: &str) -> Result<()> {
    // 检查 nep 版本号是一位数字
    if pkg_str.len() != 1 || pkg_str.parse::<u32>().is_err() {
        return Err(anyhow!("Error:Invalid nep version '{pkg_str}'"));
    }

    // 检查第一位兼容
    if ept_str.starts_with(pkg_str) {
        Ok(())
    } else {
        Err(anyhow!(
            "Error:Nep package version '{pkg_str}' incompatible, current ept only accept version starts with '{major}'",
            major=ept_str.chars().next().unwrap_or_default()
        ))
    }
}

#[test]
fn test_update_main_program() {
    use crate::utils::test::_default_test_cfg;
    let cfg = _default_test_cfg();
    let located = "examples/Dism++";
    let mut pkg = parse_package(&cfg, "examples/Dism++/package.toml", located, true).unwrap();
    pkg.package.version = "10.1.112.1".to_string();
    let software = pkg.clone().software.unwrap();

    update_ver_with_main_program(
        &mut pkg,
        &software.main_program.unwrap(),
        "examples/Dism++/Dism++",
        Path::new("test/nul.toml"),
    )
    .unwrap();
    assert_eq!(pkg.package.version, *"10.1.1002.0");

    update_ver_with_main_program(
        &mut pkg,
        "${SystemDrive}/Windows/notepad.exe",
        "examples/Dism++/Dism++",
        Path::new("test/nul.toml"),
    )
    .unwrap();
}

#[test]
fn test_is_nep_version_compatible() {
    assert!(is_nep_version_compatible("0", "0.2.1").is_ok());
    assert!(is_nep_version_compatible("1", "1.10.3").is_ok());
    assert!(is_nep_version_compatible("1", "1.0.30").is_ok());
    assert!(is_nep_version_compatible("1", "1.0.30").is_ok());
    assert!(is_nep_version_compatible("1", "2.0.0").is_err());
}

#[test]
fn test_parse_package() {
    use crate::utils::flags::{set_flag, Flag};
    use crate::utils::test::_default_test_cfg;
    set_flag(Flag::Debug, true);
    let cfg = _default_test_cfg();
    let located = "examples/VSCode";
    let pkg = parse_package(&cfg, "examples/VSCode/package.toml", located, false).unwrap();
    let answer = GlobalPackage {
        nep: "0".to_string(),
        package: crate::types::package::Package {
            scope: "Microsoft".to_string(),
            name: "VSCode".to_string(),
            description: "Visual Studio Code".to_string(),
            template: "Software".to_string(),
            version: "1.75.4.0".to_string(),
            authors: vec![
                "Cno <dsyourshy@qq.com>".to_string(),
                "Microsoft".to_string(),
            ],
            license: Some("MIT".to_string()),
            icon: None,
            strict: None,
        },
        software: Some(crate::types::software::Software {
            upstream: "https://code.visualstudio.com/".to_string(),
            category: "办公编辑".to_string(),
            arch: None,
            language: "Multi".to_string(),
            main_program: Some("Code.exe".to_string()),
            tags: Some(vec!["Electron".to_string()]),
            alias: None,
            registry_entry: None,
        }),
    };
    assert_eq!(pkg, answer)
}
