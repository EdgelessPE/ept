use super::{
    info_local, install_using_package, list, uninstall,
    utils::{
        package::{clean_temp, unpack_nep},
        validator::installed_validator,
    },
};
use crate::types::constants::{
    DIR_NEP_CONTEXT, DIR_WORKFLOWS, WORKFLOW_REMOVE, WORKFLOW_SETUP, WORKFLOW_UPDATE,
};

use crate::{
    entrances::{expand_workshop, is_workshop_expandable},
    executor::{workflow_executor, workflow_reverse_executor},
    p2s,
    parsers::{parse_author, parse_workflow},
    signature::blake3::compute_hash_blake3_from_string,
    types::{
        author::Author, cfg::Cfg, extended_semver::ExSemVer, info::UpdateInfo,
        package::GlobalPackage,
    },
    utils::{
        cache::spawn_cache,
        download::download_nep,
        fs::move_or_copy,
        get_path_apps, get_path_cache,
        parse_inputs::{parse_update_inputs, ParseInputResEnum},
        term::ask_yn,
    },
};
use crate::{log, log_ok_last};
use anyhow::{anyhow, Result};
use std::{fs::remove_dir_all, path::Path, str::FromStr};

fn same_authors(a: &[String], b: &[String]) -> bool {
    let ai: Vec<Author> = a.iter().map(|raw| parse_author(raw).unwrap()).collect();
    let bi: Vec<Author> = b.iter().map(|raw| parse_author(raw).unwrap()).collect();

    ai.eq(&bi)
}

// 验证版本升级是否允许
fn validate_version_update(name: &str, local_ver: &str, fresh_ver: &str) -> Result<()> {
    let local_version = ExSemVer::from_str(local_ver)?;
    let fresh_version = ExSemVer::from_str(fresh_ver)?;
    log!("Debug:Comparing versions for '{name}': local={local_version}, fresh={fresh_version}");
    if local_version >= fresh_version {
        return Err(anyhow!("Error:Package '{name}' has been up to date ({local_version}), can't update to the version of given package ({fresh_version})"));
    }
    Ok(())
}

// 处理作者不匹配的情况，需要卸载后重新安装
fn handle_author_mismatch(
    cfg: &Cfg,
    source_file: &str,
    local: &GlobalPackage,
    fresh: &GlobalPackage,
    local_ver: String,
    verify_signature: bool,
) -> Result<Option<UpdateInfo>> {
    if same_authors(&local.package.authors, &fresh.package.authors) {
        return Ok(None);
    }

    log!(
        "Debug:Author mismatch detected for '{name}': local={local:?}, fresh={fresh:?}",
        name = local.package.name,
        local = local.package.authors,
        fresh = fresh.package.authors
    );

    if !ask_yn(cfg, format!("The given package is not the same as the author of the installed package (local:{:?}, given:{:?}), uninstall the installed package first?",local.package.authors,fresh.package.authors),true) {
        return Err(anyhow!("Error:Update canceled by user"));
    }

    uninstall(cfg, Some(local.package.scope.clone()), &local.package.name)?;
    install_using_package(cfg, source_file, verify_signature)?;

    Ok(Some(UpdateInfo {
        name: fresh.package.name.clone(),
        scope: fresh.package.scope.clone(),
        from_version: local_ver,
        to_version: fresh.package.version.clone(),
    }))
}

// 如有需要，执行旧包的移除工作流
fn run_old_remove_if_needed(
    cfg: &Cfg,
    located: &Path,
    temp_dir: &Path,
    local_pkg: &GlobalPackage,
) -> Result<()> {
    let remove_path = located
        .join(DIR_NEP_CONTEXT)
        .join(DIR_WORKFLOWS)
        .join(WORKFLOW_REMOVE);
    let update_path = temp_dir.join(DIR_WORKFLOWS).join(WORKFLOW_UPDATE);

    if remove_path.exists() && !update_path.exists() {
        log!("Info:Running remove workflow...");
        let remove_workflow = parse_workflow(&p2s!(remove_path))?;
        let located_str = p2s!(located);
        workflow_executor(cfg.clone(), remove_workflow, located_str, local_pkg.clone())?;
        log_ok_last!("Info:Running remove workflow...");
    }
    Ok(())
}

