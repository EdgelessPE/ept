use std::fmt::Display;

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    entrances::{auto_mirror_update_all, info_local, info_online},
    types::{
        extended_semver::ExSemVer,
        matcher::{PackageInputEnum, PackageMatcher},
    },
    utils::fmt_print::fmt_package_line,
};

use super::{
    cfg::get_config,
    get_path_apps,
    mirror::{filter_release, get_url_with_version_req},
    path::find_scope_with_name,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ParsePackageInputRes {
    pub name: String,
    pub scope: String,
    pub current_version: Option<String>,
    pub target_version: String,
    pub download_url: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
                    "{:>12}: {}",
                    "Package",
                    fmt_package_line(&p.scope, &p.name, &version_tip, None)
                )
            }
        };
        write!(f, "{line}")
    }
}

impl ParseInputResEnum {
    // 打印内联的预览语句
    pub fn preview(&self) -> String {
        match self {
            ParseInputResEnum::LocalPath(p) => format!("{}: {p}", "local path"),
            ParseInputResEnum::Url(u) => format!("{}: {u}", "url"),
            ParseInputResEnum::PackageMatcher(p) => {
                let version_tip = if let Some(cur) = &p.current_version {
                    format!("{cur} → {}", p.target_version)
                } else {
                    p.target_version.to_owned()
                };
                format!(
                    "{}: {}/{} ({})",
                    "package",
                    &p.scope.truecolor(100, 100, 100).italic(),
                    &p.name.cyan().bold(),
                    version_tip
                )
            }
        }
    }
}

pub fn parse_install_inputs(packages: Vec<String>) -> Result<Vec<ParseInputResEnum>> {
    let mut res: Vec<ParseInputResEnum> = Vec::new();
    let mut mirror_updated = false;
    for p in packages {
        // 首先解析输入类型
        match PackageInputEnum::parse(p, false, false)? {
            PackageInputEnum::Url(url) => res.push(ParseInputResEnum::Url(url)),
            PackageInputEnum::LocalPath(source_file) => {
                res.push(ParseInputResEnum::LocalPath(source_file))
            }
            // 如果是 PackageMatcher，则解析信息
            PackageInputEnum::PackageMatcher(matcher) => {
                // 更新镜像源
                if !mirror_updated {
                    let cfg = get_config();
                    auto_mirror_update_all(&cfg)?;
                    mirror_updated = true;
                }
                // 查找 scope 并使用 scope 更新纠正大小写
                let (scope, package_name) =
                    find_scope_with_name(&matcher.name, matcher.scope.clone())?;
                // 检查对应包名有没有被安装过，如果安装过就作为 update 解析
                if let Ok((_, diff)) = info_local(&scope, &package_name) {
                    log!("Warning:Package '{scope}/{package_name}' has been installed({ver}), would be switched to update entrance",ver = diff.version);
                    let mut update_parsed =
                        parse_update_inputs(vec![format!("{scope}/{package_name}")])?;
                    res.append(&mut update_parsed);
                    continue;
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

pub fn parse_update_inputs(packages: Vec<String>) -> Result<Vec<ParseInputResEnum>> {
    let mut res: Vec<ParseInputResEnum> = Vec::new();
    let mut mirror_updated = false;
    for p in packages {
        // 首先解析输入类型
        match PackageInputEnum::parse(p, false, false)? {
            PackageInputEnum::Url(url) => res.push(ParseInputResEnum::Url(url)),
            PackageInputEnum::LocalPath(source_file) => {
                res.push(ParseInputResEnum::LocalPath(source_file))
            }
            // 如果是 PackageMatcher，则解析信息
            PackageInputEnum::PackageMatcher(matcher) => {
                // 更新镜像源
                if !mirror_updated {
                    let cfg = get_config();
                    auto_mirror_update_all(&cfg)?;
                    mirror_updated = true;
                }
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
                    filter_release(online_item.releases, matcher.version_req.clone(), true)?;
                if selected_release.version <= ExSemVer::parse(&local_diff.version)? {
                    return Err(anyhow!("Error:Package '{name}' has been up to date ({local_version}), can't update to the version of given package ({fresh_version})",name=package_name,local_version=&local_diff.version,fresh_version=&selected_release.version));
                }
                // 解析 url
                let (url, target_release) = get_url_with_version_req(matcher)?;
                res.push(ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
                    name: package_name,
                    scope,
                    current_version: Some(local_diff.version),
                    target_version: target_release.version.to_string(),
                    download_url: url,
                }))
            }
        };
    }
    Ok(res)
}

pub fn parse_uninstall_inputs(packages: Vec<String>) -> Result<Vec<(String, String, String)>> {
    let mut arr = Vec::new();
    for p in packages {
        // 简单校验是否可以卸载
        let parse_res = PackageMatcher::parse(&p, true, true)?;

        // 查找 scope 并使用 scope 更新纠正大小写
        let (scope, package_name) = find_scope_with_name(&parse_res.name, parse_res.scope)?;

        // 解析安装路径
        let app_path = get_path_apps(&scope, &package_name, false)?;
        if !app_path.exists() {
            return Err(anyhow!("Error:Package '{p}' not installed"));
        }

        // 查询版本号
        let version = if let Ok((_, local_diff)) = info_local(&scope, &package_name) {
            local_diff.version
        } else {
            "broken".to_string()
        };

        arr.push((scope, package_name, version));
    }

    Ok(arr)
}

#[test]
fn test_print_enum() {
    // 提升覆盖率用
    assert!(ParseInputResEnum::LocalPath("test".to_string())
        .to_string()
        .contains("test"));
    assert!(ParseInputResEnum::LocalPath("test".to_string())
        .preview()
        .contains("test"));

    assert!(ParseInputResEnum::Url("http://localhost/test".to_string())
        .to_string()
        .contains("http://localhost/test"));
    assert!(ParseInputResEnum::Url("http://localhost/test".to_string())
        .preview()
        .contains("http://localhost/test"));

    assert!(ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
        name: "test".to_string(),
        scope: "test".to_string(),
        current_version: None,
        target_version: "test".to_string(),
        download_url: "URL_ADDRESS".to_string()
    })
    .to_string()
    .contains("test"));
    assert!(ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
        name: "test".to_string(),
        scope: "test".to_string(),
        current_version: None,
        target_version: "test".to_string(),
        download_url: "URL_ADDRESS".to_string()
    })
    .preview()
    .contains("test"));
    assert!(ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
        name: "test".to_string(),
        scope: "test".to_string(),
        current_version: Some("1.75.4.0".to_string()),
        target_version: "test".to_string(),
        download_url: "URL_ADDRESS".to_string()
    })
    .to_string()
    .contains("test"));
    assert!(ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
        name: "test".to_string(),
        scope: "test".to_string(),
        current_version: Some("1.75.4.0".to_string()),
        target_version: "test".to_string(),
        download_url: "URL_ADDRESS".to_string()
    })
    .preview()
    .contains("test"));
}

