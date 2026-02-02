use crate::parsers::{parse_package, parse_workflow};
use crate::types::constants::DIR_WORKFLOWS;
use crate::types::{
    constants::{FILE_PACKAGE, WORKFLOW_EXPAND, WORKFLOW_REMOVE, WORKFLOW_SETUP, WORKFLOW_UPDATE},
    context::VerifyStepCtx,
    extended_semver::ExSemVer,
    mixed_fs::MixedFS,
    package::GlobalPackage,
    steps::Step,
    workflow::WorkflowNode,
};
use crate::utils::exe_version::get_exe_version;
use crate::utils::is_starts_with_inner_value;
use crate::utils::path::parse_relative_path_with_located;
use crate::{log, log_ok_last, p2s};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use super::utils::validator::{inner_validator, manifest_validator};
use crate::types::context::RuntimeContext;

fn get_manifest(flow: Vec<WorkflowNode>, fs: &mut MixedFS) -> Vec<String> {
    let mut manifest = Vec::new();
    let mut set = HashSet::new();
    for node in flow {
        for item in node.body.get_manifest(fs) {
            if set.contains(&item) {
                continue;
            }
            set.insert(item.clone());
            manifest.push(item);
        }
    }
    debug_assert!(manifest.clone().into_iter().fold(true, |state, cur| {
        if !state {
            state
        } else if is_starts_with_inner_value(&cur) {
            log!("Error:Fatal:Got absolute manifest '{cur}'");
            false
        } else {
            true
        }
    }));
    manifest
}

fn get_workflow_path(source_dir: &str, file_name: &str) -> PathBuf {
    Path::new(source_dir)
        .join(DIR_WORKFLOWS)
        .join(file_name)
        .to_path_buf()
}

// 返回是否调用了 call_installer
fn verify_workflow(cx: &VerifyStepCtx, flow: Vec<WorkflowNode>) -> Result<bool> {
    let mut have_call_installer = false;
    for node in flow {
        node.verify_step(cx)?;
        if let Step::StepExecute(step) = node.body {
            if !have_call_installer {
                have_call_installer = step.call_installer.unwrap_or(false);
            }
        }
    }
    Ok(have_call_installer)
}

