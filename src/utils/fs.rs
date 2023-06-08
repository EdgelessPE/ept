use crate::p2s;
use anyhow::{anyhow, Result};
use std::{
    fs::{create_dir_all, read_dir, remove_dir_all, remove_file},
    path::Path,
};

pub fn try_recycle<P: AsRef<Path>>(path: P) -> Result<()> {
    let p = path.as_ref();
    let str = p2s!(p);
    if p.is_dir() {
        if let Err(e) = trash::delete(&p) {
            log!("Warning:Failed to recycle '{str}' : {e}, try removing it");
            remove_dir_all(p).map_err(|e| anyhow!("Error:Failed to delete '{str}' : {e}"))
        } else {
            Ok(())
        }
    } else {
        if let Err(e) = trash::delete(&p) {
            log!("Warning:Failed to recycle '{str}' : {e}, try removing it");
            remove_file(p).map_err(|e| anyhow!("Error:Failed to delete '{str}' : {e}"))
        } else {
            Ok(())
        }
    }
}

pub fn read_sub_dir<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
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

pub fn count_sub_files<P, F>(path: P, filter: F) -> Result<i32>
where
    P: AsRef<Path>,
    F: Fn(String) -> bool,
{
    let count = read_dir(path.as_ref())
        .map_err(|e| {
            anyhow!(
                "Error:Can't read '{p}' as directory : {e}",
                p = p2s!(path.as_ref().as_os_str())
            )
        })?
        .into_iter()
        .filter_map(|entry_res| {
            if let Ok(entry) = entry_res {
                if entry.path().is_file() {
                    Some(p2s!(entry.file_name()))
                } else {
                    None
                }
            } else {
                log!(
                    "Debug:Failed to get entry : {e}",
                    e = entry_res.unwrap_err()
                );
                None
            }
        })
        .fold(0, |acc, x| if filter(x) { acc + 1 } else { acc });
    Ok(count)
}

pub fn ensure_dir_exist<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        create_dir_all(path).map_err(|e| {
            anyhow!(
                "Error:Failed to create dir '{p}' : {err}",
                p = p2s!(path),
                err = e.to_string()
            )
        })?;
    }

    Ok(())
}
