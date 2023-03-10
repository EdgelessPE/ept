use anyhow::{anyhow, Result};
use std::{collections::HashMap, path::Path};

use crate::p2s;

pub fn inner_validator(dir: String) -> Result<()> {
    let manifest = vec!["package.toml", "workflows/setup.toml"];
    for file_name in manifest {
        let p = Path::new(&dir).join(file_name);
        if !p.exists() {
            return Err(anyhow!(
                "Error:Invalid nep inner package : missing '{}' in '{}'",
                &file_name,
                &dir
            ));
        }
    }
    Ok(())
}

pub fn manifest_validator(dir: String, manifest: Vec<String>) -> Result<()> {
    for file_name in manifest {
        let p = Path::new(&dir).join(&file_name);
        if !p.exists() {
            return Err(anyhow!(
                "Error:Invalid nep inner package : missing flow item '{}' in '{}'",
                &file_name,
                &dir
            ));
        }
    }
    Ok(())
}

// 定义外包的装箱单（内包名称需要传入）
macro_rules! def_outer_manifest {
    ($inner_pkg_name:expr) => {{
        vec!["signature.toml", &$inner_pkg_name]
    }};
}

// 返回内包路径
pub fn outer_validator(dir: String, stem: String) -> Result<String> {
    let inner_pkg_name = stem + ".tar.zst";
    let manifest = def_outer_manifest!(inner_pkg_name);
    for file_name in manifest {
        let p = Path::new(&dir).join(file_name);
        if !p.exists() {
            return Err(anyhow!(
                "Error:Invalid nep outer package : missing '{}' in '{}'",
                &file_name,
                &dir
            ));
        }
    }

    let inner_path = Path::new(&dir).join(&inner_pkg_name);
    Ok(p2s!(inner_path))
}

pub fn outer_hashmap_validator(map: &HashMap<String, Vec<u8>>, stem: String) -> Result<()> {
    let inner_pkg_name = stem + ".tar.zst";
    let manifest = def_outer_manifest!(inner_pkg_name);
    for file_name in manifest {
        let entry = map.get(file_name);
        if entry.is_none() {
            return Err(anyhow!(
                "Error:Invalid nep outer package : missing '{}'",
                &file_name,
            ));
        }
    }

    Ok(())
}

// 返回上下文目录路径
pub fn installed_validator(dir: String) -> Result<String> {
    let ctx_path = Path::new(&dir).join(".nep_context");
    if !ctx_path.exists() || ctx_path.is_file() {
        return Err(anyhow!(
            "Error:Invalid nep app folder : missing '.nep_context' folder in '{}'",
            &dir
        ));
    }

    let ctx_str = p2s!(ctx_path);
    inner_validator(ctx_str.clone())?;
    Ok(ctx_str)
}
