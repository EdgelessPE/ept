use anyhow::Result;

use crate::{
    log,
    types::{
        info::Info,
        matcher::{PackageInputEnum, PackageMatcher},
    },
    utils::{fs::read_sub_dir, get_bare_apps},
};

use super::info::info;

pub fn list(cfg: &crate::types::context::RuntimeContext) -> Result<Vec<Info>> {
    let app_dir = get_bare_apps(cfg)?;
    let mut res = vec![];
    // 扫描本地 apps 目录
    for scope in read_sub_dir(app_dir.clone())? {
        // 扫描 scope 目录
        for name in read_sub_dir(app_dir.join(&scope))? {
            // 尝试将其作为合法的 nep 安装目录读取 info
            let info_res = info(
                cfg,
                PackageInputEnum::PackageMatcher(PackageMatcher {
                    scope: Some(scope.clone()),
                    name: name.clone(),
                    mirror: None,
                    version_req: None,
                }),
                false,
            );
            if let Ok((r, _)) = info_res {
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
    use crate::utils::test::_default_test_cfg;
    let cfg = _default_test_cfg();
    let res = list(&cfg).unwrap();
    println!("{res:#?}");
}
