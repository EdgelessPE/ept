use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    p2s,
    parsers::parse_workflow,
    types::{
        matcher::PackageInputEnum,
        meta::MetaResult,
        package::GlobalPackage,
        permissions::{Generalizable, Permission, PermissionKey, PermissionLevel},
    },
    utils::{get_path_apps, path::find_scope_with_name},
};
use anyhow::{anyhow, Result};

use super::{
    info_local,
    utils::{package::unpack_nep, validator::installed_validator},
    verify::verify,
};

// 返回 (临时目录，工作流所在目录，全局包)
fn find_meta_target(
    input: PackageInputEnum,
    verify_signature: bool,
) -> Result<(PathBuf, PathBuf, GlobalPackage)> {
    match input {
        PackageInputEnum::LocalPath(local_path) => {
            // 作为路径使用，可以是一个包或者已经解包的目录
            let p = Path::new(&local_path);
            if p.exists() {
                let (path, pkg) = unpack_nep(&local_path, verify_signature)?;
                verify(&p2s!(path))?;
                return Ok((path.clone(), path.join("workflows"), pkg));
            }
        }
        PackageInputEnum::PackageMatcher(matcher) => {
            // 作为名称使用，在本地已安装列表中搜索
            if let Ok((scope, package_name)) = find_scope_with_name(&matcher.name, matcher.scope) {
                let path = get_path_apps(&scope, &package_name, false)?;
                let (pkg, _) = info_local(&scope, &package_name)?;
                installed_validator(&p2s!(path))?;
                return Ok((path.clone(), path.join(".nep_context/workflows"), pkg));
            }
        }
        PackageInputEnum::Url(_) => {
            return Err(anyhow!("Error:URL is not acceptable"));
        }
    }

    Err(anyhow!(
        "Error:Failed to find meta by input, input valid path or installed package matcher"
    ))
}

pub fn meta(input: PackageInputEnum, verify_signature: bool) -> Result<MetaResult> {
    // 解包
    let (temp_dir_inner_path, workflow_path, global) = find_meta_target(input, verify_signature)?;
    let temp_dir = p2s!(temp_dir_inner_path);

    // 检查工作流存在
    let exists_workflows: Vec<(String, String)> = vec!["setup.toml", "update.toml", "remove.toml"]
        .into_iter()
        .filter_map(|name| {
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
    let mut map: HashMap<(PermissionLevel, PermissionKey), HashSet<String>> = HashMap::new();
    for node in total_workflow {
        for perm in node.generalize_permissions()? {
            let entry = map.entry((perm.level, perm.key)).or_default();
            for target in perm.targets {
                entry.insert(target);
            }
        }
    }

    // println!("map {map:#?}");

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

    Ok(MetaResult {
        temp_dir,
        permissions,
        workflows: exists_workflows.into_iter().map(|(name, _)| name).collect(),
        package: global,
    })
}

#[test]
fn test_meta() {
    use crate::types::matcher::PackageMatcher;
    use crate::utils::envmnt;
    envmnt::set("CONFIRM", "true");
    let res = meta(
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
                    "installer /S".to_string(),
                    "uninstaller /S".to_string(),
                    "updater /S".to_string(),
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

    // 从本地安装中生成 meta
    crate::utils::test::_ensure_testing_vscode();
    meta(
        PackageInputEnum::PackageMatcher(PackageMatcher {
            name: "VSCode".to_string(),
            scope: None,
            mirror: None,
            version_req: None,
        }),
        false,
    )
    .unwrap();
    crate::utils::test::_ensure_testing_vscode_uninstalled();
}
