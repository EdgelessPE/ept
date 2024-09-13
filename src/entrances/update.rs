use super::{
    info_local, install_using_package, list, uninstall,
    utils::{
        package::{clean_temp, unpack_nep},
        validator::installed_validator,
    },
};
use crate::{
    entrances::{expand_workshop, is_workshop_expandable},
    executor::workflow_executor,
    p2s,
    parsers::{parse_author, parse_workflow},
    types::{author::Author, extended_semver::ExSemVer},
    utils::{
        download::download_nep,
        fs::move_or_copy,
        get_path_apps,
        parse_inputs::{parse_update_inputs, ParseInputResEnum},
        term::ask_yn,
    },
};
use crate::{executor::workflow_reverse_executor, types::info::UpdateInfo};
use crate::{log, log_ok_last};
use anyhow::{anyhow, Result};
use std::{fs::remove_dir_all, str::FromStr};

fn same_authors(a: &[String], b: &[String]) -> bool {
    let ai: Vec<Author> = a.iter().map(|raw| parse_author(raw).unwrap()).collect();
    let bi: Vec<Author> = b.iter().map(|raw| parse_author(raw).unwrap()).collect();

    ai.eq(&bi)
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
    let located_str = p2s!(located);
    log_ok_last!("Info:Resolving package...");

    // 如果旧包有 remove 且新包没有 update 则执行旧包的 remove
    let remove_path = located
        .join(".nep_context")
        .join("workflows")
        .join("remove.toml");
    let update_path = temp_dir_inner_path.join("workflows").join("update.toml");
    if remove_path.exists() && !update_path.exists() {
        log!("Info:Running remove workflow...");
        let remove_workflow = parse_workflow(&p2s!(remove_path))?;
        workflow_executor(remove_workflow, located_str.clone(), local_package.clone())?;
        log_ok_last!("Info:Running remove workflow...");
    };

    // 逆向执行安装工作流
    let setup_path = located
        .join(".nep_context")
        .join("workflows")
        .join("setup.toml");
    let setup_workflow = parse_workflow(&p2s!(setup_path))?;
    log!("Info:Running reverse setup workflow...");
    workflow_reverse_executor(setup_workflow, located_str.clone(), local_package)?;
    log_ok_last!("Info:Running reverse setup workflow...");

    // 执行展开工作流
    let temp_dir_inner = p2s!(temp_dir_inner_path);
    if is_workshop_expandable(&temp_dir_inner) {
        expand_workshop(&temp_dir_inner)?;
    }

    // 移除旧的 app 目录
    // TODO:尽可能提前检查占用，避免无法删除
    log!("Info:Removing old package...");
    remove_dir_all(&located)?;
    log_ok_last!("Info:Removing old package...");

    // 移动程序至 apps 目录
    log!("Info:Deploying files...");
    move_or_copy(temp_dir_inner_path.join(&name), located.clone())?;
    log_ok_last!("Info:Deploying files...");

    // 执行新包的 update，如果没有则执行新包的 setup
    if update_path.exists() {
        // 执行 update 工作流
        log!("Info:Running update workflow...");
        let update_workflow = parse_workflow(&p2s!(update_path))?;
        workflow_executor(update_workflow, located_str.clone(), fresh_package)?;
        log_ok_last!("Info:Running update workflow...");
    } else {
        // 执行 setup 工作流
        log!("Info:Running setup workflow...");
        let setup_workflow = parse_workflow(&p2s!(update_path.with_file_name("setup.toml")))?;
        workflow_executor(setup_workflow, located_str.clone(), fresh_package)?;
        log_ok_last!("Info:Running setup workflow...");
    }

    // 保存上下文
    let ctx_path = located.join(".nep_context");
    move_or_copy(temp_dir_inner_path, ctx_path)?;

    // 检查更新是否完整
    log!("Info:Validating update...");
    installed_validator(&located_str)?;
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

pub fn update_using_package_matcher(matcher: String, verify_signature: bool) -> Result<UpdateInfo> {
    // 解析
    let parsed = parse_update_inputs(vec![matcher])?;
    // 执行更新
    if let ParseInputResEnum::PackageMatcher(p) = parsed.first().unwrap() {
        update_using_url(&p.download_url, verify_signature)
    } else {
        Err(anyhow!(
            "Error:Fatal:Input matcher can't be parsed as package matcher"
        ))
    }
}

pub fn update_using_parsed(
    parsed: Vec<ParseInputResEnum>,
    verify_signature: bool,
) -> Result<Vec<UpdateInfo>> {
    let mut arr = Vec::new();
    for parsed in parsed {
        log!("Info:Start updating with {}", parsed.preview());
        let res = match parsed {
            ParseInputResEnum::LocalPath(p) => update_using_package(&p, false)?,
            ParseInputResEnum::Url(u) => update_using_url(&u, false)?,
            ParseInputResEnum::PackageMatcher(p) => {
                update_using_url(&p.download_url, verify_signature)?
            }
        };
        log!("{}", res.format_success());
        arr.push(res);
    }
    Ok(arr)
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
            .fold("\nUpdatable packages:\n".to_string(), |acc, node| {
                acc + &node.to_string()
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
        let res =
            update_using_package_matcher(format!("{}/{}", info.scope, info.name), verify_signature);
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
fn test_same_author() {
    assert!(same_authors(
        &["J3rry <j3rry@qq.com>".to_string(), "Microsoft".to_string()],
        &["J3rry <j3rry@qq.com>".to_string(), "Microsoft".to_string()]
    ));
    assert!(!same_authors(
        &["J3rry <j3rry@qq.com>".to_string(), "Microsoft".to_string()],
        &[
            "J3rry <dsyourshy@qq.com>".to_string(),
            "Microsoft".to_string()
        ]
    ));
}

#[test]
fn test_update_using_package() {
    envmnt::set("DEBUG", "true");
    envmnt::set("CONFIRM", "true");
    crate::utils::test::_ensure_clear_test_dir();

    // 卸载
    crate::utils::test::_ensure_testing_vscode_uninstalled();

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
    crate::utils::test::_modify_package_dir_version("test/VSCode", "1.75.4.1");

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

#[test]
fn test_update_all() {
    let tup = crate::utils::test::_mount_custom_mirror();
    let (_, mut handler) = crate::utils::test::_run_static_file_server();
    envmnt::set("DEBUG", "true");
    envmnt::set("CONFIRM", "true");
    crate::utils::test::_ensure_clear_test_dir();

    // 确保已卸载
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "Notepad");

    // 生成旧的 Notepad 包
    crate::utils::fs::copy_dir("examples/Notepad", "test/Notepad").unwrap();
    crate::utils::test::_modify_package_dir_version("test/Notepad", "22.0.0.0");

    // 安装旧版本
    install_using_package(&"examples/VSCode".to_string(), false).unwrap();
    install_using_package(&"test/Notepad".to_string(), false).unwrap();

    // 生成新包
    let source_dir = crate::utils::test::_fork_example_with_version("examples/VSCode", "1.75.4.2");
    std::fs::create_dir("test/static").unwrap();
    crate::pack(
        &source_dir,
        Some("./test/static/VSCode_1.75.4.2_Cno.nep".to_string()),
        false,
    )
    .unwrap();
    crate::pack(
        &"./examples/Notepad".to_string(),
        Some("./test/static/Notepad_22.1.0.0_Cno.nep".to_string()),
        false,
    )
    .unwrap();

    // 更新全部
    let (_, failure_count) = update_all(false).unwrap();
    assert_eq!(failure_count, 0);
    assert!(
        info_local(&"Microsoft".to_string(), &"VSCode".to_string())
            .unwrap()
            .1
            .version
            == *"1.75.4.2"
    );
    assert!(
        info_local(&"Microsoft".to_string(), &"Notepad".to_string())
            .unwrap()
            .1
            .version
            == *"22.1.0.0"
    );

    // 卸载
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "Notepad");

    // 清理测试服务器
    handler.kill().unwrap();
    crate::utils::test::_unmount_custom_mirror(tup);
}

#[test]
fn test_update_workflow_executions() {
    use std::path::Path;
    let desktop = crate::utils::env::env_desktop();
    assert!(crate::utils::wild_match::parse_wild_match("vsc*.lnk".to_string(), &desktop).is_err());
    envmnt::set("CONFIRM", "true");

    // (旧包类型，新包类型，更新后断言存在的文件)
    let test_arr = vec![
        (0, 0, vec!["vsc0-setup-1.75.4.1"]),
        (0, 1, vec!["vsc1-setup-1.75.4.1"]),
        (0, 2, vec!["vsc2-update-1.75.4.1"]),
        (0, 3, vec!["vsc3-update-1.75.4.1"]),
        (1, 0, vec!["vsc1-remove-1.75.4.0", "vsc0-setup-1.75.4.1"]),
        (1, 1, vec!["vsc1-remove-1.75.4.0", "vsc1-setup-1.75.4.1"]),
        (1, 2, vec!["vsc2-update-1.75.4.1"]),
        (1, 3, vec!["vsc3-update-1.75.4.1"]),
        (2, 0, vec!["vsc0-setup-1.75.4.1"]),
        (2, 1, vec!["vsc1-setup-1.75.4.1"]),
        (2, 2, vec!["vsc2-update-1.75.4.1"]),
        (2, 3, vec!["vsc3-update-1.75.4.1"]),
        (3, 0, vec!["vsc3-remove-1.75.4.0", "vsc0-setup-1.75.4.1"]),
        (3, 1, vec!["vsc3-remove-1.75.4.0", "vsc1-setup-1.75.4.1"]),
        (3, 2, vec!["vsc2-update-1.75.4.1"]),
        (3, 3, vec!["vsc3-update-1.75.4.1"]),
    ];

    for (old_type, new_type, assert_files) in test_arr {
        log!("Info:Testing updating {old_type} -> {new_type}");
        // 卸载
        crate::utils::test::_ensure_testing_vscode_uninstalled();

        // 安装旧版本
        crate::entrances::install_using_package(
            &format!("examples/UpdateSuit/VSCode{old_type}"),
            false,
        )
        .unwrap();
        assert!(Path::new(&desktop)
            .join(format!("vsc{old_type}-setup-1.75.4.0.lnk"))
            .exists());

        // 更新
        let source_file = crate::utils::test::_fork_example_with_version(
            &format!("examples/UpdateSuit/VSCode{new_type}"),
            "1.75.4.1",
        );
        crate::entrances::update_using_package(&source_file, false).unwrap();

        // 断言仅存在指定文件
        for file in assert_files {
            let p = Path::new(&desktop).join(format!("{file}.lnk"));
            assert!(p.exists());
            std::fs::remove_file(p).unwrap();
        }
        assert!(
            crate::utils::wild_match::parse_wild_match("vsc*.lnk".to_string(), &desktop).is_err()
        );

        // 卸载
        crate::utils::test::_ensure_testing_vscode_uninstalled();
        let remove_lnk = Path::new(&desktop).join(format!("vsc{new_type}-remove-1.75.4.1.lnk"));
        if remove_lnk.exists() {
            std::fs::remove_file(remove_lnk).unwrap()
        }
    }
}

#[test]
fn test_update_with_different_author() {
    envmnt::set("CONFIRM", "true");
    let desktop = crate::utils::env::env_desktop();
    assert!(crate::utils::wild_match::parse_wild_match("vsc*.lnk".to_string(), &desktop).is_err());
    let desktop_path = std::path::Path::new(&desktop);

    // 卸载
    crate::utils::test::_ensure_testing_vscode_uninstalled();

    // 安装旧版本
    crate::entrances::install_using_package(&"examples/UpdateSuit/VSCode3".to_string(), false)
        .unwrap();
    assert!(desktop_path.join("vsc3-setup-1.75.4.0.lnk").exists());

    // 安装新版本
    let source_file = crate::utils::test::_fork_example_with_version("examples/VSCode", "1.75.4.2");
    std::fs::copy(
        "examples/UpdateSuit/VSCode2/workflows/update.toml",
        std::path::Path::new(&source_file).join("workflows/update.toml"),
    )
    .unwrap();
    update_using_package(&source_file, false).unwrap();

    // 断言是先卸载再安装的
    assert!(!desktop_path.join("vsc3-setup-1.75.4.0.lnk").exists());
    assert!(!desktop_path.join("vsc3-update-1.75.4.0.lnk").exists());
    assert!(!desktop_path.join("vsc2-update-1.75.4.2.lnk").exists());
    assert!(desktop_path.join("vsc3-remove-1.75.4.0.lnk").exists());
    assert!(desktop_path.join("Visual Studio Code.lnk").exists());

    // 卸载
    std::fs::remove_file(desktop_path.join("vsc3-remove-1.75.4.0.lnk")).unwrap();
    crate::utils::test::_ensure_testing_vscode_uninstalled();
}

#[test]
fn test_update_expandable() {
    use std::path::Path;
    envmnt::set("CONFIRM", "true");
    crate::utils::test::_ensure_clear_test_dir();
    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "VSCodeE");

    // 断言原来的包中不包含这个二进制文件
    assert!(!Path::new("examples/VSCodeE/VSCodeE/Code.exe").exists());

    // 启动文件服务器
    let (_addr, mut handler) = crate::utils::test::_run_static_file_server();
    std::fs::copy("examples/VSCode/VSCode/Code.exe", "test/Code.exe").unwrap();

    // 安装
    crate::utils::fs::copy_dir("examples/VSCodeE", "test/VSCodeE").unwrap();
    install_using_package(&"test/VSCodeE".to_string(), false).unwrap();

    // 断言安装成功
    assert!(info_local(&"Microsoft".to_string(), &"VSCodeE".to_string()).is_ok());
    let app_exe_path = get_path_apps(&"Microsoft".to_string(), &"VSCodeE".to_string(), false)
        .unwrap()
        .join("Code.exe");
    assert!(app_exe_path.exists());

    // 手动删除这个包
    std::fs::remove_file(&app_exe_path).unwrap();
    assert!(!app_exe_path.exists());

    // 生成一个更新包
    let pkg_path = crate::utils::test::_fork_example_with_version("examples/VSCodeE", "1.75.5.0");

    // 断言原来的包中不包含这个二进制文件
    assert!(!Path::new(&pkg_path).join("VSCodeE/Code.exe").exists());

    // 安装更新包
    update_using_package(&pkg_path, false).unwrap();

    // 断言安装成功
    assert!(
        info_local(&"Microsoft".to_string(), &"VSCodeE".to_string())
            .unwrap()
            .1
            .version
            == "1.75.5.0"
    );
    assert!(app_exe_path.exists());

    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "VSCodeE");
    handler.kill().unwrap();
}
