use std::{fs::remove_dir_all, str::FromStr};

use anyhow::{anyhow, Result};

use crate::{
    executor::workflow_executor,
    p2s,
    parsers::{parse_author, parse_workflow},
    types::extended_semver::ExSemVer,
    utils::{fs::move_or_copy, get_path_apps, term::ask_yn},
};
use crate::{log, log_ok_last};

use super::{
    info_local, install_using_package, uninstall,
    utils::package::{clean_temp, unpack_nep},
    utils::validator::installed_validator,
};

fn same_authors(a: &Vec<String>, b: &Vec<String>) -> bool {
    let ai = a.into_iter().map(|raw| parse_author(raw).unwrap());
    let bi = b.into_iter().map(|raw| parse_author(raw).unwrap());

    ai.eq(bi)
}

pub fn update_using_package(source_file: &String, verify_signature: bool) -> Result<()> {
    log!("Info:Preparing to update with package '{source_file}'");

    // 解包
    let (temp_dir_inner_path, fresh_package) = unpack_nep(source_file, verify_signature)?;
    let fresh_software = fresh_package.software.clone().unwrap();

    // 确认包是否已安装
    log!("Info:Resolving package...");
    let (local_package, local_diff) =
        info_local(&fresh_software.scope, &fresh_package.package.name).map_err(|_| {
            anyhow!(
                "Error:Package '{name}' hasn't been installed, use 'ept install \"{source_file}\"' instead",
                name = &fresh_package.package.name,
            )
        })?;
    let local_software = local_package.software.clone().unwrap();

    // 确认是否允许升级
    let local_version = ExSemVer::from_str(&local_diff.version)?;
    let fresh_version = ExSemVer::from_str(&fresh_package.package.version)?;
    if local_version >= fresh_version {
        return Err(anyhow!("Error:Package '{name}' has been up to date ({local_version}), can't update to the version of given package ({fresh_version})",name=fresh_package.package.name));
    }

    // 确认作者是否一致
    if !same_authors(
        &local_package.package.authors,
        &fresh_package.package.authors,
    ) {
        // 需要卸载然后重新安装
        log!("Warning:The given package is not the same as the author of the installed package (local:{la:?}, given:{fa:?}), uninstall the installed package first? (y/n)",la=local_package.package.authors,fa=fresh_package.package.authors);
        if !ask_yn() {
            return Err(anyhow!("Error:Update canceled by user"));
        }
        // 卸载
        uninstall(Some(local_software.scope), &local_package.package.name)?;
        // 安装
        return install_using_package(source_file, verify_signature);
    }

    let located = get_path_apps(&local_software.scope, &local_package.package.name, true)?;
    log_ok_last!("Info:Resolving package...");

    // 执行旧的 remove 工作流
    let remove_path = located
        .join(".nep_context")
        .join("workflows")
        .join("remove.toml");
    let run_remove = if remove_path.exists() {
        log!("Info:Running remove workflow...");
        let remove_workflow = parse_workflow(&p2s!(remove_path))?;
        workflow_executor(remove_workflow, p2s!(located), local_package)?;
        log_ok_last!("Info:Running remove workflow...");
        true
    } else {
        false
    };

    // 移除旧的 app 目录
    // TODO:尽可能提前检查占用，避免无法删除
    log!("Info:Removing old package...");
    remove_dir_all(&located)?;
    log_ok_last!("Info:Removing old package...");

    // 移动程序至 apps 目录
    log!("Info:Deploying files...");
    move_or_copy(
        temp_dir_inner_path.join(&fresh_package.package.name),
        located.clone(),
    )?;
    log_ok_last!("Info:Deploying files...");

    // 检查有无 update 工作流
    let update_path = temp_dir_inner_path.join("workflows").join("update.toml");
    if update_path.exists() {
        // 执行 update 工作流
        log!("Info:Running update workflow...");
        let update_workflow = parse_workflow(&p2s!(update_path))?;
        workflow_executor(update_workflow, p2s!(located), fresh_package)?;
        log_ok_last!("Info:Running update workflow...");
    } else {
        if run_remove {
            // 没有升级但是跑了一遍卸载，需要重新跑一遍 setup
            log!("Info:Running setup workflow...");
            let setup_workflow = parse_workflow(&p2s!(update_path.with_file_name("setup.toml")))?;
            workflow_executor(setup_workflow, p2s!(located), fresh_package)?;
            log_ok_last!("Info:Running setup workflow...");
        }
    }

    // 保存上下文
    let ctx_path = located.join(".nep_context");
    move_or_copy(temp_dir_inner_path, ctx_path)?;

    // 检查更新是否完整
    log!("Info:Validating update...");
    installed_validator(&p2s!(located))?;
    log_ok_last!("Info:Validating update...");

    // 清理临时文件夹
    clean_temp(source_file)?;

    Ok(())
}

#[test]
fn test_update_using_package() {
    envmnt::set("DEBUG", "true");
    envmnt::set("CONFIRM", "true");
    crate::utils::test::_ensure_clear_test_dir();

    // 卸载
    if crate::entrances::info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_ok() {
        crate::uninstall(Some("Microsoft".to_string()), &"VSCode".to_string()).unwrap();
    }

    // 安装旧版本
    crate::pack(
        &"./examples/VSCode".to_string(),
        Some("./test/VSCode_1.75.0.0_Cno.nep".to_string()),
        true,
    )
    .unwrap();
    install_using_package(&"./test/VSCode_1.75.0.0_Cno.nep".to_string(), true).unwrap();

    // 手动更新版本号
    crate::utils::fs::copy_dir("examples/VSCode", "test/VSCode").unwrap();
    let mut pkg = crate::parsers::parse_package(
        &"test/VSCode/package.toml".to_string(),
        &"test/VSCode".to_string(),
        false,
    )
    .unwrap();
    pkg.package.version = "1.75.0.1".to_string();
    let text = toml::to_string_pretty(&pkg).unwrap();
    std::fs::write("test/VSCode/package.toml", text).unwrap();

    // 更新文件
    let old_ico = get_path_apps(&"Microsoft".to_string(), &"VSCode".to_string(), false)
        .unwrap()
        .join("favicon.ico");
    let new_ico = get_path_apps(&"Microsoft".to_string(), &"VSCode".to_string(), false)
        .unwrap()
        .join("icon.ico");
    assert!(old_ico.exists());
    std::fs::rename(
        "test/VSCode/VSCode/favicon.ico",
        "test/VSCode/VSCode/icon.ico",
    )
    .unwrap();

    // 安装新版本
    update_using_package(&"test/VSCode".to_string(), false).unwrap();
    assert!(!old_ico.exists());
    assert!(new_ico.exists());

    // 卸载
    crate::uninstall(None, &"VSCode".to_string()).unwrap();
}
