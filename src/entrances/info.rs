use std::path::Path;

use anyhow::{anyhow, Result};

use crate::{
    p2s,
    parsers::parse_package,
    signature::blake3::compute_hash_blake3_from_string,
    types::{
        info::{Info, InfoDiff},
        matcher::PackageInputEnum,
        mirror::TreeItem,
        package::GlobalPackage,
    },
    utils::{
        cache::spawn_cache,
        download::download_nep,
        fs::read_sub_dir,
        get_path_apps, get_path_cache, get_path_mirror,
        mirror::{filter_release, read_quick_maps},
        path::find_scope_with_name,
    },
};

use super::{
    meta,
    utils::{package::unpack_nep, validator::installed_validator},
};

pub fn info_local(scope: &String, package_name: &String) -> Result<(GlobalPackage, InfoDiff)> {
    let local_path = get_path_apps(scope, package_name, false)?;
    if !local_path.exists() {
        return Err(anyhow!(
            "Error:Can't find package '{scope}/{package_name}' locally"
        ));
    }
    let local_str = p2s!(local_path);
    // 检查是否为标准的已安装目录
    let ctx_str = installed_validator(&local_str)?;
    let ctx_path = Path::new(&ctx_str);
    // 读入包信息
    let pkg_path = ctx_path.join("package.toml");
    let global = parse_package(&p2s!(pkg_path), &local_str, true)?;
    // 写本地信息
    let authors = global.package.authors.clone();
    let local = InfoDiff {
        version: global.package.version.clone(),
        authors,
    };
    Ok((global.clone(), local))
}

// 第二个参数为 URL 模板
pub fn info_online(
    scope: &String,
    package_name: &String,
    mirror: Option<String>,
) -> Result<(TreeItem, String)> {
    // 定义匹配函数
    let item_matcher = |mirror_name: &String| {
        let quick_maps = read_quick_maps(mirror_name)?;
        let res = quick_maps
            .full_map
            .get(&(scope.to_lowercase(), package_name.to_lowercase()));
        if let Some(item) = res {
            Ok((item.clone(), quick_maps.url_template))
        } else {
            Err(anyhow!("Error:Failed to find '{scope}/{package_name}'"))
        }
    };
    if let Some(mirror_name) = mirror {
        return item_matcher(&mirror_name);
    } else {
        // 遍历 mirror 目录，读出软件包树并进行查找
        let p = get_path_mirror()?;
        let mirror_names = read_sub_dir(p)?;
        for name in mirror_names {
            if let Ok(res) = item_matcher(&name) {
                return Ok(res);
            }
        }
    }

    Err(anyhow!(
        "Error:Package '{package_name}' in scope '{scope}' not found"
    ))
}

pub fn info(current_input: PackageInputEnum, next_input: Option<PackageInputEnum>) -> Result<Info> {
    let mut mirror = None;
    let (scope, package_name, local, meta_res) = match current_input {
        PackageInputEnum::PackageMatcher(matcher) => {
            let scope = matcher.scope.clone();
            let package_name = matcher.name.clone();
            mirror = matcher.mirror.clone();
            // 查找 scope 并使用 scope 更新纠正大小写
            let (scope, package_name) = find_scope_with_name(&package_name, scope)?;
            // 扫描本地安装目录
            let local_path = get_path_apps(&scope, &package_name, false)?;
            if local_path.exists() {
                let (_global, local) = info_local(&scope, &package_name)?;
                (
                    scope,
                    package_name,
                    Some(local),
                    Some(meta(
                        PackageInputEnum::PackageMatcher(matcher.clone()),
                        false,
                    )?),
                )
            } else {
                (scope, package_name, None, None)
            }
        }
        PackageInputEnum::LocalPath(_) => {
            let meta_res = meta(current_input, false)?;
            let package = meta_res.package.package.clone();
            (
                package.scope,
                package.name,
                Some(InfoDiff {
                    version: package.version,
                    authors: package.authors,
                }),
                Some(meta_res),
            )
        }
        PackageInputEnum::Url(url) => {
            // 下载文件到临时目录
            let cache_path = get_path_cache()?;
            let url_hash = compute_hash_blake3_from_string(&url)?;
            let (p, cache_ctx) = download_nep(&url, Some((cache_path, url_hash)))?;
            let p_str = p2s!(p);
            // 缓存下载的包
            spawn_cache(cache_ctx)?;

            let (p, pkg) = unpack_nep(&p_str, false)?;
            let p_str = p2s!(p);
            let package = pkg.package;
            let meta_res = meta(PackageInputEnum::LocalPath(p_str), false)?;

            (
                package.scope,
                package.name,
                Some(InfoDiff {
                    version: package.version,
                    authors: package.authors,
                }),
                Some(meta_res),
            )
        }
    };

    // 创建结果结构体
    let mut info = Info {
        scope: scope.clone(),
        name: package_name.clone(),
        local,
        online: None,
        // package: None,
        // software: None,
        meta: meta_res,
    };

    // 在线检查
    if let Some(next_package_input) = next_input {
        let next_meta = meta(next_package_input, false)?;
        info.online = Some(InfoDiff {
            version: next_meta.package.package.version.clone(),
            authors: next_meta.package.package.authors.clone(),
        });
        info.meta = Some(next_meta);
    } else if let Ok((item, _)) = info_online(&scope, &package_name, mirror) {
        let latest = filter_release(item.releases, None, false)?;
        let mut authors = Vec::new();
        if let Some(meta) = latest.meta {
            authors = meta.package.package.authors.clone();
            info.meta = Some(meta);
        }
        info.online = Some(InfoDiff {
            version: latest.version.to_string(),
            authors,
        });
    }

    // 检查到底有没有这个包
    if info.local.is_some() || info.online.is_some() {
        Ok(info)
    } else {
        Err(anyhow!("Error:Unknown package '{package_name}'"))
    }
}

#[test]
fn test_info() {
    use crate::types::matcher::PackageMatcher;
    use crate::utils::flags::{set_flag, Flag};
    use crate::utils::test::_ensure_testing_vscode;
    set_flag(Flag::Confirm, true);
    // 替换测试镜像源
    let custom_mirror_ctx = crate::utils::test::_mount_custom_mirror();
    _ensure_testing_vscode();

    // 带 scope
    let base = info(
        PackageInputEnum::PackageMatcher(PackageMatcher {
            scope: Some("Microsoft".to_string()),
            name: "VSCode".to_string(),
            mirror: None,
            version_req: None,
        }),
        None,
    )
    .unwrap();
    println!("{base:#?}");

    // 单纯名字
    let res = info(
        PackageInputEnum::PackageMatcher(PackageMatcher {
            scope: None,
            name: "vscode".to_string(),
            mirror: None,
            version_req: None,
        }),
        None,
    )
    .unwrap();
    assert_eq!(base, res);

    // 别名
    let res = info(
        PackageInputEnum::PackageMatcher(PackageMatcher {
            scope: None,
            name: "CoDe".to_string(),
            mirror: None,
            version_req: None,
        }),
        None,
    )
    .unwrap();
    assert_eq!(base, res);

    // 换回原镜像源
    crate::utils::test::_unmount_custom_mirror(custom_mirror_ctx);
}
