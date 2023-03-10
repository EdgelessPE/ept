use anyhow::Result;
use std::fs::read_dir;

use crate::{types::Info, utils::get_path_apps};

use super::info::info;

pub fn list() -> Result<Vec<Info>> {
    // 扫描本地 apps 目录
    let mut res = vec![];
    for entry in read_dir(get_path_apps())? {
        let entry = entry?;
        let package_name = entry.file_name().to_string_lossy().to_string();
        let i = info(package_name)?;
        res.push(i);
    }

    Ok(res)
}

#[test]
fn test_list() {
    let res = list();
    println!("{:?}", res);
}
