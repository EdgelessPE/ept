use anyhow::Result;

use crate::{
    log,
    types::info::Info,
    utils::{fs::read_sub_dir, get_bare_apps},
};

use super::info::info;

pub fn list() -> Result<Vec<Info>> {
    let app_dir = get_bare_apps()?;
    let mut res = vec![];
    // 扫描本地 apps 目录
    for scope in read_sub_dir(app_dir.clone())? {
        // 扫描 scope 目录
        for name in read_sub_dir(app_dir.join(&scope))? {
            // 尝试将其作为合法的 nep 安装目录读取 info
            let info_res = info(Some(scope.clone()), &name);
            if let Ok(r) = info_res {
                res.push(r);
            } else {
                log!(
                    "Warning:Skip invalid folder '{scope}/{name}' : {e}",
                    e = info_res.unwrap_err()
                )
            }
        }
    }

    Ok(res)
}

#[test]
fn test_list() {
    let res = list().unwrap();
    println!("{res:#?}");
}
