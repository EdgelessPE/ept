use crate::compression::{compress, pack_tar};
use crate::parsers::parse_package;
use crate::signature::{sign};
use crate::types::Signature;
use crate::utils::{log, log_ok_last};
use anyhow::{Result};
use std::fs::{create_dir_all, remove_dir_all, write};
use std::path::Path;

use super::validator::inner_validator;

pub fn pack(
    source_dir: String,
    into_file:Option<String>,
    packager: String,
    need_sign: bool,
) -> Result<String> {

    log(format!("Info:Preparing to pack '{}'",&source_dir));

    // 打包检查
    log(format!("Info:Validating source directory..."));
    inner_validator(source_dir.clone())?;
    log_ok_last(format!("Info:Validating source directory..."));

    // 读取包信息
    log(format!("Info:Resolving data..."));
    let pkg_path=Path::new(&source_dir).join("package.toml");
    let global=parse_package(pkg_path.to_string_lossy().to_string())?;
    let file_stem=format!("{}_{}_{}",&global.package.name,&global.package.version,&packager);
    let into_file=into_file.unwrap_or(String::from("./")+&file_stem+".nep");
    log_ok_last(format!("Info:Resolving data..."));

    // 创建临时目录
    let temp_dir_path = Path::new("./temp").join(&file_stem);
    if temp_dir_path.exists() {
        remove_dir_all(&temp_dir_path)?;
    }
    create_dir_all(&temp_dir_path)?;

    // 生成内包
    log(format!("Info:Generating inner package..."));
    let inner_path_str = temp_dir_path
        .join(&(file_stem.clone() + ".tar.zst"))
        .to_string_lossy()
        .to_string();
    compress(source_dir, inner_path_str.clone())?;
    log_ok_last(format!("Info:Generating inner package..."));

    // 对内包进行签名
    log(format!("Info:Signing inner package..."));
    let signature = if need_sign {
        let signature = sign(inner_path_str.clone())?;
        Some(signature)
    } else {
        None
    };
    let sign_file_path = temp_dir_path.join("signature.toml");
    let signature_struct = Signature {
        packager,
        signature,
    };
    let text = toml::to_string_pretty(&signature_struct)?;
    write(sign_file_path, &text)?;
    log_ok_last(format!("Info:Signing inner package..."));

    // 生成外包
    log(format!("Info:Generating outer package..."));
    pack_tar(temp_dir_path.to_string_lossy().to_string(), into_file.clone())?;
    log_ok_last(format!("Info:Generating outer package..."));

    Ok(into_file)
}

#[test]
fn test_pack() {
    pack(
        r"D:\Download\VSCode-win32-x64-1.75.0".to_string(),
        Some("./examples/VSCode_1.75.0.0_Cno.nep".to_string()),
        "test@edgeless.top".to_string(),
        true,
    ).unwrap();
}
