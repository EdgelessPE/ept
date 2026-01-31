use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use semver::VersionReq;

use crate::{
    log, p2s,
    parsers::parse_package,
    signature::blake3::compute_hash_blake3_from_string,
    types::{
        cfg::Cfg,
        constants::FILE_PACKAGE,
        info::{Info, InfoDiff},
        matcher::PackageInputEnum,
        meta::MetaResult,
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

fn consume_info_diff(
    cfg: &Cfg,
    item: &TreeItem,
    semver_matcher: Option<VersionReq>,
) -> Result<(InfoDiff, Option<MetaResult>)> {
    let latest = filter_release(cfg, item.releases.clone(), semver_matcher, true)?;
    let version = latest.version.to_string();
    Ok(if let Some(meta) = latest.meta {
        (
            InfoDiff {
                version,
                authors: meta.package.package.authors.clone(),
            },
            Some(meta),
        )
    } else {
        (
            InfoDiff {
                version,
                authors: vec![],
            },
            None,
        )
    })
}

pub fn info_local(cfg: &Cfg, scope: &str, package_name: &str) -> Result<(GlobalPackage, InfoDiff)> {
    log!("Debug:Reading local info for '{scope}/{package_name}'");
    let local_path = get_path_apps(cfg, scope, package_name, false)?;
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
    let pkg_path = ctx_path.join(FILE_PACKAGE);
    let global = parse_package(&p2s!(pkg_path), &local_str, true)?;
    // 写本地信息
    let authors = global.package.authors.clone();
    let local = InfoDiff {
        version: global.package.version.clone(),
        authors,
    };
    log!(
        "Debug:Found local package '{scope}/{package_name}' version '{ver}'",
        ver = &local.version
    );
    Ok((global.clone(), local))
}

// 第二个参数为 URL 模板，第三个参数为 mirror
pub fn info_online(
    cfg: &Cfg,
    scope: &str,
    package_name: &str,
    mirror: Option<String>,
) -> Result<(TreeItem, String, String)> {
    log!("Debug:Reading online info for '{scope}/{package_name}'");
    // 定义匹配函数
    let item_matcher = |mirror_name: &str| {
        let quick_maps = read_quick_maps(cfg, mirror_name)?;
        let res = quick_maps
            .full_map
            .get(&(scope.to_lowercase(), package_name.to_lowercase()));
        if let Some(item) = res {
            log!("Debug:Found '{scope}/{package_name}' in mirror '{mirror_name}'");
            Ok((
                item.clone(),
                quick_maps.url_template,
                mirror_name.to_string(),
            ))
        } else {
            Err(anyhow!("Error:Failed to find '{scope}/{package_name}'"))
        }
    };
    if let Some(mirror_name) = mirror {
        return item_matcher(&mirror_name);
    } else {
        // 遍历 mirror 目录，读出软件包树并进行查找
        let p = get_path_mirror(cfg)?;
        let mirror_names = read_sub_dir(p)?;
        for name in mirror_names {
            if let Ok((res, url_template, mirror_name)) = item_matcher(&name) {
                return Ok((res, url_template, mirror_name));
            }
        }
    }

    Err(anyhow!(
        "Error:Package '{package_name}' in scope '{scope}' not found"
    ))
}

// 不同类型输入的 info 结果类型别名
type InfoResult = (String, String, InfoDiff, Option<MetaResult>);

fn info_from_matcher(
    cfg: &Cfg,
    matcher: crate::types::matcher::PackageMatcher,
    _verify: bool,
) -> Result<InfoResult> {
    let mirror = matcher.mirror.clone();
    let (scope, package_name) = find_scope_with_name(cfg, &matcher.name, matcher.scope.as_deref())?;
    log!("Debug:Resolving matcher for '{scope}/{package_name}'");

    // 先尝试在线获取
    if let Ok((item, _, _)) = info_online(cfg, &scope, &package_name, mirror.clone()) {
        log!("Debug:Found online info for '{scope}/{package_name}'");
        let (info_diff, meta) = consume_info_diff(cfg, &item, matcher.version_req)?;
        return Ok((scope, package_name, info_diff, meta));
    }

    // 回退到本地获取
    log!("Debug:Trying local fallback for '{scope}/{package_name}'");
    let local_path = get_path_apps(cfg, &scope, &package_name, false)?;
    if local_path.exists() {
        let (_global, local) = info_local(cfg, &scope, &package_name)?;
        let meta_res = meta(cfg, PackageInputEnum::PackageMatcher(matcher), false)?;
        return Ok((scope, package_name, local, Some(meta_res)));
    }

    Err(anyhow!(
        "Error:Can't locate package '{scope}/{package_name}'"
    ))
}

fn info_from_local_path(cfg: &Cfg, path: String, verify: bool) -> Result<InfoResult> {
    let meta_res = meta(cfg, PackageInputEnum::LocalPath(path), verify)?;
    let package = &meta_res.package.package;
    Ok((
        package.scope.clone(),
        package.name.clone(),
        InfoDiff {
            version: package.version.clone(),
            authors: package.authors.clone(),
        },
        Some(meta_res),
    ))
}

fn info_from_url(cfg: &Cfg, url: String, verify: bool) -> Result<InfoResult> {
    log!("Debug:Fetching info from URL '{url}'");
    let cache_path = get_path_cache(cfg)?;
    let url_hash = compute_hash_blake3_from_string(&url)?;
    let (p, cache_ctx) = download_nep(cfg, &url, Some((cache_path, url_hash)))?;
    let p_str = p2s!(p);

    spawn_cache(cache_ctx)?;

    let (p, pkg) = unpack_nep(cfg, &p_str, verify)?;
    let p_str = p2s!(p);
    let meta_res = meta(cfg, PackageInputEnum::LocalPath(p_str), false)?;
    let package = pkg.package;
    log!(
        "Debug:Got info from URL for '{scope}/{name}' version '{ver}'",
        scope = &package.scope,
        name = &package.name,
        ver = &package.version
    );

    Ok((
        package.scope,
        package.name,
        InfoDiff {
            version: package.version,
            authors: package.authors,
        },
        Some(meta_res),
    ))
}

// 使用本地和在线数据丰富 info 信息
fn enrich_info(cfg: &Cfg, mut info: Info, mirror: Option<String>) -> Result<Info> {
    log!(
        "Debug:Enriching info for '{scope}/{name}'",
        scope = &info.scope,
        name = &info.name
    );
    if let Ok((_, local)) = info_local(cfg, &info.scope, &info.name) {
        info.local = Some(local);
    }
    if let Ok((item, _, _)) = info_online(cfg, &info.scope, &info.name, mirror) {
        let (info_diff, meta) = consume_info_diff(cfg, &item, None)?;
        info.online = Some(info_diff);
        if info.meta.is_none() {
            info.meta = meta;
        }
    }
    Ok(info)
}

pub fn info(
    cfg: &Cfg,
    target_input: PackageInputEnum,
    verify_signature: bool,
) -> Result<(Info, Option<PathBuf>)> {
    let (scope, package_name, target, meta_res, mirror) = match target_input {
        PackageInputEnum::PackageMatcher(matcher) => {
            let mirror = matcher.mirror.clone();
            let (scope, name, target, meta) = info_from_matcher(cfg, matcher, verify_signature)?;
            (scope, name, target, meta, mirror)
        }
        PackageInputEnum::LocalPath(path) => {
            let (scope, name, target, meta) = info_from_local_path(cfg, path, verify_signature)?;
            (scope, name, target, meta, None)
        }
        PackageInputEnum::Url(url) => {
            let (scope, name, target, meta) = info_from_url(cfg, url, verify_signature)?;
            (scope, name, target, meta, None)
        }
    };

    let temp_dir = meta_res.as_ref().and_then(|m| m.temp_dir.clone());

    let info = Info {
        scope,
        name: package_name.clone(),
        target,
        local: None,
        online: None,
        meta: meta_res,
    };

    let enriched = enrich_info(cfg, info, mirror)?;
    Ok((enriched, temp_dir))
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
        &crate::types::cfg::Cfg::default(),
        PackageInputEnum::PackageMatcher(PackageMatcher {
            scope: Some("Microsoft".to_string()),
            name: "VSCode".to_string(),
            mirror: None,
            version_req: None,
        }),
        false,
    )
    .unwrap();
    println!("{base:#?}");

    // 单纯名字
    let res = info(
        &crate::types::cfg::Cfg::default(),
        PackageInputEnum::PackageMatcher(PackageMatcher {
            scope: None,
            name: "vscode".to_string(),
            mirror: None,
            version_req: None,
        }),
        false,
    )
    .unwrap();
    assert_eq!(base, res);

    // 别名
    let res = info(
        &crate::types::cfg::Cfg::default(),
        PackageInputEnum::PackageMatcher(PackageMatcher {
            scope: None,
            name: "CoDe".to_string(),
            mirror: None,
            version_req: None,
        }),
        false,
    )
    .unwrap();
    assert_eq!(base, res);

    // 换回原镜像源
    crate::utils::test::_unmount_custom_mirror(custom_mirror_ctx);
}

#[test]
fn test_info_offline() {
    crate::utils::flags::set_flag(crate::utils::flags::Flag::Confirm, true);
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    assert!(info(
        &crate::types::cfg::Cfg::default(),
        PackageInputEnum::LocalPath("examples/vscode".to_string()),
        true
    )
    .is_err());
}
