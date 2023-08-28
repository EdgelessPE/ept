use crate::p2s;
use anyhow::{anyhow, Result};
use fs_extra::dir::CopyOptions;
use std::{
    fs::{create_dir_all, read_dir, remove_dir_all, remove_file, rename},
    path::Path,
};

pub fn try_recycle<P: AsRef<Path>>(path: P) -> Result<()> {
    let p = path.as_ref();
    let str = p2s!(p);
    if let Err(e) = trash::delete(&p) {
        log!("Warning:Failed to recycle '{str}' : {e}, try removing it");
        if p.is_dir() {
            remove_dir_all(p).map_err(|e| anyhow!("Error:Failed to delete directory '{str}' : {e}"))
        } else {
            remove_file(p).map_err(|e| anyhow!("Error:Failed to delete file '{str}' : {e}"))
        }
    } else {
        Ok(())
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
        create_dir_all(path)
            .map_err(|e| anyhow!("Error:Failed to create dir '{p}' : {e}", p = p2s!(path)))?;
    }

    Ok(())
}

pub fn move_or_copy<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    let from_str = p2s!(from);
    let to_str = p2s!(to);

    if let Err(e) = rename(from, to) {
        log!("Warning:Failed to move '{from_str}' to '{to_str}', trying to copy : {e}");

        // 尝试进行 Copy
        copy_dir(from, to)?;
    }

    Ok(())
}

pub fn copy_dir<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let from_str = p2s!(from.as_ref());
    let to_str = p2s!(to.as_ref());

    let opt = CopyOptions::new()
        .copy_inside(true)
        .overwrite(true)
        .content_only(true);
    fs_extra::dir::copy(from, to, &opt)
        .map_err(|e| anyhow!("Error:Failed to copy directory '{from_str}' to '{to_str}' : {e}"))?;

    Ok(())
}
