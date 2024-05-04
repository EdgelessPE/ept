use std::fs::write;

use anyhow::{anyhow, Result};
use reqwest::blocking::get;
use toml::{to_string_pretty, Value};

use crate::{
    types::{
        mirror::{MirrorHello, MirrorPkgSoftware, SearchResult, ServiceKeys},
        verifiable::Verifiable,
    },
    utils::{
        fs::{ensure_dir_exist, read_sub_dir, try_recycle},
        get_path_mirror,
        mirror::{
            build_index_for_mirror, filter_service_from_meta, read_local_mirror_hello,
            search_index_for_mirror,
        },
    },
};

pub fn mirror_add(url: &String, should_match_name: Option<String>) -> Result<String> {
    // 请求 url
    let res: MirrorHello = get(url)?.json()?;
    let mirror_name = res.name.clone();

    // 检查名称是否符合
    if let Some(n) = should_match_name {
        if mirror_name != n {
            return Err(anyhow!("Error:Mirror has changed its registry name (from '{n}' to '{mirror_name}'), use 'ept mirror remove {n}' to remove the old mirror first"));
        }
    }

    // 校验
    res.verify_self(&"".to_string())?;

    // 写 hello.toml
    let p = get_path_mirror()?.join(&mirror_name);
    ensure_dir_exist(&p)?;
    let value = Value::try_from(res.clone())?;
    let text = to_string_pretty(&value)?;
    write(p.join("hello.toml"), text)?;

    // 请求软件包列表
    let (ps_url, _) = filter_service_from_meta(res, ServiceKeys::PkgSoftware)?;
    let pkg_software_res: MirrorPkgSoftware = get(&ps_url)?.json()?;

    // 校验
    pkg_software_res.verify_self(&"".to_string())?;

    // 更新索引并写 pkg-software.toml
    build_index_for_mirror(pkg_software_res.clone(), p.join("index"))?;
    let value = Value::try_from(pkg_software_res.clone())?;
    let text = to_string_pretty(&value)?;
    write(p.join("pkg-software.toml"), text)?;

    Ok(mirror_name)
}

pub fn mirror_update(name: &String) -> Result<String> {
    // 读取 meta 文件
    let (meta, _) = read_local_mirror_hello(name)?;
    // 筛选出 hello 服务
    let (hello_path, _) = filter_service_from_meta(meta, ServiceKeys::Hello)?;
    // 调用 add
    mirror_add(&hello_path, Some(name.to_string()))
}

pub fn mirror_update_all() -> Result<Vec<String>> {
    let p = get_path_mirror()?;
    let mut names = Vec::new();
    for name in read_sub_dir(&p)? {
        let n = mirror_update(&name)?;
        names.push(n);
    }
    Ok(names)
}

pub fn mirror_remove(name: &String) -> Result<()> {
    // 获取目录路径
    let (_, p) = read_local_mirror_hello(name)?;
    // 移除目录
    try_recycle(p)
}

// #[test]
// fn test_mirror_add() {
//     mirror_add(&"http://localhost:3000/api/hello".to_string(), None).unwrap();
// }

// #[test]
// fn test_mirror_update() {
//     mirror_update(&"official".to_string()).unwrap();
// }

// #[test]
// fn test_mirror_remove() {
//     mirror_remove(&"official".to_string()).unwrap();
// }