#[test]
fn test_parse_inputs() {
    envmnt::set("DEBUG", "true");
    envmnt::set("CONFIRM", "true");

    // 使用 mock 的镜像数据
    let mock_ctx = crate::utils::test::_use_mock_mirror_data();

    // 先卸载 vscode
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    // 测试安装的解析
    let res = parse_install_inputs(vec![
        "examples/VSCode".to_string(),
        "vscode".to_string(),
        "http://localhost/vscode.nep".to_string(),
    ])
    .unwrap();
    assert_eq!(
        res,
        vec![
            ParseInputResEnum::LocalPath("examples/VSCode".to_string()),
            ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
                name: "VSCode".to_string(),
                scope: "Microsoft".to_string(),
                current_version: None,
                target_version: "1.75.4.2".to_string(),
                download_url: "http://localhost:19191/static/VSCode_1.75.4.2_Cno.nep?scope=Microsoft&software=VSCode".to_string()
            }),
            ParseInputResEnum::Url("http://localhost/vscode.nep".to_string()),
        ]
    );
    // 测试更新的解析
    assert!(parse_update_inputs(vec!["vscode".to_string(),]).is_err());

    // 安装 vscode
    crate::utils::test::_ensure_testing_vscode();
    // 测试安装的解析
    let res = parse_install_inputs(vec![
        "examples/VSCode".to_string(),
        "vscode".to_string(),
        "http://localhost/vscode.nep".to_string(),
    ])
    .unwrap();
    assert_eq!(
        res,
        vec![
            ParseInputResEnum::LocalPath("examples/VSCode".to_string()),
            ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
                name: "VSCode".to_string(),
                scope: "Microsoft".to_string(),
                current_version: Some("1.75.4.0".to_string()),
                target_version: "1.75.4.2".to_string(),
                download_url: "http://localhost:19191/static/VSCode_1.75.4.2_Cno.nep?scope=Microsoft&software=VSCode".to_string()
            }),
            ParseInputResEnum::Url("http://localhost/vscode.nep".to_string()),
        ]
    );
    // 测试更新的解析
    let res = parse_update_inputs(vec![
        "examples/VSCode".to_string(),
        "vscode".to_string(),
        "http://localhost/vscode.nep".to_string(),
    ])
    .unwrap();
    assert_eq!(
        res,
        vec![
            ParseInputResEnum::LocalPath("examples/VSCode".to_string()),
            ParseInputResEnum::PackageMatcher(ParsePackageInputRes {
                name: "VSCode".to_string(),
                scope: "Microsoft".to_string(),
                current_version: Some("1.75.4.0".to_string()),
                target_version: "1.75.4.2".to_string(),
                download_url: "http://localhost:19191/static/VSCode_1.75.4.2_Cno.nep?scope=Microsoft&software=VSCode".to_string()
            }),
            ParseInputResEnum::Url("http://localhost/vscode.nep".to_string()),
        ]
    );

    // 测试卸载的解析
    let res = parse_uninstall_inputs(vec!["vscode".to_string()]).unwrap();
    assert_eq!(
        res,
        vec![(
            "Microsoft".to_string(),
            "VSCode".to_string(),
            "1.75.4.0".to_string()
        ),]
    );
    let res = parse_uninstall_inputs(vec!["microSOFT/Vscode".to_string()]).unwrap();
    assert_eq!(
        res,
        vec![(
            "Microsoft".to_string(),
            "VSCode".to_string(),
            "1.75.4.0".to_string()
        )]
    );

    crate::utils::test::_restore_mirror_data(mock_ctx);
    crate::utils::test::_ensure_testing_vscode_uninstalled();
}