// 逆向执行安装工作流
fn reverse_setup_workflow(cfg: &Cfg, located: &Path, local_pkg: GlobalPackage) -> Result<()> {
    let setup_path = located
        .join(DIR_NEP_CONTEXT)
        .join(DIR_WORKFLOWS)
        .join(WORKFLOW_SETUP);
    let setup_workflow = parse_workflow(&p2s!(setup_path))?;
    let located_str = p2s!(located);

    log!("Info:Running reverse setup workflow...");
    workflow_reverse_executor(cfg.clone(), setup_workflow, located_str, local_pkg)?;
    log_ok_last!("Info:Running reverse setup workflow...");
    Ok(())
}

// 部署新版本文件
fn deploy_update(temp_dir: &Path, located: &Path, name: &str) -> Result<()> {
    log!("Info:Removing old package...");
    log!("Debug:Removing directory '{path}'", path = p2s!(located));
    remove_dir_all(located)?;
    log_ok_last!("Info:Removing old package...");

    log!("Info:Deploying files...");
    log!(
        "Debug:Deploying from '{src}' to '{dst}'",
        src = p2s!(temp_dir.join(name)),
        dst = p2s!(located)
    );
    move_or_copy(temp_dir.join(name), located.to_path_buf())?;
    log_ok_last!("Info:Deploying files...");
    Ok(())
}

// 执行新包的 update 或 setup 工作流
fn run_new_workflow(
    cfg: &Cfg,
    temp_dir: &Path,
    located: &Path,
    fresh_pkg: GlobalPackage,
) -> Result<()> {
    let update_path = temp_dir.join(DIR_WORKFLOWS).join(WORKFLOW_UPDATE);
    let located_str = p2s!(located);

    if update_path.exists() {
        log!("Info:Running update workflow...");
        let update_workflow = parse_workflow(&p2s!(update_path))?;
        workflow_executor(cfg.clone(), update_workflow, located_str, fresh_pkg)?;
        log_ok_last!("Info:Running update workflow...");
    } else {
        log!("Info:Running setup workflow...");
        let setup_path = update_path.with_file_name(WORKFLOW_SETUP);
        let setup_workflow = parse_workflow(&p2s!(setup_path))?;
        workflow_executor(cfg.clone(), setup_workflow, located_str, fresh_pkg)?;
        log_ok_last!("Info:Running setup workflow...");
    }
    Ok(())
}

pub fn update_using_package(
    cfg: &Cfg,
    source_file: &str,
    verify_signature: bool,
) -> Result<UpdateInfo> {
    log!("Info:Preparing to update with package '{source_file}'");

    // 解包
    let (temp_dir_inner_path, fresh_package) = unpack_nep(cfg, source_file, verify_signature)?;
    let name = fresh_package.package.name.clone();
    let fresh_scope = fresh_package.package.scope.clone();
    log!(
        "Debug:Unpacked package '{name}' version '{ver}' from '{source_file}'",
        ver = fresh_package.package.version
    );

    // 验证包是否已安装
    log!("Info:Resolving package...");
    let (local_package, local_diff) = info_local(cfg, &fresh_scope, &name).map_err(|e| {
        anyhow!("Error:Package '{name}' hasn't been installed or installation broken, use 'ept install' or 'ept uninstall' instead : '{e}'")
    })?;

    // 验证版本
    validate_version_update(&name, &local_diff.version, &fresh_package.package.version)?;

    // 处理作者不匹配
    if let Some(result) = handle_author_mismatch(
        cfg,
        source_file,
        &local_package,
        &fresh_package,
        local_diff.version.clone(),
        verify_signature,
    )? {
        return Ok(result);
    }

    let located = get_path_apps(cfg, &local_package.package.scope, &name, false)?;
    log!(
        "Debug:Located installation at '{path}'",
        path = p2s!(&located)
    );
    log_ok_last!("Info:Resolving package...");

    // 执行工作流转换
    log!("Debug:Running workflow transitions for update");
    run_old_remove_if_needed(cfg, &located, &temp_dir_inner_path, &local_package)?;
    reverse_setup_workflow(cfg, &located, local_package)?;

    // 如有展开工作流则执行
    let temp_dir_inner = p2s!(temp_dir_inner_path);
    if is_workshop_expandable(&temp_dir_inner) {
        expand_workshop(cfg, &temp_dir_inner)?;
    }

    // 部署并运行新工作流
    deploy_update(&temp_dir_inner_path, &located, &name)?;
    run_new_workflow(cfg, &temp_dir_inner_path, &located, fresh_package.clone())?;

    // 保存上下文并验证
    let ctx_path = located.join(DIR_NEP_CONTEXT);
    move_or_copy(temp_dir_inner_path, ctx_path)?;

    log!("Info:Validating update...");
    let located_str = p2s!(located);
    installed_validator(&located_str)?;
    log_ok_last!("Info:Validating update...");

    clean_temp(cfg, source_file)?;

    Ok(UpdateInfo {
        name,
        scope: fresh_scope,
        from_version: local_diff.version,
        to_version: fresh_package.package.version,
    })
}

