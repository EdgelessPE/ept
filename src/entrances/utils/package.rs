use std::path::PathBuf;

use anyhow::{Result,anyhow};

/// 根据源文件路径创建并返回临时目录
fn get_temp_dir_path(source_file:String,keep_clear:bool)->Result<PathBuf>{
    let file_stem = Path::new(&source_file)
    .file_stem()?
    .to_string_lossy()
    .to_string();
    let temp_dir_path = get_path_temp().join(&file_stem);
    if !keep_clear{
        return Ok(temp_dir_path);
    }

    let temp_dir_outer_path = temp_dir_path.join("Outer");
    let temp_dir_inner_path = temp_dir_path.join("Inner");

    if temp_dir_path.exists() {
        remove_dir_all(&temp_dir_path)?;
    }
    create_dir_all(&temp_dir_outer_path)?;
    create_dir_all(&temp_dir_inner_path)?;

    Ok(temp_dir_path)
}

/// 清理临时目录（会判断 debug）
pub fn clean_temp(source_file:String)->Result<()>{
    let temp_dir_path=get_temp_dir_path(source_file, false)?;
    if !is_debug_mode() {
        log(format!("Info:Cleaning..."));
        let clean_res = remove_dir_all(&temp_dir_path);
        if clean_res.is_ok() {
            log_ok_last(format!("Info:Cleaning..."));
        } else {
            log(format!(
                "Warning:Failed to remove temporary directory '{:?}'",
                temp_dir_path
            ));
        }
    } else {
        log(format!(
            "Debug:Leaving temporary directory '{:?}'",
            temp_dir_path
        ));
    }

    Ok(())
}

/// 返回 Inner 临时目录
pub fn unpack_nep(source_file:String,verify_signature: bool)->Result<PathBuf>{
    // 创建临时目录
    let temp_dir_path = get_temp_dir_path(source_file, true)?;
    let temp_dir_outer_path = temp_dir_path.join("Outer");
    let temp_dir_inner_path = temp_dir_path.join("Inner");

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

    Ok(temp_dir_inner_path)
}