use std::fmt::Display;

use anyhow::{anyhow, Result};

use crate::{
    entrances::{info_local, info_online},
    types::{extended_semver::ExSemVer, matcher::PackageInputEnum},
};

use super::{
    mirror::{filter_release, get_url_with_version_req},
    path::find_scope_with_name,
};

pub struct ParsePackageInputRes {
    pub name: String,
    pub scope: String,
    pub current_version: Option<String>,
    pub target_version: String,
    pub download_url: String,
}
pub enum ParseInputResEnum {
    LocalPath(String),
    Url(String),
    PackageMatcher(ParsePackageInputRes),
}

impl Display for ParseInputResEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = match self {
            ParseInputResEnum::LocalPath(p) => format!("{:>12}: {p}", "Local path"),
            ParseInputResEnum::Url(u) => format!("{:>12}: {u}", "URL"),
            ParseInputResEnum::PackageMatcher(p) => {
                let version_tip = if let Some(cur) = &p.current_version {
                    format!("{cur} → {}", p.target_version)
                } else {
                    p.target_version.to_owned()
                };
                format!(
                    "{:>12}: {}/{}    ({})",
                    "Package", p.scope, p.name, version_tip
                )
            }
        };
        writeln!(f, "{line}")
    }
}

pub fn parse_install_input(packages: Vec<String>) -> Result<Vec<ParseInputResEnum>> {
    let mut res: Vec<ParseInputResEnum> = Vec::new();
    for p in packages {
        // 首先解析输入类型
        match PackageInputEnum::parse(p, false, false)? {
            PackageInputEnum::Url(url) => res.push(ParseInputResEnum::Url(url)),
            PackageInputEnum::LocalPath(source_file) => {
                res.push(ParseInputResEnum::LocalPath(source_file))
            }
            // 如果是 PackageMatcher，则解析信息
            PackageInputEnum::PackageMatcher(matcher) => {
                // 查找 scope 并使用 scope 更新纠正大小写
                let (scope, package_name) =
                    find_scope_with_name(&matcher.name, matcher.scope.clone())?;
                // 检查对应包名有没有被安装过
                if let Ok((_, diff)) = info_local(&scope, &package_name) {
                    log!("Warning:Package '{scope}/{package_name}' has been installed({ver}), would be switched to update entrance",ver = diff.version);
                }
                // 解析 url
                let (url, target_release) = get_url_with_version_req(matcher)?;
                res.push(ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
                    name: package_name,
                    scope,
                    current_version: None,
                    target_version: target_release.version.to_string(),
                    download_url: url,
                }))
            }
        };
    }
    Ok(res)
}

pub fn parse_update_input(packages: Vec<String>) -> Result<Vec<ParseInputResEnum>> {
    let mut res: Vec<ParseInputResEnum> = Vec::new();
    for p in packages {
        // 首先解析输入类型
        match PackageInputEnum::parse(p, false, false)? {
            PackageInputEnum::Url(url) => res.push(ParseInputResEnum::Url(url)),
            PackageInputEnum::LocalPath(source_file) => {
                res.push(ParseInputResEnum::LocalPath(source_file))
            }
            // 如果是 PackageMatcher，则解析信息
            PackageInputEnum::PackageMatcher(matcher) => {
                // 查找 scope 并使用 scope 更新纠正大小写
                let (scope, package_name) =
                    find_scope_with_name(&matcher.name, matcher.scope.clone())?;
                // 检查对应包名有没有被安装过
                let (_global, local_diff) = info_local(&scope, &package_name).map_err(|_| {
                    anyhow!("Error:Package '{scope}/{package_name}' hasn't been installed, use 'ept install' instead")
                })?;
                // 检查包的版本号是否允许升级
                let (online_item, _url_template) =
                    info_online(&scope, &package_name, matcher.mirror.clone())?;
                let selected_release =
                    filter_release(online_item.releases, matcher.version_req.clone())?;
                if selected_release.version <= ExSemVer::parse(&local_diff.version)? {
                    return Err(anyhow!("Error:Package '{name}' has been up to date ({local_version}), can't update to the version of given package ({fresh_version})",name=package_name,local_version=&local_diff.version,fresh_version=&selected_release.version));
                }
                // 解析 url
                let (url, target_release) = get_url_with_version_req(matcher)?;
                res.push(ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
                    name: package_name,
                    scope,
                    current_version: None,
                    target_version: target_release.version.to_string(),
                    download_url: url,
                }))
            }
        };
    }
    Ok(res)
}
