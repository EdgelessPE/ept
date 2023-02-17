use std::{
    fs::{create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

use crate::{
    compression::{decompress, release_tar},
    parsers::{parse_author, parse_package, parse_signature},
    signature::verify,
    types::GlobalPackage,
    utils::{get_path_temp, is_debug_mode},
};
use crate::{log, log_ok_last};

use super::{inner_validator, outer_validator};

/// 根据源文件路径创建并返回(临时目录,文件茎)
fn get_temp_dir_path(source_file: String, keep_clear: bool) -> Result<(PathBuf, String)> {
    let file_stem = Path::new(&source_file)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let temp_dir_path = get_path_temp().join(&file_stem);
    if !keep_clear {
        return Ok((temp_dir_path, file_stem));
    }

    let temp_dir_outer_path = temp_dir_path.join("Outer");
    let temp_dir_inner_path = temp_dir_path.join("Inner");

    if temp_dir_path.exists() {
        remove_dir_all(&temp_dir_path)?;
    }
    create_dir_all(&temp_dir_outer_path)?;
    create_dir_all(&temp_dir_inner_path)?;

    Ok((temp_dir_path, file_stem))
}

/// 清理临时目录(会判断 debug)
pub fn clean_temp(source_file: String) -> Result<()> {
    let (temp_dir_path, _) = get_temp_dir_path(source_file, false)?;
    if !is_debug_mode() {
        log!("Info:Cleaning...");
        let clean_res = remove_dir_all(&temp_dir_path);
        if clean_res.is_ok() {
            log_ok_last!("Info:Cleaning...");
        } else {
            log!(
                "Warning:Failed to remove temporary directory '{:?}'",
                temp_dir_path
            );
        }
    } else {
        log!("Debug:Leaving temporary directory '{:?}'", temp_dir_path);
    }

    Ok(())
}

/// 返回 (Inner 临时目录,package 结构体)
pub fn unpack_nep(source_file: String, verify_signature: bool) -> Result<(PathBuf, GlobalPackage)> {
    // 创建临时目录
    let (temp_dir_path, file_stem) = get_temp_dir_path(source_file.clone(), true)?;
    let temp_dir_outer_path = temp_dir_path.join("Outer");
    let temp_dir_inner_path = temp_dir_path.join("Inner");

    // 解压外包
    log!("Info:Unpacking outer package...");
    let temp_dir_outer_str = temp_dir_outer_path.to_string_lossy().to_string();
    release_tar(source_file, temp_dir_outer_str.clone())
        .map_err(|e| anyhow!("Error:Invalid nep package : {}", e.to_string()))?;
    let inner_pkg_str = outer_validator(temp_dir_outer_str.clone(), file_stem.clone())?;
    let signature_path = temp_dir_outer_path.join("signature.toml");
    log_ok_last!("Info:Unpacking outer package...");

    // 签名文件加载与校验
    let signature_struct = parse_signature(signature_path.to_string_lossy().to_string())?.package;
    if verify_signature {
        log!("Info:Verifying package signature...");
        if signature_struct.signature.is_some() {
            let check_res = verify(
                inner_pkg_str.clone(),
                signature_struct.signer.clone(),
                signature_struct.signature.unwrap(),
            )?;
            if !check_res {
                return Err(anyhow!(
                    "Error:Failed to verify package signature, this package may have been hacked"
                ));
            }
            log_ok_last!("Info:Verifying package signature...");
        } else {
            return Err(anyhow!(
                "Error:This package doesn't contain signature, use offline mode to install"
            ));
        }
    } else {
        log!("Warning:Signature verification has been disabled!");
    }

    // 解压内包
    log!("Info:Decompressing inner package...");
    let temp_dir_inner_str = temp_dir_inner_path.to_string_lossy().to_string();
    decompress(inner_pkg_str.clone(), temp_dir_inner_str.clone())
        .map_err(|e| anyhow!("Error:Invalid nep package : {}", e.to_string()))?;
    inner_validator(temp_dir_inner_str.clone())?;
    log_ok_last!("Info:Decompressing inner package...");

    // 读取 package.toml
    let package_struct = parse_package(
        temp_dir_inner_path
            .join("package.toml")
            .to_string_lossy()
            .to_string(),
        None,
    )?;

    // 检查签名者与第一作者是否一致
    let author = parse_author(package_struct.package.authors[0].clone())?;
    if signature_struct.signer != author.email.unwrap() {
        if verify_signature {
            return Err(anyhow!(
                "Error:Invalid package : expect first author '{}' to be the package signer '{}'",
                &package_struct.package.authors[0],
                &signature_struct.signer
            ));
        } else {
            log!("Warning:Invalid package : expect first author '{}' to be the package signer '{}', ignoring this error due to signature verification has been disabled",&package_struct.package.authors[0],&signature_struct.signer);
        }
    }

    Ok((temp_dir_inner_path, package_struct))
}