pub fn verify(ctx: &RuntimeContext, source_dir: &str) -> Result<GlobalPackage> {
    log!("Debug:Starting verification for source directory '{source_dir}'");
    // 打包检查
    log!("Info:Validating source directory...");
    // 如果目录中文件数量超过 3 个则拒绝
    let dir_list = read_dir(source_dir)?;
    let dir_count = dir_list.into_iter().fold(0, |acc, _| acc + 1);
    log!("Debug:Found {dir_count} items in source directory");
    if dir_count != 3 {
        return Err(anyhow!(
            "Error:Expected 3 items in '{source_dir}', got {dir_count} items"
        ));
    }
    // 运行内包检查器
    inner_validator(source_dir)?;
    log_ok_last!("Info:Validating source directory...");

    // 读取包信息
    log!("Info:Resolving data...");
    let pkg_path = Path::new(source_dir).join(FILE_PACKAGE);
    let global = parse_package(ctx, &p2s!(pkg_path), source_dir, false)?;
    let software = global.software.clone().unwrap();
    let pkg_content_path = p2s!(Path::new(source_dir).join(&global.package.name));
    log!(
        "Debug:Resolved package '{name}' version '{ver}' from '{source_dir}'",
        name = global.package.name,
        ver = global.package.version
    );
    log_ok_last!("Info:Resolving data...");

    // 校验工作流
    log!("Info:Verifying workflows...");
    let setup_path = get_workflow_path(source_dir, WORKFLOW_SETUP);
    log!(
        "Debug:Parsing setup workflow at '{path}'",
        path = p2s!(&setup_path)
    );
    let setup_flow = parse_workflow(&p2s!(setup_path))?;

    // 记录 setup 中是否用到 call_installer
    let check_call_installer = verify_workflow(
        &VerifyStepCtx {
            mixed_fs: &MixedFS::new(&pkg_content_path),
            runtime_ctx: ctx,
            is_expand_flow: false,
        },
        setup_flow.clone(),
    )?;

    // 如果用到了 call_installer 则有一些特殊逻辑，除非提供了 registry_entry：
    if check_call_installer && software.registry_entry.is_none() {
        // 必须有卸载流
        if !get_workflow_path(source_dir, WORKFLOW_REMOVE).exists() {
            return Err(anyhow!("Error:Workflow '{WORKFLOW_REMOVE}' should include 'Execute' step with 'call_installer' field enabled when workflow '{WORKFLOW_SETUP}' includes such step"));
        }

        // 必须提供绝对路径的 main_program
        if let Some(mp) = software.main_program.clone() {
            if !Path::new(&mp).is_absolute() {
                return Err(anyhow!("Error:Field 'main_program' in table 'software' should starts with inner value when workflow '{WORKFLOW_SETUP}' includes 'Execute' step with 'call_installer' field, got '{mp}'"));
            }
        } else {
            return Err(anyhow!("Error:Field 'main_program' or 'registry_entry' in table 'software' should be provided when workflow '{WORKFLOW_SETUP}' includes 'Execute' step with 'call_installer' field"));
        }
    }

    // 检查更新、卸载工作流
    let optional_workflows = vec![WORKFLOW_UPDATE, WORKFLOW_REMOVE];
    let cx = VerifyStepCtx {
        mixed_fs: &MixedFS::new(source_dir),
        runtime_ctx: ctx,
        is_expand_flow: false,
    };
    for opt_workflow in optional_workflows {
        let opt_path = get_workflow_path(source_dir, opt_workflow);
        if opt_path.exists() {
            let flow = parse_workflow(&p2s!(opt_path))?;
            let call_installer = verify_workflow(&cx, flow)?;
            if check_call_installer && !call_installer {
                return Err(anyhow!("Error:Workflow '{opt_workflow}' should include 'Execute' step with 'call_installer' field enabled when workflow '{WORKFLOW_SETUP}' includes such step"));
            }
        }
    }

    // 检查展开工作流
    let cx = VerifyStepCtx {
        mixed_fs: &MixedFS::new(source_dir),
        runtime_ctx: ctx,
        is_expand_flow: true,
    };
    let expand_path = get_workflow_path(source_dir, WORKFLOW_EXPAND);
    if expand_path.exists() {
        let flow = parse_workflow(&p2s!(expand_path))?;
        verify_workflow(&cx, flow)?;
    }

    log_ok_last!("Info:Verifying workflows...");
    log!("Debug:All workflows verified successfully for '{source_dir}'");

    // 校验 setup 工作流装箱单
    log!("Info:Checking manifest...");
    let mut fs = MixedFS::new(&pkg_content_path);
    // 如果有展开工作流，先使用展开工作流跑一遍
    if expand_path.exists() {
        let expand_flow = parse_workflow(&p2s!(expand_path))?;
        let _expand_manifest = get_manifest(expand_flow, &mut fs);
    }
    let mut setup_manifest = get_manifest(setup_flow, &mut fs);
    // 加上 update 工作流的装箱单
    let update_path = get_workflow_path(source_dir, WORKFLOW_UPDATE);
    if update_path.exists() {
        let update_flow = parse_workflow(&p2s!(update_path))?;
        let mut update_manifest = get_manifest(update_flow, &mut fs);
        setup_manifest.append(&mut update_manifest);
    }
    manifest_validator(ctx, &pkg_content_path, setup_manifest, &mut fs)?;
    log_ok_last!("Info:Checking manifest...");
    log!("Debug:Manifest validation completed for '{pkg_content_path}'");

    // 如果显式提供了相对路径的主程序，检查该主程序是否可以正常读取版本号
    if let Some(mp) = software.main_program {
        // 仅对相对路径且不是尚未拓展的主程序进行检查
        if !mp.starts_with("${") && Path::new(&mp).is_relative() {
            let mp_path = parse_relative_path_with_located(
                &format!("{name}/{mp}", name = global.package.name),
                source_dir,
            );
            let is_virtual = !mp_path.exists() && fs.exists(&mp);
            log!(
                "Debug:Main program path : '{}',with source_dir = '{source_dir}', is_virtual = {is_virtual}",
                p2s!(mp_path)
            );
            if !is_virtual {
                let read_res = get_exe_version(mp_path);
                if let Ok(version) = read_res {
                    // 与申明的版本号进行比较，仅要求 semver 部分相等即可
                    let d_ver = ExSemVer::parse(&global.package.version)?;
                    let r_ver = ExSemVer::parse(&version)?;
                    if d_ver.semver_instance != r_ver.semver_instance {
                        return Err(anyhow!("Error:The version declared ({dv}) is inconsistent with the version obtained by the read main program ({version}), consider remove field 'software.main_program'",dv=&global.package.version));
                    }
                } else {
                    return Err(anyhow!(
                    "Error:Failed to read version of main program '{mp}', consider remove field 'software.main_program' : {e}",
                    e = read_res.unwrap_err()
                ));
                }
            }
        }
    }

    Ok(global)
}

