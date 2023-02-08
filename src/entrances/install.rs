use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs::{create_dir_all, remove_dir_all, rename};
use std::path::Path;

use super::{
    info_local,
    validator::{inner_validator, installed_validator, outer_validator},
};
use crate::utils::is_debug_mode;
use crate::{
    compression::{decompress, release_tar},
    executor::workflow_executor,
    parsers::{parse_package, parse_signature, parse_workflow},
    signature::verify,
    utils::{log, log_ok_last},
};

pub fn install_using_package(source_file: String, verify_signature: bool) -> Result<()> {
    log(format!("Info:Preparing to install '{}'", &source_file));

    // 创建临时目录
    let file_stem = Path::new(&source_file)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let temp_dir_path = Path::new("./temp").join(&file_stem);
    let temp_dir_outer_path = temp_dir_path.join("Outer");
    let temp_dir_inner_path = temp_dir_path.join("Inner");
    if temp_dir_path.exists() {
        remove_dir_all(&temp_dir_path)?;
    }
    create_dir_all(&temp_dir_outer_path)?;
    create_dir_all(&temp_dir_inner_path)?;

    // 解压外包
    log(format!("Info:Unpacking outer package..."));
    let temp_dir_outer_str = temp_dir_outer_path.to_string_lossy().to_string();
    release_tar(source_file, temp_dir_outer_str.clone())
        .map_err(|e| anyhow!("Error:Invalid nep package : {}", e.to_string()))?;
    let inner_pkg_str = outer_validator(temp_dir_outer_str.clone(), file_stem.clone())?;
    let signature_path = temp_dir_outer_path.join("signature.toml");
    log_ok_last(format!("Info:Unpacking outer package..."));

    // 签名文件加载与校验
    let signature_struct = parse_signature(signature_path.to_string_lossy().to_string())?.package;
    if verify_signature {
        log(format!("Info:Verifying package signature..."));
        if signature_struct.signature.is_some() {
            verify(
                inner_pkg_str.clone(),
                signature_struct.signer.clone(),
                signature_struct.signature.unwrap(),
            )?;
            log_ok_last(format!("Info:Verifying package signature..."));
        } else {
            return Err(anyhow!(
                "Error:This package doesn't contain signature, use offline mode to install"
            ));
        }
    } else {
        log("Warning:Signature verification has been disabled!".to_string());
    }

    // 解压内包
    log(format!("Info:Decompressing inner package..."));
    let temp_dir_inner_str = temp_dir_inner_path.to_string_lossy().to_string();
    decompress(inner_pkg_str.clone(), temp_dir_inner_str.clone())
        .map_err(|e| anyhow!("Error:Invalid nep package : {}", e.to_string()))?;
    inner_validator(temp_dir_inner_str.clone())?;
    log_ok_last(format!("Info:Decompressing inner package..."));

    // 读入包信息和安装工作流
    log(format!("Info:Resolving package..."));
    let pkg_file_path = temp_dir_inner_path.join("package.toml");
    let package_struct = parse_package(pkg_file_path.to_string_lossy().to_string())?;
    let setup_file_path = temp_dir_inner_path.join("workflows/setup.toml");
    let setup_workflow = parse_workflow(setup_file_path.to_string_lossy().to_string())?;

    // 检查签名者与第一作者是否一致
    if signature_struct.signer != package_struct.package.authors[0] {
        if verify_signature {
            return Err(anyhow!(
                "Error:Invalid package : expect first author '{}' to be the package signer '{}'",
                &package_struct.package.authors[0],
                &signature_struct.signer
            ));
        } else {
            log(format!("Warning:Invalid package : expect first author '{}' to be the package signer '{}', ignoring this error due to signature verification has been disabled",&package_struct.package.authors[0],&signature_struct.signer));
        }
    } else {
        log_ok_last(format!("Info:Resolving package..."));
    }

    // 创建 apps 文件夹
    log(format!("Info:Deploying files..."));
    if !Path::new("./apps").exists() {
        create_dir_all("./apps")?;
    }

    // 检查对应包名有没有被安装过
    let try_get_info_res = info_local(package_struct.package.name.clone());
    if try_get_info_res.is_ok() {
        // TODO:支持升级后此处进行升级
        let (_, diff) = try_get_info_res.unwrap();
        return Err(anyhow!(
            "Error:Package '{}' has been installed({}), current ept doesn't support upgrade",
            &package_struct.package.name,
            diff.version
        ));
    }

    // 解析最终安装位置
    let into_dir = Path::new("./apps")
        .join(&package_struct.package.name)
        .to_string_lossy()
        .to_string();

    // 移动程序至 apps 目录
    let app_path = temp_dir_inner_path.join(&package_struct.package.name);
    if !app_path.exists() {
        return Err(anyhow!(
            "Error:App folder not found : {}",
            app_path.to_string_lossy()
        ));
    }
    rename(app_path, into_dir.clone())?;
    log_ok_last(format!("Info:Deploying files..."));

    // 执行安装工作流
    log(format!("Info:Running setup workflow..."));
    workflow_executor(setup_workflow, into_dir.clone())?;
    log(format!("Info:Running setup workflow...   {}", "ok".green()));

    // 保存上下文
    let ctx_path = Path::new(&into_dir).join(".nep_context");
    rename(temp_dir_inner_path, ctx_path)?;

    // 检查安装是否完整
    log(format!("Info:Validating setup..."));
    installed_validator(into_dir)?;
    log_ok_last(format!("Info:Validating setup..."));

    // 清理临时文件夹
    if !is_debug_mode() {
        log(format!("Info:Cleaning..."));
        let clean_res = remove_dir_all(&temp_dir_path);
        if clean_res.is_ok() {
            log_ok_last(format!("Info:Cleaning..."));
        } else {
            log(format!(
                "Warning:Failed to remove temporary directory '{}'",
                temp_dir_path.to_string_lossy().to_string()
            ));
        }
    } else {
        log(format!(
            "Debug:Leaving temporary directory '{}'",
            temp_dir_path.to_string_lossy().to_string()
        ));
    }

    Ok(())
}

#[test]
fn test_install() {
    install_using_package(
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode_1.75.0.0_Cno.nep".to_string(),
        false,
    )
    .unwrap();
}
