use crate::p2s;
use anyhow::{anyhow, Result};
use std::{
    fs::{read_dir, remove_dir_all, remove_file},
    path::{Path, PathBuf},
};

pub fn try_recycle(p: PathBuf) -> Result<()> {
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