#[test]
fn test_get_manifest() {
    let mut fs = MixedFS::new("./examples/VSCode");
    let setup_workflow = parse_workflow("examples/PermissionsTest/workflows/setup.toml").unwrap();
    assert_eq!(
        get_manifest(setup_workflow, &mut fs),
        vec![
            "bin".to_string(),
            "Code.exe".to_string(),
            "installer.exe".to_string()
        ]
    );

    let update_workflow = parse_workflow("examples/PermissionsTest/workflows/update.toml").unwrap();
    assert_eq!(
        get_manifest(update_workflow, &mut fs),
        vec!["bin".to_string(), "updater.exe".to_string()]
    );
}

#[test]
fn test_verify() {
    use crate::utils::flags::{set_flag, Flag};
    use crate::utils::test::_default_test_cfg;
    let cfg = _default_test_cfg();
    use std::fs::write;
    verify(&cfg, "./examples/VSCode").unwrap();
    verify(&cfg, "./examples/VSCodeE").unwrap();
    verify(&cfg, "./examples/CallInstaller").unwrap();
    verify(&cfg, "./examples/PermissionsTest").unwrap();

    // 手动添加没有 call_installer 的 update.toml
    std::fs::copy(
        "./examples/VSCode/workflows/setup.toml",
        "./examples/CallInstaller/workflows/update.toml",
    )
    .unwrap();
    assert!(verify(&cfg, "./examples/CallInstaller").is_err());
    std::fs::remove_file("./examples/CallInstaller/workflows/update.toml").unwrap();

    // 调用了 call_installer 但是不提供 remove.toml
    std::fs::rename(
        "examples/CallInstaller/workflows/remove.toml",
        "examples/CallInstaller/workflows/_remove.toml",
    )
    .unwrap();
    assert!(verify(&cfg, "./examples/CallInstaller").is_err());
    std::fs::rename(
        "examples/CallInstaller/workflows/_remove.toml",
        "examples/CallInstaller/workflows/remove.toml",
    )
    .unwrap();

    // 保存现场
    let package_scene = std::fs::read_to_string("examples/CallInstaller/package.toml").unwrap();
    // 读取 package
    let pkg_path = "examples/CallInstaller/package.toml";
    let mut raw_pkg = parse_package(&cfg, pkg_path, "examples/CallInstaller", false).unwrap();

    // 删除 CallInstaller 的 main_program
    raw_pkg.software = raw_pkg.software.map(|mut soft| {
        soft.main_program = None;
        soft
    });
    write(pkg_path, toml::to_string_pretty(&raw_pkg).unwrap()).unwrap();
    assert!(verify(&cfg, "./examples/CallInstaller").is_err());

    // 令 CallInstaller 的 main_program 为相对路径
    raw_pkg.software = raw_pkg.software.map(|mut soft| {
        soft.main_program = Some("Installer.exe".to_string());
        soft
    });
    write(pkg_path, toml::to_string_pretty(&raw_pkg).unwrap()).unwrap();
    assert!(verify(&cfg, "./examples/CallInstaller").is_err());

    // 还原现场
    write(pkg_path, package_scene).unwrap();
}
