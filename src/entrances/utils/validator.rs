use anyhow::{anyhow, Result};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    executor::values_validator_path,
    log, p2s,
    types::mixed_fs::MixedFS,
    utils::{ask_yn, contains_wild_match},
};

pub fn inner_validator(dir: &String) -> Result<()> {
    let manifest = vec!["package.toml", "workflows/setup.toml"];
    for file_name in manifest {
        let p = Path::new(dir).join(file_name);
        if !p.exists() {
            return Err(anyhow!(
                "Error:Invalid nep inner package : missing '{file_name}' in '{dir}'"
            ));
        }
    }
    Ok(())
}

pub fn manifest_validator(
    base: &String,
    manifest: Vec<String>,
    fs: &mut MixedFS,
) -> Result<()> {
    let mut missing_list = HashSet::new();
    for path in manifest {
        values_validator_path(&path)?;
        if contains_wild_match(&path) {
            return Err(anyhow!(
                "Error:Wild match shouldn't appear in manifest item '{path}'"
            ));
        }
        if !fs.exists(&path) {
            missing_list.insert(path);
        }
    }
    if !missing_list.is_empty() {
        let items: Vec<String> = missing_list.into_iter().collect();
        if fs.var_warn_manifest {
            log!("Warning:May missing these flow items '{items:?}' in '{base}', continue? (y/n)");
            if !ask_yn() {
                return Err(anyhow!("Error:Operation canceled by user"));
            }
        } else {
            return Err(anyhow!(
                "Error:Invalid nep inner package : missing flow item '{items:?}' in '{base}'"
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
pub fn outer_validator(dir: &String, stem: &String) -> Result<String> {
    let inner_pkg_name = stem.to_owned() + ".tar.zst";
    let manifest = def_outer_manifest!(inner_pkg_name);
    for file_name in manifest {
        let p = Path::new(dir).join(file_name);
        if !p.exists() {
            return Err(anyhow!(
                "Error:Invalid nep outer package : missing '{file_name}' in '{dir}'"
            ));
        }
    }

    let inner_path = Path::new(dir).join(&inner_pkg_name);
    Ok(p2s!(inner_path))
}

pub fn outer_hashmap_validator(map: &HashMap<String, Vec<u8>>, stem: &String) -> Result<()> {
    let inner_pkg_name = stem.to_owned() + ".tar.zst";
    let manifest = def_outer_manifest!(inner_pkg_name);
    for file_name in manifest {
        let entry = map.get(file_name);
        if entry.is_none() {
            return Err(anyhow!(
                "Error:Invalid nep outer package : missing '{file_name}'"
            ));
        }
    }

    Ok(())
}

// 返回上下文目录路径
pub fn installed_validator(dir: &String) -> Result<String> {
    let ctx_path = Path::new(dir).join(".nep_context");
    if !ctx_path.exists() || ctx_path.is_file() {
        return Err(anyhow!(
            "Error:Invalid nep app folder : missing '.nep_context' folder in '{dir}'"
        ));
    }

    let ctx_str = p2s!(ctx_path);
    inner_validator(&ctx_str)?;
    Ok(ctx_str)
}
