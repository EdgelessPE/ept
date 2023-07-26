use crate::parsers::{parse_package, parse_workflow};
use crate::types::mixed_fs::MixedFS;
use crate::types::package::GlobalPackage;
use crate::types::steps::Step;
use crate::types::verifiable::Verifiable;
use crate::types::workflow::WorkflowNode;
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
    manifest
}

fn get_workflow_path(source_dir: &String, file_name: &str) -> PathBuf {
    Path::new(source_dir)
        .join("workflows")
        .join(file_name)
        .to_path_buf()
}

// 返回是否调用了 call_installer
fn verify_workflow(flow: Vec<WorkflowNode>, located: &String) -> Result<bool> {
    let mut have_call_installer = false;
    for node in flow {
        node.verify_self(located)?;
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
    let global = parse_package(&p2s!(pkg_path), None)?;
    let pkg_content_path = p2s!(Path::new(source_dir).join(&global.package.name));
    global.verify_self(&pkg_content_path)?;
    log_ok_last!("Info:Resolving data...");

    // 校验工作流
    log!("Info:Verifying workflows...");
    let setup_path = get_workflow_path(source_dir, "setup.toml");
    let setup_flow = parse_workflow(&p2s!(setup_path))?;

    // 记录 setup 中是否用到 call_installer
    let check_call_installer = verify_workflow(setup_flow.clone(), &pkg_content_path)?;

    // 用到了 call_installer 必须有卸载流
    if check_call_installer && !get_workflow_path(source_dir, "remove.toml").exists() {
        return Err(anyhow!("Error:Workflow 'remove.toml' should include 'Execute' step with 'call_installer' field enabled when workflow 'setup.toml' includes such step"));
    }

    // 检查其他工作流
    let optional_workflows = vec!["update.toml", "remove.toml"];
    for opt_workflow in optional_workflows {
        let opt_path = get_workflow_path(source_dir, opt_workflow);
        if opt_path.exists() {
            let flow = parse_workflow(&p2s!(opt_path))?;
            let call_installer = verify_workflow(flow, source_dir)?;
            if check_call_installer && !call_installer {
                return Err(anyhow!("Error:Workflow '{opt_workflow}' should include 'Execute' step with 'call_installer' field enabled when workflow 'setup.toml' includes such step"));
            }
        }
    }
    log_ok_last!("Info:Verifying workflows...");

    // 校验 setup 工作流装箱单
    log!("Info:Checking manifest...");
    let mut fs = MixedFS::new(pkg_content_path.clone());
    let setup_manifest = get_manifest(setup_flow, &mut fs);
    manifest_validator(&pkg_content_path, setup_manifest, &mut fs)?;
    log_ok_last!("Info:Checking manifest...");

    Ok(global)
}

#[test]
fn test_verify() {
    envmnt::set("DEBUG", "true");
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
    ).unwrap();
    assert!(verify(&"./examples/CallInstaller".to_string()).is_err());
    std::fs::rename(
        "examples/CallInstaller/workflows/_remove.toml",
        "examples/CallInstaller/workflows/remove.toml",
    ).unwrap();
}
