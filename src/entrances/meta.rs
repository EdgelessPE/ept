use crate::{
    p2s,
    parsers::parse_workflow,
    types::{meta::MetaResult, permissions::Generalizable},
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
    let exists_workflows: Vec<&str> = vec!["setup.toml", "update.toml", "remove.toml"]
        .into_iter()
        .filter(|name| {
            let p = temp_dir_inner_path.join("workflows").join(name);
            p.exists()
        })
        .collect();

    // 收集所有工作流
    let total_workflow = exists_workflows
        .clone()
        .into_iter()
        .map(|name| {
            let p = temp_dir_inner_path.join("workflows").join(name);
            parse_workflow(&p2s!(p)).unwrap()
        })
        .fold(Vec::new(), |mut acc, mut x| {
            acc.append(&mut x);
            acc
        });

    // 收集权限
    let mut permissions = Vec::new();
    for node in total_workflow {
        permissions.append(&mut node.generalize_permissions()?);
    }
    permissions.sort_by(|a, b| {
        if a.level != b.level {
            a.level.partial_cmp(&b.level).unwrap()
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
        &r"D:\Desktop\Projects\EdgelessPE\edgeless-bot\workshop\360压缩\_ready".to_string(),
        false,
    );
    println!("{res:#?}");
}
