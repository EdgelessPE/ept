use anyhow::{anyhow, Result};
use path_clean::PathClean;
use std::{
    fs::{canonicalize, read_dir},
    path::{Path, PathBuf},
};

use crate::p2s;

use super::{get_bare_apps, get_config};

pub fn parse_relative_path(relative: &String) -> Result<PathBuf> {
    let path = Path::new(&relative);

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let cfg = get_config();
        let relative = Path::new(&cfg.local.base).join(path);
        let dirty_abs = p2s!(canonicalize(relative)?);
        Path::new(&dirty_abs[4..]).to_path_buf()
    }
    .clean();

    log!(
        "Debug:Parse relative path '{relative}' into '{p}'",
        p = p2s!(absolute_path)
    );
    Ok(absolute_path)
}

pub fn read_sub_dir<P>(path: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let res = read_dir(path.as_ref())
        .map_err(|e| {
            anyhow!(
                "Error:Can't read '{p}' as directory : {e}",
                p = p2s!(path.as_ref().as_os_str())
            )
        })?
        .into_iter()
        .filter_map(|entry_res| {
            if let Ok(entry) = entry_res {
                if !entry.path().is_dir() {
                    log!(
                        "Debug:Ignoring {f} due to not a directory",
                        f = p2s!(entry.file_name())
                    );
                    return None;
                }
                Some(p2s!(entry.file_name()))
            } else {
                log!(
                    "Debug:Failed to get entry : {e}",
                    e = entry_res.unwrap_err()
                );
                None
            }
        })
        .collect();
    Ok(res)
}

/// name 大小写不敏感
pub fn find_scope_with_name_locally(name: &String) -> Result<String> {
    let app_dir = get_bare_apps()?;
    for scope in read_sub_dir(app_dir.clone())? {
        for dir_name in read_sub_dir(app_dir.join(&scope))? {
            if dir_name.to_ascii_lowercase() == name.to_ascii_lowercase() {
                return Ok(scope);
            }
        }
    }
    Err(anyhow!("Error:Can't find scope for '{name}'"))
}

#[test]
fn test_parse_relative_path() {
    let p1 = String::from("./VSCode/VSCode.exe");
    let p2 = String::from(r"D:\Desktop\Projects\") + "./code.exe";
    let p3 = p2s!(std::env::current_dir().unwrap().join("./code.exe"));

    println!("{:?}", parse_relative_path(&p1));
    println!("{:?}", parse_relative_path(&p2));
    println!("{:?}", parse_relative_path(&p3));
}
