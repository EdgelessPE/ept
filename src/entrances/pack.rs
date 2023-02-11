use crate::compression::{compress, pack_tar};
use crate::parsers::parse_package;
use crate::signature::sign;
use crate::types::{Signature, SignatureNode};
use crate::utils::{is_debug_mode, log, log_ok_last, get_path_temp};
use anyhow::Result;
use std::fs::{create_dir_all, remove_dir_all, write};
use std::path::Path;

use super::utils::inner_validator;

pub fn pack(
    source_dir: String,
    into_file: Option<String>,
    package_signer: String,
    need_sign: bool,
) -> Result<String> {
    log(format!("Info:Preparing to pack '{}'", &source_dir));

    // 打包检查
    log(format!("Info:Validating source directory..."));
    inner_validator(source_dir.clone())?;
    log_ok_last(format!("Info:Validating source directory..."));

    // 读取包信息
    log(format!("Info:Resolving data..."));
    let pkg_path = Path::new(&source_dir).join("package.toml");
    let global = parse_package(pkg_path.to_string_lossy().to_string(), None)?;
    let file_stem = format!(
        "{}_{}_{}",
        &global.package.name, &global.package.version, &global.package.authors[0]
    );
    let into_file = into_file.unwrap_or(String::from("./") + &file_stem + ".nep");
    log_ok_last(format!("Info:Resolving data..."));

    // 创建临时目录
    let temp_dir_path = get_path_temp().join(&file_stem);
    if temp_dir_path.exists() {
        remove_dir_all(&temp_dir_path)?;
    }
    create_dir_all(&temp_dir_path)?;

    // 生成内包
    log(format!("Info:Compressing inner package..."));
    let inner_path_str = temp_dir_path
        .join(&(file_stem.clone() + ".tar.zst"))
        .to_string_lossy()
        .to_string();
    compress(source_dir, inner_path_str.clone())?;
    log_ok_last(format!("Info:Compressing inner package..."));

    // 对内包进行签名
    let signature = if need_sign {
        log(format!("Info:Signing inner package..."));
        let signature = sign(inner_path_str.clone())?;
        Some(signature)
    } else {
        None
    };
    let sign_file_path = temp_dir_path.join("signature.toml");
    let signature_struct = Signature {
        package: SignatureNode {
            signer: package_signer,
            signature,
        },
    };
    let text = toml::to_string_pretty(&signature_struct)?;
    write(sign_file_path, &text)?;
    if need_sign {
        log_ok_last(format!("Info:Signing inner package..."));
    } else {
        log("Warning:Signing has been disabled!".to_string())
    }

    // 生成外包
    log(format!("Info:Packing outer package..."));
    pack_tar(
        temp_dir_path.to_string_lossy().to_string(),
        into_file.clone(),
    )?;
    log_ok_last(format!("Info:Packing outer package..."));

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

    Ok(into_file)
}

#[test]
fn test_pack() {
    pack(
        r"D:\Download\VSCode-win32-x64-1.75.0".to_string(),
        Some("./examples/VSCode_1.75.0.0_Cno.nep".to_string()),
        "test@edgeless.top".to_string(),
        true,
    )
    .unwrap();
}
