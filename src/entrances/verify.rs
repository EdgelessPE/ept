use crate::parsers::{parse_package, parse_workflow};
use crate::types::mixed_fs::MixedFS;
use crate::types::package::GlobalPackage;
use crate::types::verifiable::Verifiable;
use crate::types::workflow::WorkflowNode;
use crate::{log, log_ok_last, p2s};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use super::utils::validator::{inner_validator, manifest_validator};

fn get_manifest(flow: Vec<WorkflowNode>, fs: &mut MixedFS) -> (Vec<String>, bool) {
    let mut manifest = Vec::new();
    // TEMP:手动维护一份会导致文件系统新增文件的步骤名称列表，检测到白名单步骤时对manifest检测异常警告而非报错
    let white_list = HashSet::from(["Copy", "Move", "Rename", "New"]);
    let mut var_warn_manifest = false;

    for node in flow {
        manifest.append(&mut node.body.get_manifest(fs));
        if white_list.contains(node.header.step.as_str()) {
            var_warn_manifest = true;
        }
    }
    (manifest, var_warn_manifest)
}

fn get_workflow_path(source_dir: &String, file_name: &str) -> PathBuf {
    Path::new(source_dir)
        .join("workflows")
        .join(file_name)
        .to_path_buf()
}

fn verify_workflow(flow: Vec<WorkflowNode>, located: &String) -> Result<()> {
    for node in flow {
        node.verify_self(located)?;
    }
    Ok(())
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
    verify_workflow(setup_flow.clone(), &pkg_content_path)?;
    let optional_path: Vec<PathBuf> = vec!["update.toml", "remove.toml"]
        .into_iter()
        .map(|name| get_workflow_path(source_dir, name))
        .collect();
    for opt_path in optional_path {
        if opt_path.exists() {
            let flow = parse_workflow(&p2s!(opt_path))?;
            verify_workflow(flow, source_dir)?;
        }
    }
    log_ok_last!("Info:Verifying workflows...");

    // 校验 setup 工作流装箱单
    log!("Info:Checking manifest...");
    let mut fs = MixedFS::new(pkg_content_path.clone());
    let (setup_manifest, var_warn_manifest) = get_manifest(setup_flow, &mut fs);
    manifest_validator(
        &pkg_content_path,
        setup_manifest,
        &mut fs,
        var_warn_manifest,
    )?;
    log_ok_last!("Info:Checking manifest...");

    Ok(global)
}

#[test]
fn test_verify() {
    envmnt::set("DEBUG", "true");
    verify(&r"D:\Desktop\Projects\EdgelessPE\edgeless-bot\workshop\adb\_ready".to_string())
        .unwrap();
}
