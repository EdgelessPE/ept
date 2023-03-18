use crate::compression::{compress, pack_tar};
use crate::parsers::{parse_author, parse_package, parse_workflow};
use crate::signature::sign;
use crate::types::{Signature, SignatureNode, WorkflowNode};
use crate::utils::{ask_yn, get_path_temp, is_debug_mode};
use crate::{log, log_ok_last, p2s};
use anyhow::{anyhow, Result};
use std::fs::{create_dir_all, read_dir, remove_dir_all, write};
use std::path::Path;

use super::utils::{inner_validator, manifest_validator};

fn get_manifest(flow: Vec<WorkflowNode>) -> Vec<String> {
    let mut manifest = Vec::new();
    for node in flow {
        manifest.append(&mut node.body.get_manifest());
    }
    manifest
}

pub fn pack(source_dir: String, into_file: Option<String>, need_sign: bool) -> Result<String> {
    log!("Info:Preparing to pack '{}'", &source_dir);

    // 打包检查
    log!("Info:Validating source directory...");
    // 如果目录中文件数量超过 3 个则拒绝
    let dir_list = read_dir(&source_dir)?;
    let dir_count = dir_list.into_iter().fold(0, |acc, _| acc + 1);
    if dir_count != 3 {
        return Err(anyhow!(
            "Error:Expected 3 items in '{}', got {} items",
            &source_dir,
            dir_count
        ));
    }
    // 运行内包检查器
    inner_validator(source_dir.clone())?;
    log_ok_last!("Info:Validating source directory...");

    // 读取包信息
    log!("Info:Resolving data...");
    let pkg_path = Path::new(&source_dir).join("package.toml");
    let global = parse_package(p2s!(pkg_path), None)?;
    let first_author = parse_author(global.package.authors[0].to_owned())?;
    let file_stem = format!(
        "{}_{}_{}",
        &global.package.name, &global.package.version, &first_author.name
    );
    let into_file = into_file.unwrap_or(String::from("./") + &file_stem + ".nep");
    log_ok_last!("Info:Resolving data...");

    // 校验 setup 流装箱单
    log!("Info:Checking manifest...");
    let setup_path = Path::new(&source_dir).join("workflows").join("setup.toml");
    let setup_flow = parse_workflow(p2s!(setup_path))?;
    let setup_manifest = get_manifest(setup_flow);
    let pkg_content_path = Path::new(&source_dir).join(&global.package.name);
    manifest_validator(p2s!(pkg_content_path), setup_manifest)?;
    log_ok_last!("Info:Checking manifest...");

    // 校验 into_file 是否存在
    let into_file_path = Path::new(&into_file);
    if into_file_path.exists() {
        if into_file_path.is_dir() {
            return Err(anyhow!(
                "Error:Target '{}' is a existing directory",
                &into_file
            ));
        } else {
            log!(
                "Warning:Overwrite the existing file '{}'? (y/n)",
                &into_file
            );
            if !ask_yn() {
                return Err(anyhow!("Error:Pack canceled by user"));
            }
        }
    }

    // 创建临时目录
    let temp_dir_path = get_path_temp(&file_stem);
    if temp_dir_path.exists() {
        remove_dir_all(&temp_dir_path)?;
    }
    create_dir_all(&temp_dir_path)?;

    // 生成内包
    log!("Info:Compressing inner package...");
    let inner_path_str = p2s!(temp_dir_path.join(&(file_stem.clone() + ".tar.zst")));
    compress(source_dir, inner_path_str.clone())?;
    log_ok_last!("Info:Compressing inner package...");

    // 对内包进行签名
    let signature = if need_sign {
        log!("Info:Signing inner package...");
        let signature = sign(inner_path_str.clone())?;
        Some(signature)
    } else {
        None
    };
    let sign_file_path = temp_dir_path.join("signature.toml");
    let signature_struct = Signature {
        package: SignatureNode {
            signer: first_author.email.unwrap(),
            signature,
        },
    };
    let text = toml::to_string_pretty(&signature_struct)?;
    write(sign_file_path, &text)?;
    if need_sign {
        log_ok_last!("Info:Signing inner package...");
    } else {
        log!("Warning:Signing has been disabled!")
    }

    // 生成外包
    log!("Info:Packing outer package...");
    pack_tar(p2s!(temp_dir_path), into_file.clone())?;
    log_ok_last!("Info:Packing outer package...");

    // 清理临时文件夹
    if !is_debug_mode() {
        log!("Info:Cleaning...");
        let clean_res = remove_dir_all(&temp_dir_path);
        if clean_res.is_ok() {
            log_ok_last!("Info:Cleaning...");
        } else {
            log!(
                "Warning:Failed to remove temporary directory '{}'",
                p2s!(temp_dir_path)
            );
        }
    } else {
        log!(
            "Debug:Leaving temporary directory '{}'",
            p2s!(temp_dir_path)
        );
    }

    Ok(into_file)
}

#[test]
fn test_pack() {
    envmnt::set("DEBUG", "true");
    pack(
        r"D:\Desktop\VSCode_1.75.0.0_Cno".to_string(),
        Some(r"D:\Desktop\VSCode_1.75.0.0_Cno.nep".to_string()),
        true,
    )
    .unwrap();
}
