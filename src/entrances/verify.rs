use crate::compression::{compress, pack_tar};
use crate::entrances::verify::verify;
use crate::parsers::{parse_author, parse_package, parse_workflow};
use crate::signature::sign;
use crate::types::mixed_fs::MixedFS;
use crate::types::verifiable::Verifiable;
use crate::types::{signature::Signature, signature::SignatureNode, workflow::WorkflowNode};
use crate::utils::{ask_yn, get_path_temp, is_debug_mode};
use crate::{log, log_ok_last, p2s};
use anyhow::{anyhow, Result};
use std::fs::{read_dir, remove_dir_all, write};
use std::path::{Path, PathBuf};

use super::utils::validator::{inner_validator, manifest_validator};


fn get_manifest(flow: Vec<WorkflowNode>, fs: &mut MixedFS) -> Vec<String> {
    let mut manifest = Vec::new();
    for node in flow {
        manifest.append(&mut node.body.get_manifest(fs));
    }
    manifest
}

fn get_workflow_path(source_dir:&String,file_name:&str)->PathBuf{
    Path::new(source_dir).join("workflows").join(name).to_path_buf()
}

fn verify_workflow(flow: Vec<WorkflowNode>)->Result<()>{
    for node in flow{
        node.verify_self()?;
    }
    Ok(())
}

pub fn verify(source_dir:&String)->Result<()>{
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
    let first_author = parse_author(&global.package.authors[0])?;
    let file_stem = format!(
        "{pn}_{pv}_{fa}",
        pn = global.package.name,
        pv = global.package.version,
        fa = first_author.name
    );
    log_ok_last!("Info:Resolving data...");

    // 校验 setup 工作流装箱单
    log!("Info:Checking manifest...");
    let setup_path = get_workflow_path(source_dir, "setup.toml");
    let setup_flow = parse_workflow(&p2s!(setup_path))?;
    let mut fs = MixedFS::new();
    let setup_manifest = get_manifest(setup_flow.clone(), &mut fs);
    let pkg_content_path = Path::new(source_dir).join(&global.package.name);
    manifest_validator(&p2s!(pkg_content_path), setup_manifest, &mut fs)?;
    log_ok_last!("Info:Checking manifest...");

    // 校验工作流
    verify_workflow(setup_flow)?;
    let optional_path:Vec<PathBuf>=vec!["update.toml","remove.toml"]
    .into_iter()
    .map(|name|get_workflow_path(source_dir, name))
    .collect();
    for opt_path in optional_path{
        if opt_path.exists(){
            let flow=parse_workflow(&p2s!(opt_path))?;
            verify_workflow(flow)?;
        }
    }

    Ok(())
}