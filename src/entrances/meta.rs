use std::collections::{HashMap, HashSet};

use crate::{
    p2s,
    parsers::parse_workflow,
    types::{meta::MetaResult, permissions::{Generalizable, PermissionLevel, Permission}},
};
use anyhow::Result;

use super::{utils::package::unpack_nep, verify::verify};

pub fn meta(source_file: &String, verify_signature: bool) -> Result<MetaResult> {
    // 解包
    let (temp_dir_inner_path, global) = unpack_nep(source_file, verify_signature)?;
    let temp_dir = p2s!(temp_dir_inner_path);

    // 校验
    verify(&temp_dir)?;

    // 检查工作流存在
    let exists_workflows: Vec<String> = vec!["setup.toml", "update.toml", "remove.toml"]
        .into_iter()
        .filter_map(|name| {
            let p = temp_dir_inner_path.join("workflows").join(name);
            if p.exists(){
                Some(p2s!(p))
            }else{
                None
            }
        })
        .collect();

    // 收集所有工作流
    let total_workflow = exists_workflows
        .clone()
        .into_iter()
        .map(|p| {
            parse_workflow(&p).unwrap()
        })
        .fold(Vec::new(), |mut acc, mut x| {
            acc.append(&mut x);
            acc
        });

    // 收集并合并同类权限
    let mut map:HashMap<(PermissionLevel,String), HashSet<String>>=HashMap::new();
    for node in total_workflow{
        for perm in node.generalize_permissions()?{
            let entry=map.entry((perm.level,perm.key)).or_insert(HashSet::new());
            for target in perm.targets{
                entry.insert(target);
            }
        }
    }

    // println!("map {map:#?}");

    let mut permissions = Vec::new();
    for ((level,key),targets) in map {
        permissions.push(Permission{
            key,
            level,
            targets:Vec::from_iter(targets)
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
        workflows: exists_workflows
            .into_iter()
            .map(|str| str.to_string())
            .collect(),
        package: global,
    })
}

#[test]
fn test_meta() {
    let res = meta(
        &"./examples/PermissionsTest".to_string(),
        false,
    );
    println!("{res:#?}");
    assert!(res.is_ok());
}
