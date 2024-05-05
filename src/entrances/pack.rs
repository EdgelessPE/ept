use crate::compression::{compress, pack_tar};
use crate::entrances::verify::verify;
use crate::parsers::parse_author;
use crate::signature::sign;
use crate::types::{signature::Signature, signature::SignatureNode};
use crate::utils::{allocate_path_temp, is_debug_mode, term::ask_yn};
use crate::{log, log_ok_last, p2s};
use anyhow::{anyhow, Result};
use std::fs::{remove_dir_all, write};
use std::path::Path;

pub fn pack(source_dir: &String, into_file: Option<String>, need_sign: bool) -> Result<String> {
    log!("Info:Preparing to pack '{source_dir}'");

    // 通用校验
    let global = verify(source_dir)?;
    let first_author = parse_author(&global.package.authors[0])?;
    let file_stem = format!(
        "{pn}_{pv}_{fa}",
        pn = global.package.name,
        pv = global.package.version,
        fa = first_author.name
    );

    // 校验 into_file 是否存在
    let into_file = into_file.unwrap_or(String::from("./") + &file_stem + ".nep");
    let into_file_path = Path::new(&into_file);
    if into_file_path.exists() {
        if into_file_path.is_dir() {
            return Err(anyhow!(
                "Error:Target '{into_file}' is a existing directory"
            ));
        } else {
            log!("Warning:Overwrite the existing file '{into_file}'? (y/n)");
            if !ask_yn() {
                return Err(anyhow!("Error:Pack canceled by user"));
            }
        }
    }

    // 创建临时目录
    let temp_dir_path = allocate_path_temp(&file_stem, false)?;

    // 生成内包
    log!("Info:Compressing inner package...");
    let inner_path_str = p2s!(temp_dir_path.join(&(file_stem.clone() + ".tar.zst")));
    compress(source_dir, &inner_path_str)?;
    log_ok_last!("Info:Compressing inner package...");

    // 对内包进行签名
    let signature = if need_sign {
        log!("Info:Signing inner package...");
        let signature = sign(&inner_path_str)?;
        Some(signature)
    } else {
        None
    };
    let sign_file_path = temp_dir_path.join("signature.toml");
    let signature_struct = Signature {
        package: SignatureNode {
            raw_name_stem: file_stem,
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
    pack_tar(&p2s!(temp_dir_path), &into_file)?;
    log_ok_last!("Info:Packing outer package...");

    // 清理临时文件夹
    if !is_debug_mode() {
        log!("Info:Cleaning...");
        let clean_res = remove_dir_all(&temp_dir_path);
        if clean_res.is_ok() {
            log_ok_last!("Info:Cleaning...");
        } else {
            log!(
                "Warning:Failed to remove temporary directory '{dir}'",
                dir = p2s!(temp_dir_path)
            );
        }
    } else {
        log!(
            "Debug:Leaving temporary directory '{dir}'",
            dir = p2s!(temp_dir_path)
        );
    }

    Ok(into_file)
}

#[test]
fn test_pack() {
    envmnt::set("DEBUG", "true");
    envmnt::set("CONFIRM", "true");
    pack(
        &"./examples/ComplexFS".to_string(),
        Some("./test/ComplexFS_1.75.0.0_Cno.nep".to_string()),
        true,
    )
    .unwrap();
    pack(
        &"./examples/ComplexFS".to_string(),
        Some("./test/ComplexFS_1.75.0.0_Cno.nep".to_string()),
        false,
    )
    .unwrap();
}
