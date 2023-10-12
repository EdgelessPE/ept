use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    p2s,
    parsers::parse_workflow,
    types::{
        meta::MetaResult,
        package::GlobalPackage,
        permissions::{Generalizable, Permission, PermissionLevel},
    },
    utils::{get_path_apps, path::find_scope_with_name_locally},
};
use anyhow::{anyhow, Result};

use super::{
    info_local,
    utils::{package::unpack_nep, validator::installed_validator},
    verify::verify,
};

// 返回 (临时目录，工作流所在目录，全局包)
fn find_meta_target(
    input: &String,
    verify_signature: bool,
) -> Result<(PathBuf, PathBuf, GlobalPackage)> {
    // 作为路径使用，可以是一个包或者已经解包的目录
    let p = Path::new(input);
    if p.exists() {
        let (path, pkg) = unpack_nep(input, verify_signature)?;
        verify(&p2s!(path))?;
        return Ok((path.clone(), path.join("workflows"), pkg));
    }

    // 作为名称使用，在本地已安装列表中搜索
    if let Ok(scope) = find_scope_with_name_locally(input) {
        let path = get_path_apps(&scope, input, false)?;
        let (pkg, _) = info_local(&scope, input)?;
        installed_validator(&p2s!(path))?;
        return Ok((path.clone(), path.join(".nep_context/workflows"), pkg));
    }

    Err(anyhow!(
        "Error:Failed to find meta by '{input}', input valid path or installed package name"
    ))
}

pub fn meta(input: &String, verify_signature: bool) -> Result<MetaResult> {
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
    let mut map: HashMap<(PermissionLevel, String), HashSet<String>> = HashMap::new();
    for node in total_workflow {
        for perm in node.generalize_permissions()? {
            let entry = map.entry((perm.level, perm.key)).or_insert(HashSet::new());
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
    envmnt::set("CONFIRM", "true");
    let res = meta(&"examples/PermissionsTest".to_string(), false).unwrap();
    println!("{res:#?}");

    // 从本地安装中生成 meta
    crate::utils::test::_ensure_testing_vscode();
    meta(&"VSCode".to_string(), false).unwrap();
    crate::utils::test::_ensure_testing_vscode_uninstalled();
}
