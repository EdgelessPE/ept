use super::{
    info_local, info_online, install_using_package, list, uninstall,
    utils::{
        package::{clean_temp, unpack_nep},
        validator::installed_validator,
    },
};
use crate::types::info::UpdateInfo;
use crate::{
    executor::workflow_executor,
    p2s,
    parsers::{parse_author, parse_workflow},
    types::{extended_semver::ExSemVer, matcher::PackageMatcher},
    utils::{
        download::download_nep,
        fs::move_or_copy,
        get_path_apps,
        mirror::{filter_release, get_url_with_version_req},
        path::find_scope_with_name,
        term::ask_yn,
    },
};
use crate::{log, log_ok_last};
use anyhow::{anyhow, Result};
use std::{fs::remove_dir_all, str::FromStr};

fn same_authors(a: &[String], b: &[String]) -> bool {
    let ai = a.iter().map(|raw| parse_author(raw).unwrap());
    let bi = b.iter().map(|raw| parse_author(raw).unwrap());

    ai.eq(bi)
}

pub fn update_using_package(source_file: &String, verify_signature: bool) -> Result<UpdateInfo> {
    log!("Info:Preparing to update with package '{source_file}'");

    // 解包
    let (temp_dir_inner_path, fresh_package) = unpack_nep(source_file, verify_signature)?;
    let fresh_software = fresh_package.software.clone().unwrap();
    let name = fresh_package.package.name.clone();
    let fresh_scope = fresh_software.scope;

    // 确认包是否已安装
    log!("Info:Resolving package...");
    let (local_package, local_diff) = info_local(&fresh_scope, &name).map_err(|_| {
        anyhow!("Error:Package '{name}' hasn't been installed, use 'ept install' instead",)
    })?;
    let local_software = local_package.software.clone().unwrap();

    // 确认是否允许升级
    let local_version = ExSemVer::from_str(&local_diff.version)?;
    let fresh_version_str = fresh_package.package.version.clone();
    let fresh_version = ExSemVer::from_str(&fresh_version_str)?;
    if local_version >= fresh_version {
        return Err(anyhow!("Error:Package '{name}' has been up to date ({local_version}), can't update to the version of given package ({fresh_version})"));
    }

    // 确认作者是否一致
    if !same_authors(
        &local_package.package.authors,
        &fresh_package.package.authors,
    ) {
        // 需要卸载然后重新安装
        if !ask_yn(format!("The given package is not the same as the author of the installed package (local:{:?}, given:{:?}), uninstall the installed package first?",local_package.package.authors,fresh_package.package.authors),true) {
            return Err(anyhow!("Error:Update canceled by user"));
        }
        // 卸载
        uninstall(Some(local_software.scope), &local_package.package.name)?;
        // 安装
        install_using_package(source_file, verify_signature)?;
        return Ok(UpdateInfo {
            name,
            scope: fresh_scope,
            from_version: local_diff.version,
            to_version: fresh_package.package.version,
        });
    }

    let located = get_path_apps(&local_software.scope, &name, true)?;
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
    move_or_copy(temp_dir_inner_path.join(&name), located.clone())?;
    log_ok_last!("Info:Deploying files...");

    // 检查有无 update 工作流
    let update_path = temp_dir_inner_path.join("workflows").join("update.toml");
    if update_path.exists() {
        // 执行 update 工作流
        log!("Info:Running update workflow...");
        let update_workflow = parse_workflow(&p2s!(update_path))?;
        workflow_executor(update_workflow, p2s!(located), fresh_package)?;
        log_ok_last!("Info:Running update workflow...");
    } else if run_remove {
        // 没有升级但是跑了一遍卸载，需要重新跑一遍 setup
        log!("Info:Running setup workflow...");
        let setup_workflow = parse_workflow(&p2s!(update_path.with_file_name("setup.toml")))?;
        workflow_executor(setup_workflow, p2s!(located), fresh_package)?;
        log_ok_last!("Info:Running setup workflow...");
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

    Ok(UpdateInfo {
        name,
        scope: fresh_scope,
        from_version: local_diff.version,
        to_version: fresh_version_str,
    })
}

pub fn update_using_url(url: &str, verify_signature: bool) -> Result<UpdateInfo> {
    // 下载文件到临时目录
    let p = download_nep(url)?;

    // 更新
    update_using_package(&p2s!(p), verify_signature)
}

pub fn update_using_package_matcher(
    matcher: PackageMatcher,
    verify_signature: bool,
) -> Result<UpdateInfo> {
    // 查找 scope 并使用 scope 更新纠正大小写
    let (scope, package_name) = find_scope_with_name(&matcher.name, matcher.scope.clone())?;
    // 检查对应包名有没有被安装过
    let (_global, local_diff) = info_local(&scope, &package_name).map_err(|_| {
        anyhow!(
            "Error:Package '{name}' hasn't been installed, use 'ept install' instead",
            name = package_name
        )
    })?;
    // 检查包的版本号是否允许升级
    let (online_item, _url_template) = info_online(&scope, &package_name, matcher.mirror.clone())?;
    let selected_release = filter_release(online_item.releases, matcher.version_req.clone())?;
    if selected_release.version <= ExSemVer::parse(&local_diff.version)? {
        return Err(anyhow!("Error:Package '{name}' has been up to date ({local_version}), can't update to the version of given package ({fresh_version})",name=package_name,local_version=&local_diff.version,fresh_version=&selected_release.version));
    }
    // 解析 url
    let (url, target_release) = get_url_with_version_req(matcher)?;
    // 执行更新
    if ask_yn(
        format!(
            "Ready to update '{scope}/{package_name}' from '{from}' to '{to}', continue?",
            from = local_diff.version,
            to = target_release.version
        ),
        true,
    ) {
        update_using_url(&url, verify_signature)
    } else {
        Err(anyhow!("Error:Operation canceled by user"))
    }
}

pub fn update_all(verify_signature: bool) -> Result<(i32, i32)> {
    // 遍历 list 结果，生成更新列表
    let list_res = list()?;
    let update_list: Vec<UpdateInfo> = list_res
        .iter()
        .filter_map(|node| {
            let node = node.to_owned();
            // 对比版本号
            let local_version = node.local?.version;
            let online_version = node.online?.version;
            let local_instance = ExSemVer::parse(&local_version).ok()?;
            let online_instance = ExSemVer::parse(&online_version).ok()?;
            if local_instance >= online_instance {
                return None;
            }

            Some(UpdateInfo {
                name: node.name.to_owned(),
                scope: node.software?.scope,
                from_version: local_version,
                to_version: online_version,
            })
        })
        .collect();
    let count = update_list.len();

    // 打印并确认更新
    if update_list.is_empty() {
        return Ok((0, 0));
    } else {
        let tip = update_list
            .iter()
            .fold(String::from("\nUpdatable packages:\n"), |acc, node| {
                acc + &format!("{node}")
            });
        println!("{tip}");
        if !ask_yn(
            format!("Ready to update those {count} packages, continue?"),
            true,
        ) {
            return Err(anyhow!("Error:Operation canceled by user"));
        }
    }

    // 依次更新
    let mut success_count = 0;
    let mut failure_count = 0;
    envmnt::set("CONFIRM", "true");
    for info in update_list {
        let res = update_using_package_matcher(
            PackageMatcher {
                name: info.name.clone(),
                scope: Some(info.scope.clone()),
                mirror: None,
                version_req: None,
            },
            verify_signature,
        );
        if let Err(e) = res {
            failure_count += 1;
            log!("{}", info.format_failure(e));
        } else {
            success_count += 1;
            log!("{}", info.format_success());
        }
    }
    envmnt::set("CONFIRM", "false");

    Ok((success_count, failure_count))
}

#[test]
fn test_update_using_package() {
    envmnt::set("DEBUG", "true");
    envmnt::set("CONFIRM", "true");
    crate::utils::test::_ensure_clear_test_dir();

    // 卸载
    if info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_ok() {
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
    pkg.package.version = "1.75.4.1".to_string();
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
