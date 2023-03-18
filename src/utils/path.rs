use anyhow::{anyhow, Result};
use path_clean::PathClean;
use std::{
    env::current_dir,
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::p2s;

use super::get_bare_apps;

pub fn parse_relative_path(relative: String) -> Result<PathBuf> {
    let cr = relative.replace("./", "");
    let path = Path::new(&cr);

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir()?.join(path)
    }
    .clean();

    log!(
        "Debug:Parse relative path '{}' into '{}'",
        &relative,
        &absolute_path.display()
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
                "Error:Can't read '{}' as directory : {}",
                p2s!(path.as_ref().as_os_str()),
                e.to_string()
            )
        })?
        .into_iter()
        .filter_map(|entry_res| {
            if let Ok(entry) = entry_res {
                if !entry.path().is_dir() {
                    log!(
                        "Debug:Ignoring {} due to not a directory",
                        p2s!(entry.file_name())
                    );
                    return None;
                }
                Some(p2s!(entry.file_name()))
            } else {
                log!("Debug:Failed to get entry : {}", entry_res.unwrap_err());
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
    Err(anyhow!("Error:Can't find scope for '{}'", name))
}

#[test]
fn test_parse_relative_path() {
    let p1 = String::from("./VSCode/VSCode.exe");
    let p2 = String::from(r"D:\Desktop\Projects\") + "./code.exe";
    let p3 = p2s!(current_dir().unwrap().join("./code.exe"));

    println!("{:?}", parse_relative_path(p1));
    println!("{:?}", parse_relative_path(p2));
    println!("{:?}", parse_relative_path(p3));
}
