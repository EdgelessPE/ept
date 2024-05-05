use std::{
    cmp::min,
    collections::HashMap,
    fs::{remove_dir_all, File},
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use sysinfo::System;
use tar::Archive;

use crate::{
    compression::{decompress, fast_decompress_zstd, release_tar},
    entrances,
    entrances::utils::validator::{inner_validator, outer_hashmap_validator, outer_validator},
    p2s,
    parsers::{fast_parse_signature, parse_author, parse_package, parse_signature},
    signature::{fast_verify, verify},
    types::package::GlobalPackage,
    utils::{allocate_path_temp, fs::copy_dir, is_debug_mode},
};
use crate::{log, log_ok_last};

/// 根据源文件路径创建临时目录
fn get_temp_dir_path(source_file: &String) -> Result<PathBuf> {
    let file_stem = p2s!(Path::new(source_file).file_stem().unwrap());
    let temp_dir_path = allocate_path_temp(&file_stem, true)?;

    Ok(temp_dir_path)
}

/// 清理临时目录(会判断 debug)
pub fn clean_temp(source_file: &String) -> Result<()> {
    let temp_dir_path = get_temp_dir_path(source_file)?;
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
pub fn unpack_nep(source: &String, verify_signature: bool) -> Result<(PathBuf, GlobalPackage)> {
    // 处理输入目录的情况
    let source_path = Path::new(source);
    if source_path.is_dir() {
        if verify_signature {
            return Err(anyhow!("Error:Given path refers to a directory, use '--offline' flag to process as develop directory"));
        } else {
            // 检查是否为合法的输入目录
            inner_validator(source)?;
            entrances::verify::verify(source)?;

            // 读取 package.toml
            let package_path = Path::new(source).join("package.toml");
            let global = parse_package(&p2s!(package_path), source, false)?;

            // 复制到临时目录
            let temp_path = allocate_path_temp(&global.package.name, false)?;
            copy_dir(source_path, &temp_path)?;

            return Ok((temp_path, global));
        }
    }

    // 检查文件大小
    let file = File::open(source).map_err(|e| anyhow!("Error:Can't open file '{source}' : {e}"))?;
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
        fast_unpack_nep(source, verify_signature)?
    } else {
        log!("Debug:Use normal unpack method ({size}/{size_limit})");
        normal_unpack_nep(source, verify_signature)?
    };

    // 离线模式下强制执行一次检查
    // if !verify_signature {
    //     entrances::verify::verify(&p2s!(res.0))?;
    // }

    Ok(res)
}

fn normal_unpack_nep(
    source_file: &String,
    verify_signature: bool,
) -> Result<(PathBuf, GlobalPackage)> {
    // 创建临时目录
    let temp_dir_path = get_temp_dir_path(source_file)?;
    let temp_dir_outer_path = temp_dir_path.join("Outer");
    let temp_dir_inner_path = temp_dir_path.join("Inner");

    // 解压外包
    log!("Info:Unpacking outer package...");
    let temp_dir_outer_str = p2s!(temp_dir_outer_path);
    release_tar(source_file, &temp_dir_outer_str)
        .map_err(|e| anyhow!("Error:Invalid nep package : {e}"))?;
    let signature_path = temp_dir_outer_path.join("signature.toml");
    log_ok_last!("Info:Unpacking outer package...");

    // 签名文件加载与校验
    let signature_struct = parse_signature(&p2s!(signature_path))?.package;
    let inner_pkg_str = outer_validator(&temp_dir_outer_str, &signature_struct.raw_name_stem)?;
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
    let package_struct = parse_package(
        &p2s!(temp_dir_inner_path.join("package.toml")),
        &temp_dir_inner_str,
        false,
    )?;

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
    let temp_dir_path = get_temp_dir_path(source_file)?;
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
    log_ok_last!("Info:Reading outer package...");

    // 签名文件加载与校验
    let signature_raw = outer_map.get_mut("signature.toml").ok_or(anyhow!(
        "Error:Invalid nep outer package : missing 'signature.toml'"
    ))?;
    let signature_struct = fast_parse_signature(signature_raw)?.package;
    outer_hashmap_validator(&outer_map, &signature_struct.raw_name_stem)?;
    let inner_pkg_raw = outer_map
        .get(&(signature_struct.raw_name_stem + ".tar.zst"))
        .unwrap();
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
    let package_struct = parse_package(
        &p2s!(temp_dir_inner_path.join("package.toml")),
        &temp_dir_inner_str,
        false,
    )?;

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

#[test]
fn test_unpack_nep() {
    if cfg!(debug_assertions) {
        log!("Warning:Debug mode enabled");
        envmnt::set("DEBUG", "true");
    }
    crate::utils::test::_ensure_clear_test_dir();

    crate::pack(
        &"./examples/VSCode".to_string(),
        Some("./test/VSCode_1.75.0.0_Cno.nep".to_string()),
        true,
    )
    .unwrap();

    let res = unpack_nep(&"./test/VSCode_1.75.0.0_Cno.nep".to_string(), true).unwrap();
    println!("{res:#?}");
}

#[test]
fn test_fast_unpack_nep() {
    if cfg!(debug_assertions) {
        log!("Warning:Debug mode enabled");
        envmnt::set("DEBUG", "true");
    }
    crate::utils::test::_ensure_clear_test_dir();

    crate::pack(
        &"./examples/VSCode".to_string(),
        Some("./test/VSCode_1.75.0.0_Cno.nep".to_string()),
        true,
    )
    .unwrap();

    let res = fast_unpack_nep(&"./test/VSCode_1.75.0.0_Cno.nep".to_string(), true).unwrap();
    println!("{res:#?}");
}

#[test]
fn benchmark_fast_unpack_nep() {
    envmnt::set("DEBUG", "true");
    // 准备带有一定体积的包
    crate::utils::test::_ensure_clear_test_dir();

    crate::pack(
        &"examples/Dism++".to_string(),
        Some("./test/Dism++_10.1.1002.1_Cno.nep".to_string()),
        true,
    )
    .unwrap();

    use std::time::Instant;
    let normal = Instant::now();
    for _ in 0..10 {
        unpack_nep(&"./test/Dism++_10.1.1002.1_Cno.nep".to_string(), true).unwrap();
    }

    let fast = Instant::now();
    for _ in 0..10 {
        fast_unpack_nep(&"./test/Dism++_10.1.1002.1_Cno.nep".to_string(), true).unwrap();
    }
    println!(
        "Normal unpack cost {n}ms, fast unpack cost {f}ms",
        n = normal.elapsed().as_millis(),
        f = fast.elapsed().as_millis()
    );
}

#[test]
fn test_bad_package() {
    crate::utils::test::_ensure_clear_test_dir();

    // 生成基础目录
    crate::pack(
        &"./examples/Dism++".to_string(),
        Some("./test/Normal.nep".to_string()),
        true,
    )
    .unwrap();
    release_tar(
        &"./test/Normal.nep".to_string(),
        &"./test/Normal".to_string(),
    )
    .unwrap();

    // 未签名
    crate::pack(
        &"./examples/Dism++".to_string(),
        Some("./test/UnSig++_10.1.1002.1_Cno.nep".to_string()),
        false,
    )
    .unwrap();
    assert!(unpack_nep(&"./test/UnSig++_10.1.1002.1_Cno.nep".to_string(), true).is_err());

    // 被篡改的签名
    copy_dir("test/Normal", "test/BadSig").unwrap();
    let mut signature_struct = parse_signature(&"test/BadSig/signature.toml".to_string()).unwrap();
    signature_struct.package.signature = signature_struct
        .package
        .signature
        .map(|s| s.chars().rev().collect());
    let text = toml::to_string_pretty(&signature_struct).unwrap();
    std::fs::write("test/BadSig/signature.toml", text).unwrap();
    crate::compression::pack_tar(
        &"test/BadSig".to_string(),
        &"test/BadSig++_10.1.1002.1_Cno.nep".to_string(),
    )
    .unwrap();
    assert!(unpack_nep(&"test/BadSig++_10.1.1002.1_Cno.nep".to_string(), true).is_err());

    // 缺失签名文件
    copy_dir("test/Normal", "test/NoSig").unwrap();
    std::fs::remove_file("test/NoSig/signature.toml").unwrap();
    crate::compression::pack_tar(
        &"test/NoSig".to_string(),
        &"test/NoSig++_10.1.1002.1_Cno.nep".to_string(),
    )
    .unwrap();
    assert!(unpack_nep(&"test/NoSig++_10.1.1002.1_Cno.nep".to_string(), true).is_err());

    // 错误的打包者
    copy_dir("test/Normal", "test/BadAuth").unwrap();
    let mut signature_struct = parse_signature(&"test/BadAuth/signature.toml".to_string()).unwrap();
    signature_struct.package.signer = "Jack".to_string();
    let text = toml::to_string_pretty(&signature_struct).unwrap();
    std::fs::write("test/BadAuth/signature.toml", text).unwrap();
    crate::compression::pack_tar(
        &"test/BadAuth".to_string(),
        &"test/BadAuth++_10.1.1002.1_Cno.nep".to_string(),
    )
    .unwrap();
    assert!(unpack_nep(&"test/BadAuth++_10.1.1002.1_Cno.nep".to_string(), true).is_err());
}
