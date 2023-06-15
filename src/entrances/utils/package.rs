use std::{
    cmp::min,
    collections::HashMap,
    fs::{remove_dir_all, File},
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use sysinfo::{System, SystemExt};
use tar::Archive;

use crate::{
    compression::{decompress, fast_decompress_zstd, release_tar},
    entrances,
    entrances::utils::validator::{inner_validator, outer_hashmap_validator, outer_validator},
    p2s,
    parsers::{fast_parse_signature, parse_author, parse_package, parse_signature},
    signature::{fast_verify, verify},
    types::package::GlobalPackage,
    utils::{get_path_temp, is_debug_mode},
};
use crate::{log, log_ok_last};

/// 根据源文件路径创建并返回(临时目录,文件茎)
fn get_temp_dir_path(source_file: &String, keep_clear: bool) -> Result<(PathBuf, String)> {
    let file_stem = p2s!(Path::new(source_file).file_stem().unwrap());
    let temp_dir_path = get_path_temp(&file_stem, keep_clear, true)?;

    Ok((temp_dir_path, file_stem))
}

/// 清理临时目录(会判断 debug)
pub fn clean_temp(source_file: &String) -> Result<()> {
    let (temp_dir_path, _) = get_temp_dir_path(source_file, false)?;
    if !is_debug_mode() {
        log!("Info:Cleaning...");
        let clean_res = remove_dir_all(&temp_dir_path);
        if clean_res.is_ok() {
            log_ok_last!("Info:Cleaning...");
        } else {
            log!("Warning:Failed to remove temporary directory '{temp_dir_path:?}'");
        }
    } else {
        log!("Debug:Leaving temporary directory '{temp_dir_path:?}'");
    }

    Ok(())
}

/// 返回 (Inner 临时目录,package 结构体)
pub fn unpack_nep(
    source_file: &String,
    verify_signature: bool,
) -> Result<(PathBuf, GlobalPackage)> {
    // 处理输入目录的情况
    let source_path = Path::new(source_file);
    if source_path.is_dir() {
        if verify_signature {
            return Err(anyhow!("Error:Given path refers to a directory, use '--offline' flag to process as develop directory"));
        } else {
            // 检查是否为合法的输入目录
            inner_validator(source_file)?;
            // 校验
            entrances::verify::verify(source_file)?;
            // 读取 package.toml
            let package_path = Path::new(source_file).join("package.toml");
            let global = parse_package(&p2s!(package_path), None)?;

            return Ok((Path::new(source_file).to_path_buf(), global));
        }
    }

    // 检查文件大小
    let file = File::open(source_file)
        .map_err(|e| anyhow!("Error:Can't open file '{source_file}' : {e}"))?;
    let meta = file.metadata()?;
    let size = meta.len();
    // 获取 fast 处理方法的文件大小上限
    let s = System::new_all();
    let size_limit = envmnt::get_u64(
        "FAST_UNPACK_LIMIT",
        min(s.available_memory() / 10, 500 * 1024 * 1024),
    );

    let res = if size <= size_limit {
        log!("Debug:Use fast unpack method ({size}/{size_limit})");
        fast_unpack_nep(source_file, verify_signature)?
    } else {
        log!("Debug:Use normal unpack method ({size}/{size_limit})");
        normal_unpack_nep(source_file, verify_signature)?
    };

    // 离线模式下强制执行一次检查
    if !verify_signature {
        entrances::verify::verify(&p2s!(res.0))?;
    }

    Ok(res)
}

fn normal_unpack_nep(
    source_file: &String,
    verify_signature: bool,
) -> Result<(PathBuf, GlobalPackage)> {
    // 创建临时目录
    let (temp_dir_path, file_stem) = get_temp_dir_path(source_file, true)?;
    let temp_dir_outer_path = temp_dir_path.join("Outer");
    let temp_dir_inner_path = temp_dir_path.join("Inner");

    // 解压外包
    log!("Info:Unpacking outer package...");
    let temp_dir_outer_str = p2s!(temp_dir_outer_path);
    release_tar(source_file, &temp_dir_outer_str)
        .map_err(|e| anyhow!("Error:Invalid nep package : {e}"))?;
    let inner_pkg_str = outer_validator(&temp_dir_outer_str, &file_stem)?;
    let signature_path = temp_dir_outer_path.join("signature.toml");
    log_ok_last!("Info:Unpacking outer package...");

    // 签名文件加载与校验
    let signature_struct = parse_signature(&p2s!(signature_path))?.package;
    if verify_signature {
        log!("Info:Verifying package signature...");
        if signature_struct.signature.is_some() {
            let check_res = verify(
                &inner_pkg_str,
                &signature_struct.signer,
                &signature_struct.signature.unwrap(),
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
    let temp_dir_inner_str = p2s!(temp_dir_inner_path);
    decompress(&inner_pkg_str, &temp_dir_inner_str)
        .map_err(|e| anyhow!("Error:Invalid nep package : {e}"))?;
    inner_validator(&temp_dir_inner_str)?;
    log_ok_last!("Info:Decompressing inner package...");

    // 读取 package.toml
    let package_struct = parse_package(&p2s!(temp_dir_inner_path.join("package.toml")), None)?;

    // 检查签名者与第一作者是否一致
    let author = parse_author(&package_struct.package.authors[0])?;
    if signature_struct.signer != author.email.unwrap() {
        if verify_signature {
            return Err(anyhow!(
                "Error:Invalid package : expect first author '{fa}' to be the package signer '{ps}'",
                fa=package_struct.package.authors[0],
                ps=signature_struct.signer
            ));
        } else {
            log!("Warning:Invalid package : expect first author '{fa}' to be the package signer '{ps}', ignoring this error due to signature verification has been disabled",fa=package_struct.package.authors[0],ps=signature_struct.signer);
        }
    }

    Ok((temp_dir_inner_path, package_struct))
}
fn fast_unpack_nep(
    source_file: &String,
    verify_signature: bool,
) -> Result<(PathBuf, GlobalPackage)> {
    // 创建临时目录
    let (temp_dir_path, file_stem) = get_temp_dir_path(source_file, true)?;
    let temp_dir_inner_path = temp_dir_path.join("Inner");

    // 读取外包，生成 hashmap
    log!("Info:Reading outer package...");
    let outer_file =
        File::open(source_file).map_err(|e| anyhow!("Error:Can't open '{source_file}' : {e}"))?;
    let mut outer_tar = Archive::new(outer_file);
    let mut outer_map = HashMap::new();
    for entry in outer_tar.entries()? {
        let mut entry = entry?;
        let name = p2s!(entry.path()?);
        let mut buffer = Vec::with_capacity(entry.header().size()? as usize);
        entry.read_to_end(&mut buffer)?;
        outer_map.insert(name, buffer);
    }
    outer_hashmap_validator(&outer_map, &file_stem)?;
    log_ok_last!("Info:Reading outer package...");

    // 签名文件加载与校验
    let signature_raw = outer_map.get_mut("signature.toml").unwrap();
    let signature_struct = fast_parse_signature(signature_raw)?.package;
    let inner_pkg_raw = outer_map.get(&(file_stem + ".tar.zst")).unwrap();
    if verify_signature {
        log!("Info:Verifying package signature...");
        if signature_struct.signature.is_some() {
            let check_res = fast_verify(
                inner_pkg_raw,
                &signature_struct.signer,
                &signature_struct.signature.unwrap(),
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

    // 解压内包到临时目录
    log!("Info:Decompressing inner package...");
    let temp_dir_inner_str = p2s!(temp_dir_inner_path);
    let inner_tar_raw = fast_decompress_zstd(inner_pkg_raw)?;
    let inner_tar_file = Cursor::new(inner_tar_raw);
    let mut inner_archive = Archive::new(inner_tar_file);
    inner_archive.unpack(&temp_dir_inner_str)?;
    inner_validator(&temp_dir_inner_str)?;
    log_ok_last!("Info:Decompressing inner package...");

    // 读取 package.toml
    let package_struct = parse_package(&p2s!(temp_dir_inner_path.join("package.toml")), None)?;

    // 检查签名者与第一作者是否一致
    let author = parse_author(&package_struct.package.authors[0])?;
    if signature_struct.signer != author.email.unwrap() {
        if verify_signature {
            return Err(anyhow!(
                "Error:Invalid package : expect first author '{fa}' to be the package signer '{ps}'",
                fa=package_struct.package.authors[0],
                ps=signature_struct.signer
            ));
        } else {
            log!("Warning:Invalid package : expect first author '{fa}' to be the package signer '{ps}', ignoring this error due to signature verification has been disabled",fa=package_struct.package.authors[0],ps=signature_struct.signer);
        }
    }

    Ok((temp_dir_inner_path, package_struct))
}

#[test] // TO-FIX
fn test_unpack_nep() {
    if cfg!(debug_assertions) {
        log!("Warning:Debug mode enabled");
        envmnt::set("DEBUG", "true");
    }
    let res = unpack_nep(
        &r"D:\Desktop\Projects\EdgelessPE\ept\VSCode_1.75.0.0_Cno.nep".to_string(),
        true,
    )
    .unwrap();
    println!("{res:?}");
}

#[test] // TO-FIX
fn test_fast_unpack_nep() {
    let res = fast_unpack_nep(
        &r"D:\Desktop\Projects\EdgelessPE\ept\VSCode_1.75.0.0_Cno.nep".to_string(),
        true,
    )
    .unwrap();
    println!("{res:?}");
}

#[test] // TO-FIX
fn benchmark_fast_unpack_nep() {
    use std::time::Instant;
    let normal = Instant::now();
    for _ in 0..10 {
        unpack_nep(
            &r"D:\Desktop\Projects\EdgelessPE\ept\VSCode_1.75.0.0_Cno.nep".to_string(),
            true,
        )
        .unwrap();
    }
    println!(
        "Normal unpack cost {sec}s",
        sec = normal.elapsed().as_secs()
    ); // 42s

    let fast = Instant::now();
    for _ in 0..10 {
        fast_unpack_nep(
            &r"D:\Desktop\Projects\EdgelessPE\ept\VSCode_1.75.0.0_Cno.nep".to_string(),
            true,
        )
        .unwrap();
    }
    println!("Fast unpack cost {sec}s", sec = fast.elapsed().as_secs()); // 34s
}