pub fn update_using_url(cfg: &Cfg, url: &str, verify_signature: bool) -> Result<UpdateInfo> {
    // 下载文件到临时目录
    let cache_path = get_path_cache(cfg)?;
    let url_hash = compute_hash_blake3_from_string(url)?;
    let (p, cache_ctx) = download_nep(cfg, url, Some((cache_path, url_hash)))?;

    // 更新
    let info = update_using_package(cfg, &p2s!(p), verify_signature)?;

    // 缓存下载的包
    spawn_cache(cache_ctx)?;

    Ok(info)
}

pub fn update_using_package_matcher(
    cfg: &Cfg,
    matcher: String,
    verify_signature: bool,
) -> Result<UpdateInfo> {
    // 解析
    let parsed = parse_update_inputs(cfg, vec![matcher], verify_signature)?;
    // 执行更新
    if let ParseInputResEnum::PackageMatcher(p) = &parsed.first().unwrap().0 {
        update_using_url(cfg, &p.download_url, verify_signature)
    } else {
        Err(anyhow!(
            "Error:Fatal:Input matcher can't be parsed as package matcher"
        ))
    }
}

pub fn update_using_parsed(
    cfg: &Cfg,
    parsed: Vec<ParseInputResEnum>,
    verify_signature: bool,
) -> Result<Vec<UpdateInfo>> {
    let mut arr = Vec::new();
    let total = parsed.len();
    log!("Debug:Starting batch update for {total} packages");
    for (idx, parsed) in parsed.into_iter().enumerate() {
        log!(
            "Info:Start updating ({idx}/{total}) with {}",
            parsed.preview()
        );
        let res = match parsed {
            ParseInputResEnum::LocalPath(p, temp_dir) => {
                if let Some(temp_dir) = temp_dir {
                    update_using_package(cfg, &p2s!(temp_dir), false)?
                } else {
                    update_using_package(cfg, &p, verify_signature)?
                }
            }
            ParseInputResEnum::Url(u, temp_dir) => {
                if let Some(temp_dir) = temp_dir {
                    update_using_package(cfg, &p2s!(temp_dir), false)?
                } else {
                    update_using_url(cfg, &u, verify_signature)?
                }
            }
            ParseInputResEnum::PackageMatcher(p) => {
                update_using_url(cfg, &p.download_url, verify_signature)?
            }
        };
        log!("{}", res.format_success());
        arr.push(res);
    }
    Ok(arr)
}

