use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    log, p2s,
    parsers::parse_workflow,
    signature::blake3::compute_hash_blake3_from_string,
    types::{
        cfg::Cfg,
        constants::{
            DIR_NEP_CONTEXT, DIR_WORKFLOWS, WORKFLOW_EXPAND, WORKFLOW_REMOVE, WORKFLOW_SETUP,
            WORKFLOW_UPDATE,
        },
        matcher::PackageInputEnum,
        meta::MetaResult,
        package::GlobalPackage,
        permissions::{Generalizable, Permission, PermissionKey, PermissionLevel},
    },
    utils::{
        cache::spawn_cache, download::download_nep, get_manifest_path, get_path_apps,
        get_path_cache, mirror::filter_release, path::find_scope_with_name,
    },
};
use anyhow::{anyhow, Result};

use super::{
    info_local, info_online,
    utils::{package::unpack_nep, validator::installed_validator},
};

enum MetaTargetResult {
    Local(PathBuf, PathBuf),
    Online(Box<MetaResult>),
}

// 返回 (临时目录，工作流所在目录，全局包)
fn find_meta_target(
    cfg: &crate::types::context::RuntimeContext,
    input: PackageInputEnum,
    verify_signature: bool,
) -> Result<MetaTargetResult> {
    match input {
        PackageInputEnum::LocalPath(local_path) => {
            // 作为路径使用，可以是一个包或者已经解包的目录
            let p = Path::new(&local_path);
            if p.exists() {
                let (path, _) = unpack_nep(cfg, &local_path, verify_signature)?;
                // verify(&p2s!(path))?;
                return Ok(MetaTargetResult::Local(
                    path.clone(),
                    path.join(DIR_WORKFLOWS),
                ));
            }
        }
        PackageInputEnum::PackageMatcher(matcher) => {
            if let Ok((scope, package_name)) =
                find_scope_with_name(cfg, &matcher.name, matcher.scope.as_deref())
            {
                // 先尝试在本地已安装列表中搜索
                let path = get_path_apps(cfg, &scope, &package_name, false)?;
                if info_local(cfg, &scope, &package_name).is_ok() {
                    installed_validator(&p2s!(path))?;
                    return Ok(MetaTargetResult::Local(
                        path.clone(),
                        path.join(DIR_NEP_CONTEXT).join(DIR_WORKFLOWS),
                    ));
                }

                // 直接使用在线 Info 的 Meta 信息
                let (tree_item, _, mirror) =
                    info_online(cfg, &scope, &package_name, matcher.mirror)?;
                let release = filter_release(cfg, tree_item.releases, matcher.version_req, true)?;
                if let Some(meta) = release.meta {
                    log!("Debug:Found meta for '{scope}/{package_name}' in mirror '{mirror}'");
                    return Ok(MetaTargetResult::Online(Box::new(meta)));
                } else {
                    return Err(anyhow!(
                        "Error:Mirror '{mirror}' doesn't provide meta for '{scope}/{package_name}'",
                    ));
                }
            }
        }
        PackageInputEnum::Url(url) => {
            // 下载文件到临时目录
            let cache_path = get_path_cache(cfg)?;
            let url_hash = compute_hash_blake3_from_string(&url)?;
            let (p, cache_ctx) = download_nep(cfg, &url, Some((cache_path, url_hash)))?;

            // 缓存下载的包
            spawn_cache(cache_ctx)?;

            let (path, _) = unpack_nep(cfg, &p2s!(p), verify_signature)?;
            return Ok(MetaTargetResult::Local(
                path.clone(),
                path.join(DIR_WORKFLOWS),
            ));
        }
    }

    Err(anyhow!(
        "Error:Failed to find meta by input, input valid path or installed package matcher"
    ))
}

