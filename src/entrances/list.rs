use anyhow::Result;

use crate::{
    types::info::Info,
    utils::{get_bare_apps, read_sub_dir},
};

use super::info::info;

pub fn list() -> Result<Vec<Info>> {
    let app_dir = get_bare_apps()?;
    let mut res = vec![];
    // 扫描本地 apps 目录
    for scope in read_sub_dir(app_dir.clone())? {
        // 扫描 scope 目录
        for name in read_sub_dir(app_dir.join(&scope))? {
            res.push(info(Some(scope.clone()), &name)?);
        }
    }

    Ok(res)
}

#[test]
fn test_list() {
    let res = list().unwrap();
    println!("{:#?}", res);
}
