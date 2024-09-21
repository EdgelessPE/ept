use crate::parsers::{parse_package, parse_workflow};
use crate::types::extended_semver::ExSemVer;
use crate::types::mixed_fs::MixedFS;
use crate::types::package::GlobalPackage;
use crate::types::steps::{Step, VerifyStepCtx};
use crate::types::workflow::WorkflowNode;
use crate::utils::exe_version::get_exe_version;
use crate::utils::is_starts_with_inner_value;
use crate::utils::path::parse_relative_path_with_located;
use crate::{log, log_ok_last, p2s};
use anyhow::{anyhow, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use super::utils::validator::{inner_validator, manifest_validator};

fn get_manifest(flow: Vec<WorkflowNode>, fs: &mut MixedFS) -> Vec<String> {
    let mut manifest = Vec::new();
    for node in flow {
        manifest.append(&mut node.body.get_manifest(fs));
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

fn get_workflow_path(source_dir: &String, file_name: &str) -> PathBuf {
    Path::new(source_dir)
        .join("workflows")
        .join(file_name)
        .to_path_buf()
}

// 返回是否调用了 call_installer
fn verify_workflow(flow: Vec<WorkflowNode>, ctx: &VerifyStepCtx) -> Result<bool> {
    let mut have_call_installer = false;
    for node in flow {
        node.verify_step(ctx)?;
        if let Step::StepExecute(step) = node.body {
            if !have_call_installer {
                have_call_installer = step.call_installer.unwrap_or(false);
            }
        }
    }
    Ok(have_call_installer)
}

pub fn verify(source_dir: &String) -> Result<GlobalPackage> {
    // 打包检查
    log!("Info:Validating source directory...");
    // 如果目录中文件数量超过 3 个则拒绝
    let dir_list = read_dir(source_dir)?;
    let dir_count = dir_list.into_iter().fold(0, |acc, _| acc + 1);
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
    let pkg_path = Path::new(source_dir).join("package.toml");
    let global = parse_package(&p2s!(pkg_path), source_dir, false)?;
    let software = global.software.clone().unwrap();
    let pkg_content_path = p2s!(Path::new(source_dir).join(&global.package.name));
    log_ok_last!("Info:Resolving data...");

    // 校验工作流
    log!("Info:Verifying workflows...");
    let setup_path = get_workflow_path(source_dir, "setup.toml");
    let setup_flow = parse_workflow(&p2s!(setup_path))?;

    // 记录 setup 中是否用到 call_installer
    let check_call_installer = verify_workflow(
        setup_flow.clone(),
        &VerifyStepCtx {
            located: pkg_content_path.clone(),
            is_expand_flow: false,
        },
    )?;

    // 如果用到了 call_installer 则有一些特殊逻辑，除非提供了 registry_entry：
    if check_call_installer && software.registry_entry.is_none() {
        // 必须有卸载流
        if !get_workflow_path(source_dir, "remove.toml").exists() {
            return Err(anyhow!("Error:Workflow 'remove.toml' should include 'Execute' step with 'call_installer' field enabled when workflow 'setup.toml' includes such step"));
        }

        // 必须提供绝对路径的 main_program
        if let Some(mp) = software.main_program.clone() {
            if !Path::new(&mp).is_absolute() {
                return Err(anyhow!("Error:Field 'main_program' in table 'software' should starts with inner value when workflow 'setup.toml' includes 'Execute' step with 'call_installer' field, got '{mp}'"));
            }
        } else {
            return Err(anyhow!("Error:Field 'main_program' or 'registry_entry' in table 'software' should be provided when workflow 'setup.toml' includes 'Execute' step with 'call_installer' field"));
        }
    }

    // 检查更新、卸载工作流
    let optional_workflows = vec!["update.toml", "remove.toml"];
    let ctx = VerifyStepCtx {
        located: source_dir.to_string(),
        is_expand_flow: false,
    };
    for opt_workflow in optional_workflows {
        let opt_path = get_workflow_path(source_dir, opt_workflow);
        if opt_path.exists() {
            let flow = parse_workflow(&p2s!(opt_path))?;
            let call_installer = verify_workflow(flow, &ctx)?;
            if check_call_installer && !call_installer {
                return Err(anyhow!("Error:Workflow '{opt_workflow}' should include 'Execute' step with 'call_installer' field enabled when workflow 'setup.toml' includes such step"));
            }
        }
    }

    // 检查展开工作流
    let ctx = VerifyStepCtx {
        located: source_dir.to_string(),
        is_expand_flow: true,
    };
    let expand_path = get_workflow_path(source_dir, "expand.toml");
    if expand_path.exists() {
        let flow = parse_workflow(&p2s!(expand_path))?;
        verify_workflow(flow, &ctx)?;
    }

    log_ok_last!("Info:Verifying workflows...");

    // 校验 setup 工作流装箱单
    log!("Info:Checking manifest...");
    let mut fs = MixedFS::new(pkg_content_path.clone());
    // 如果有展开工作流，先使用展开工作流跑一遍
    if expand_path.exists() {
        let expand_flow = parse_workflow(&p2s!(expand_path))?;
        let _expand_manifest = get_manifest(expand_flow, &mut fs);
    }
    let setup_manifest = get_manifest(setup_flow, &mut fs);
    manifest_validator(&pkg_content_path, setup_manifest, &mut fs)?;
    log_ok_last!("Info:Checking manifest...");

    // 如果显式提供了相对路径的主程序，检查该主程序是否可以正常读取版本号
    if let Some(mp) = software.main_program {
        if !mp.starts_with("${") && Path::new(&mp).is_relative() {
            let mp_path = parse_relative_path_with_located(
                &format!("{name}/{mp}", name = global.package.name),
                source_dir,
            );
            log!(
                "Debug:Main program path : '{}',with source_dir = '{source_dir}'",
                p2s!(mp_path)
            );
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

    Ok(global)
}

#[test]
fn test_verify() {
    use crate::utils::envmnt;
    envmnt::set("DEBUG", "true");
    use std::fs::write;
    verify(&"./examples/VSCode".to_string()).unwrap();
    verify(&"./examples/CallInstaller".to_string()).unwrap();

    // 手动添加没有 call_installer 的 update.toml
    std::fs::copy(
        "./examples/VSCode/workflows/setup.toml",
        "./examples/CallInstaller/workflows/update.toml",
    )
    .unwrap();
    assert!(verify(&"./examples/CallInstaller".to_string()).is_err());
    std::fs::remove_file("./examples/CallInstaller/workflows/update.toml").unwrap();

    // 调用了 call_installer 但是不提供 remove.toml
    std::fs::rename(
        "examples/CallInstaller/workflows/remove.toml",
        "examples/CallInstaller/workflows/_remove.toml",
    )
    .unwrap();
    assert!(verify(&"./examples/CallInstaller".to_string()).is_err());
    std::fs::rename(
        "examples/CallInstaller/workflows/_remove.toml",
        "examples/CallInstaller/workflows/remove.toml",
    )
    .unwrap();

    // 保存现场
    let package_scene = std::fs::read_to_string("examples/CallInstaller/package.toml").unwrap();
    // 读取 package
    let pkg_path = &"examples/CallInstaller/package.toml".to_string();
    let mut raw_pkg =
        parse_package(pkg_path, &"examples/CallInstaller".to_string(), false).unwrap();

    // 删除 CallInstaller 的 main_program
    raw_pkg.software = raw_pkg.software.map(|mut soft| {
        soft.main_program = None;
        soft
    });
    write(pkg_path, toml::to_string_pretty(&raw_pkg).unwrap()).unwrap();
    assert!(verify(&"./examples/CallInstaller".to_string()).is_err());

    // 令 CallInstaller 的 main_program 为相对路径
    raw_pkg.software = raw_pkg.software.map(|mut soft| {
        soft.main_program = Some("Installer.exe".to_string());
        soft
    });
    write(pkg_path, toml::to_string_pretty(&raw_pkg).unwrap()).unwrap();
    assert!(verify(&"./examples/CallInstaller".to_string()).is_err());

    // 还原现场
    write(pkg_path, package_scene).unwrap();
}