pub fn meta(cfg: &crate::types::context::RuntimeContext, input: PackageInputEnum, verify_signature: bool) -> Result<MetaResult> {
    match find_meta_target(cfg, input, verify_signature)? {
        MetaTargetResult::Local(temp_dir_inner_path, workflow_path) => {
            let temp_dir = p2s!(temp_dir_inner_path);

            // 检查工作流存在
            let workflow_files = [
                WORKFLOW_SETUP,
                WORKFLOW_UPDATE,
                WORKFLOW_REMOVE,
                WORKFLOW_EXPAND,
            ];
            let exists_workflows: Vec<(String, String)> = workflow_files
                .iter()
                .filter_map(|&name| {
                    let p = workflow_path.join(name);
                    if p.exists() {
                        Some((name.to_string(), p2s!(p)))
                    } else {
                        None
                    }
                })
                .collect();

            // 收集所有工作流
            let total_workflow = exists_workflows
                .clone()
                .into_iter()
                .map(|(_, p)| parse_workflow(&p).unwrap())
                .fold(Vec::new(), |mut acc, mut x| {
                    acc.append(&mut x);
                    acc
                });

            // 收集并合并同类权限
            let mut map: HashMap<(PermissionLevel, PermissionKey), HashSet<String>> =
                HashMap::new();
            for node in total_workflow {
                for perm in node.generalize_permissions(cfg)? {
                    let entry = map.entry((perm.level, perm.key)).or_default();
                    for target in perm.targets {
                        entry.insert(target);
                    }
                }
            }

            let mut permissions = Vec::new();
            for ((level, key), targets) in map {
                permissions.push(Permission {
                    key,
                    level,
                    targets: Vec::from_iter(targets),
                });
            }
            permissions.sort_by(|a, b| {
                if a.level != b.level {
                    a.level.partial_cmp(&b.level).unwrap().reverse()
                } else {
                    a.key.cmp(&b.key)
                }
            });

            // 重新从临时目录读取 package，以解决 package 被解释的问题
            let package_path = get_manifest_path(&temp_dir)?;
            let global: GlobalPackage = toml::from_str(&std::fs::read_to_string(package_path)?)?;

            Ok(MetaResult {
                temp_dir: Some(temp_dir_inner_path),
                permissions,
                workflows: exists_workflows.into_iter().map(|(name, _)| name).collect(),
                package: global,
            })
        }
        MetaTargetResult::Online(meta) => Ok(*meta),
    }
}