pub fn update_all(cfg: &Cfg, verify_signature: bool) -> Result<(i32, i32)> {
    // 遍历 list 结果，生成更新列表
    let list_res = list(cfg)?;
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
                scope: node.scope.to_owned(),
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
            cfg,
            format!("Ready to update those {count} packages, continue?"),
            true,
        ) {
            return Err(anyhow!("Error:Operation canceled by user"));
        }
    }

    // 依次更新
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut temp_auto_confirm_cfg = cfg.clone();
    temp_auto_confirm_cfg.interaction.auto_confirm_all = true;
    for info in update_list {
        let res = update_using_package_matcher(
            &temp_auto_confirm_cfg,
            format!("{}/{}", info.scope, info.name),
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
    crate::utils::test::_ensure_clear_test_dir();

    use crate::utils::test::_default_test_cfg;
    let cfg = &_default_test_cfg();

    // 卸载
    crate::utils::test::_ensure_testing_vscode_uninstalled();

    // 安装旧版本
    crate::pack(
        cfg,
        "./examples/VSCode",
        Some("./test/VSCode_1.75.0.0_Cno.nep".to_string()),
        true,
    )
    .unwrap();
    install_using_package(cfg, "./test/VSCode_1.75.0.0_Cno.nep", true).unwrap();

    // 手动更新版本号
    crate::utils::fs::copy_dir("examples/VSCode", "test/VSCode").unwrap();
    crate::utils::test::_modify_package_dir_version("test/VSCode", "1.75.4.1");

    // 更新文件
    let old_ico = get_path_apps(cfg, "Microsoft", "VSCode", false)
        .unwrap()
        .join("favicon.ico");
    let new_ico = get_path_apps(cfg, "Microsoft", "VSCode", false)
        .unwrap()
        .join("icon.ico");
    assert!(old_ico.exists());
    std::fs::rename(
        "test/VSCode/VSCode/favicon.ico",
        "test/VSCode/VSCode/icon.ico",
    )
    .unwrap();

    // 安装新版本
    update_using_package(cfg, "test/VSCode", false).unwrap();
    assert!(!old_ico.exists());
    assert!(new_ico.exists());

    // 卸载
    crate::uninstall(cfg, None, "VSCode").unwrap();
}

#[test]
fn test_update_all() {
    let tup = crate::utils::test::_mount_custom_mirror();
    let (_, mut handler) = crate::utils::test::_run_static_file_server();
    crate::utils::test::_ensure_clear_test_dir();

    use crate::utils::test::_default_test_cfg;
    let mut cfg = _default_test_cfg();

    // 确保已卸载
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "Notepad");

    // 生成旧的 Notepad 包
    crate::utils::fs::copy_dir("examples/Notepad", "test/Notepad").unwrap();
    crate::utils::test::_modify_package_dir_version("test/Notepad", "22.0.0.0");

    // 安装旧版本
    install_using_package(&cfg, "examples/VSCode", false).unwrap();
    install_using_package(&cfg, "test/Notepad", false).unwrap();

    // 生成新包
    let source_dir = crate::utils::test::_fork_example_with_version("examples/VSCode", "1.75.4.2");
    std::fs::create_dir("test/static").unwrap();
    crate::pack(
        &cfg,
        &source_dir,
        Some("./test/static/VSCode_1.75.4.2_Cno.nep".to_string()),
        false,
    )
    .unwrap();
    crate::pack(
        &cfg,
        "./examples/Notepad",
        Some("./test/static/Notepad_22.1.0.0_Cno.nep".to_string()),
        false,
    )
    .unwrap();

    // 更新全部
    let (_, failure_count) = update_all(&mut cfg, false).unwrap();
    assert_eq!(failure_count, 0);
    assert_eq!(
        info_local(&cfg, "Microsoft", "VSCode").unwrap().1.version,
        *"1.75.4.2"
    );
    assert_eq!(
        info_local(&cfg, "Microsoft", "Notepad").unwrap().1.version,
        *"22.1.0.0"
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
    use crate::utils::test::_default_test_cfg;
    use std::path::Path;

    let desktop = crate::utils::env::env_desktop().unwrap();
    assert!(crate::utils::wild_match::parse_wild_match("vsc*.lnk", &desktop).is_err());
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

    let cfg = &_default_test_cfg();

    for (old_type, new_type, assert_files) in test_arr {
        log!("Info:Testing updating {old_type} -> {new_type}");
        // 卸载
        crate::utils::test::_ensure_testing_vscode_uninstalled();

        // 安装旧版本
        crate::entrances::install_using_package(
            cfg,
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
        crate::entrances::update_using_package(cfg, &source_file, false).unwrap();

        // 断言仅存在指定文件
        for file in assert_files {
            let p = Path::new(&desktop).join(format!("{file}.lnk"));
            assert!(p.exists());
            std::fs::remove_file(p).unwrap();
        }
        assert!(crate::utils::wild_match::parse_wild_match("vsc*.lnk", &desktop).is_err());

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
    use crate::utils::test::_default_test_cfg;

    let desktop = crate::utils::env::env_desktop().unwrap();
    assert!(crate::utils::wild_match::parse_wild_match("vsc*.lnk", &desktop).is_err());
    let desktop_path = std::path::Path::new(&desktop);

    let cfg = &_default_test_cfg();

    // 卸载
    crate::utils::test::_ensure_testing_vscode_uninstalled();

    // 安装旧版本
    crate::entrances::install_using_package(cfg, "examples/UpdateSuit/VSCode3", false).unwrap();
    assert!(desktop_path.join("vsc3-setup-1.75.4.0.lnk").exists());

    // 安装新版本
    let source_file = crate::utils::test::_fork_example_with_version("examples/VSCode", "1.75.4.2");
    std::fs::copy(
        "examples/UpdateSuit/VSCode2/workflows/update.toml",
        std::path::Path::new(&source_file).join("workflows/update.toml"),
    )
    .unwrap();
    update_using_package(cfg, &source_file, false).unwrap();

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
    use crate::utils::test::_default_test_cfg;
    use std::path::Path;
    crate::utils::test::_ensure_clear_test_dir();
    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "VSCodeE");

    let cfg = &_default_test_cfg();

    // 断言原来的包中不包含这个二进制文件
    assert!(!Path::new("examples/VSCodeE/VSCodeE/Code.exe").exists());

    // 启动文件服务器
    let (_addr, mut handler) = crate::utils::test::_run_static_file_server();
    std::fs::copy("examples/VSCode/VSCode/Code.exe", "test/Code.exe").unwrap();

    // 安装
    crate::utils::fs::copy_dir("examples/VSCodeE", "test/VSCodeE").unwrap();
    install_using_package(cfg, "test/VSCodeE", false).unwrap();

    // 断言安装成功
    assert!(info_local(cfg, "Microsoft", "VSCodeE").is_ok());
    let app_exe_path = get_path_apps(cfg, "Microsoft", "VSCodeE", false)
        .unwrap()
        .join("Code.exe");
    assert!(app_exe_path.exists());

    // 手动删除一个依赖文件
    let ico_path = get_path_apps(cfg, "Microsoft", "VSCodeE", false)
        .unwrap()
        .join("favicon.ico");
    std::fs::remove_file(&ico_path).unwrap();
    assert!(app_exe_path.exists());
    assert!(!ico_path.exists());

    // 生成一个更新包
    let pkg_path = crate::utils::test::_fork_example_with_version("examples/VSCodeE", "1.75.5.0");

    // 断言原来的包中不包含这个二进制文件
    assert!(!Path::new(&pkg_path).join("VSCodeE/Code.exe").exists());

    // 安装更新包
    update_using_package(cfg, &pkg_path, false).unwrap();

    // 断言安装成功
    assert!(info_local(cfg, "Microsoft", "VSCodeE").unwrap().1.version == "1.75.5.0");
    assert!(app_exe_path.exists());
    assert!(ico_path.exists());

    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "VSCodeE");
    handler.kill().unwrap();
}

#[test]
fn test_update_offline() {
    use crate::utils::test::_default_test_cfg;

    let cfg = &_default_test_cfg();
    crate::utils::test::_ensure_testing_vscode();
    assert!(update_using_package(cfg, "examples/vscode", true).is_err());
}