#[test]
fn test_meta() {
    use crate::types::matcher::PackageMatcher;
    use crate::utils::test::_default_test_cfg;

    let cfg = &_default_test_cfg();

    // 从本地路径中生成 meta
    let res = meta(
        cfg,
        PackageInputEnum::LocalPath("examples/PermissionsTest".to_string()),
        false,
    )
    .unwrap();
    let mut sorted_permissions: Vec<Permission> = res
        .permissions
        .into_iter()
        .map(|mut node| {
            node.targets.sort();
            node
        })
        .collect();
    sorted_permissions.sort_by(|a, b| a.key.cmp(&b.key));
    assert_eq!(
        sorted_permissions,
        vec![
            Permission {
                key: PermissionKey::path_entrances,
                level: PermissionLevel::Normal,
                targets: vec!["Code.exe".to_string()],
            },
            Permission {
                key: PermissionKey::path_dirs,
                level: PermissionLevel::Important,
                targets: vec!["bin".to_string(),],
            },
            Permission {
                key: PermissionKey::link_desktop,
                level: PermissionLevel::Normal,
                targets: vec![
                    "Build tools".to_string(),
                    "MS/Visual Studio Code".to_string(),
                ],
            },
            Permission {
                key: PermissionKey::link_startmenu,
                level: PermissionLevel::Normal,
                targets: vec!["MS/Visual Studio Code".to_string(),],
            },
            Permission {
                key: PermissionKey::execute_installer,
                level: PermissionLevel::Important,
                targets: vec![
                    "installer.exe /S".to_string(),
                    "uninstaller.exe /S".to_string(),
                    "updater.exe /S".to_string(),
                ],
            },
            Permission {
                key: PermissionKey::execute_custom,
                level: PermissionLevel::Sensitive,
                targets: vec!["unknown.exe --silent".to_string(),],
            },
            Permission {
                key: PermissionKey::fs_read,
                level: PermissionLevel::Sensitive,
                targets: vec!["${ProgramFiles_X86}/Microsoft/32.dll".to_string(),],
            },
            Permission {
                key: PermissionKey::fs_write,
                level: PermissionLevel::Sensitive,
                targets: vec![
                    "${AppData}/pwsh.exe".to_string(),
                    "${ProgramFiles_X64}/Microsoft/64.dll".to_string(),
                    "${SystemDrive}/system32/Windows/".to_string(),
                ],
            },
            Permission {
                key: PermissionKey::fs_write,
                level: PermissionLevel::Important,
                targets: vec![
                    "${Desktop}/Public".to_string(),
                    "${Home}/Download".to_string(),
                ],
            },
            Permission {
                key: PermissionKey::fs_write,
                level: PermissionLevel::Normal,
                targets: vec!["./lib".to_string(),],
            },
            Permission {
                key: PermissionKey::download_file,
                level: PermissionLevel::Important,
                targets: vec!["http://localhost:19191/Code.exe".to_string(),],
            },
            Permission {
                key: PermissionKey::notify_toast,
                level: PermissionLevel::Normal,
                targets: vec![
                    "Updated failed".to_string(),
                    "Updated successfully".to_string(),
                ],
            },
            Permission {
                key: PermissionKey::process_kill,
                level: PermissionLevel::Sensitive,
                targets: vec!["Code.exe".to_string(),],
            },
        ]
    );
    // package 不应该被解释
    let res = meta(
        cfg,
        PackageInputEnum::LocalPath("examples/VSCodeI".to_string()),
        false,
    )
    .unwrap();
    assert_eq!(
        res.package.software.unwrap().main_program.unwrap(),
        "${AppData}/Local/Programs/Microsoft VS Code/Code.exe".to_string()
    );
    let sorted_permissions: Vec<Permission> = res
        .permissions
        .into_iter()
        .map(|mut node| {
            node.targets.sort();
            node
        })
        .collect();
    assert_eq!(
        sorted_permissions,
        vec![Permission {
            key: PermissionKey::execute_installer,
            level: PermissionLevel::Important,
            targets: vec![
                "${AppData}/Local/Programs/Microsoft VS Code/unins000.exe /S".to_string(),
                "installer.exe /S".to_string(),
            ],
        },]
    );

    // 从本地安装中生成 meta
    use crate::types::package::Package;
    use crate::types::software::Software;
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    let vscode_path = crate::utils::test::_ensure_testing_vscode();
    let meta_result = meta(
        cfg,
        PackageInputEnum::PackageMatcher(PackageMatcher {
            name: "VSCode".to_string(),
            scope: None,
            mirror: None,
            version_req: None,
        }),
        false,
    )
    .unwrap();

    // 验证返回的 meta 数据（使用 assert_eq! 分别验证各个字段，避免硬编码路径）
    assert_eq!(meta_result.temp_dir, Some(vscode_path));
    assert_eq!(
        meta_result.permissions,
        vec![
            Permission {
                key: PermissionKey::path_entrances,
                level: PermissionLevel::Normal,
                targets: vec!["Code.exe".to_string()],
            },
            Permission {
                key: PermissionKey::link_desktop,
                level: PermissionLevel::Normal,
                targets: vec!["Visual Studio Code".to_string()],
            }
        ]
    );
    assert_eq!(meta_result.workflows, vec![WORKFLOW_SETUP.to_string()]);
    assert_eq!(
        meta_result.package,
        GlobalPackage {
            nep: "0".to_string(),
            package: Package {
                name: "VSCode".to_string(),
                template: "Software".to_string(),
                version: "1.75.4.0".to_string(),
                authors: vec![
                    "Cno <dsyourshy@qq.com>".to_string(),
                    "Microsoft".to_string()
                ],
                license: Some("MIT".to_string()),
                description: "Visual Studio Code".to_string(),
                scope: "Microsoft".to_string(),
                icon: None,
                strict: None,
            },
            software: Some(Software {
                upstream: "https://code.visualstudio.com/".to_string(),
                category: "办公编辑".to_string(),
                tags: Some(vec!["Electron".to_string()]),
                language: "Multi".to_string(),
                arch: None,
                main_program: Some("Code.exe".to_string()),
                alias: None,
                registry_entry: None,
            }),
        }
    );

    // 从 URL 生成 meta
    crate::utils::test::_ensure_clear_test_dir();
    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "VSCodeE");
    let (url, mut handler) = crate::utils::test::_run_static_file_server();
    crate::entrances::pack(
        cfg,
        "examples/VSCodeE",
        Some("test/VSCodeE.nep".to_string()),
        false,
    )
    .unwrap();
    let res = meta(
        cfg,
        PackageInputEnum::Url(format!("{url}/VSCodeE.nep")),
        false,
    )
    .unwrap();
    assert_eq!(
        res.permissions,
        vec![
            Permission {
                key: PermissionKey::download_file,
                level: PermissionLevel::Important,
                targets: vec!["http://localhost:19191/Code.exe".to_string()],
            },
            Permission {
                key: PermissionKey::path_entrances,
                level: PermissionLevel::Normal,
                targets: vec!["Code.exe".to_string()],
            },
            Permission {
                key: PermissionKey::link_desktop,
                level: PermissionLevel::Normal,
                targets: vec!["Visual Studio Code".to_string()],
            }
        ]
    );
    assert_eq!(res.package.package.name, "VSCodeE");
    assert_eq!(
        res.workflows,
        vec![WORKFLOW_SETUP.to_string(), WORKFLOW_EXPAND.to_string()]
    );
    handler.kill().unwrap();

    // 查询镜像源中的 Meta
    let tup = crate::utils::test::_mount_custom_mirror();
    assert_eq!(
        meta(
            cfg,
            PackageInputEnum::PackageMatcher(
                PackageMatcher::parse("notepad", false, false).unwrap()
            ),
            false
        )
        .unwrap(),
        MetaResult {
            temp_dir: None,
            permissions: vec![Permission {
                key: PermissionKey::path_entrances,
                level: PermissionLevel::Normal,
                targets: vec!["ntpd.exe".to_string()],
            }],
            workflows: vec![WORKFLOW_SETUP.to_string(), WORKFLOW_REMOVE.to_string()],
            package: GlobalPackage {
                nep: "0".to_string(),
                package: Package {
                    name: "Notepad".to_string(),
                    template: "Software".to_string(),
                    version: "22.1.0.0".to_string(),
                    authors: vec![
                        "Bot <bot@edgeless.top>".to_string(),
                        "Cno <cno4tech@gmail.com>".to_string()
                    ],
                    license: Some("MIT".to_string()),
                    description: "Notepad".to_string(),
                    scope: "Microsoft".to_string(),
                    icon: None,
                    strict: None,
                },
                software: Some(Software {
                    upstream: "https://notepad.visualstudio.com/".to_string(),
                    category: "办公编辑".to_string(),
                    tags: Some(vec!["记事本".to_string()]),
                    language: "Multi".to_string(),
                    arch: None,
                    main_program: None,
                    alias: None,
                    registry_entry: None,
                }),
            },
        }
    );

    // Firefox 没有提供 Meta，因此无法获取
    crate::utils::test::_ensure_testing_uninstalled("Mozilla", "Firefox");
    crate::utils::test::_ensure_testing_uninstalled("PortableApps", "Firefox");
    assert!(meta(
        cfg,
        PackageInputEnum::PackageMatcher(PackageMatcher::parse("firefox", false, false).unwrap()),
        false
    )
    .is_err());

    crate::utils::test::_unmount_custom_mirror(tup);
}
