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

pub fn mirror_add(url: &String, should_match_name: Option<String>) -> Result<()> {
    // 请求 url
    let res: MirrorHello = get(url)?.json()?;

    // 检查名称是否符合
    if let Some(n) = should_match_name {
        if res.name != n {
            return Err(anyhow!("Error:Mirror has changed its registry name (from '{n}' to '{m}'), use 'ept mirror remove {n}' to remove the old mirror first",m=res.name));
        }
    }

    // 校验
    res.verify_self(&"".to_string())?;

    // 写 hello.toml
    let p = get_path_mirror()?.join(&res.name);
    ensure_dir_exist(&p)?;
    let value = Value::try_from(res.clone())?;
    let text = to_string_pretty(&value)?;
    write(p.join("hello.toml"), text)?;

    // 请求软件包列表
    let (ps_url, _) = filter_service_from_meta(res, ServiceKeys::PkgSoftware)?;
    let pkg_software_res: MirrorPkgSoftware = get(&ps_url)?.json()?;

    // 更新索引并写 pkg-software.toml
    build_index_for_mirror(pkg_software_res.clone(), p.join("index"))?;
    let value = Value::try_from(pkg_software_res.clone())?;
    let text = to_string_pretty(&value)?;
    write(p.join("pkg-software.toml"), text)?;

    Ok(())
}

pub fn mirror_update(name: &String) -> Result<()> {
    // 读取 meta 文件
    let (meta, _) = read_local_mirror_hello(name)?;
    // 筛选出 hello 服务
    let (hello_path, _) = filter_service_from_meta(meta, ServiceKeys::Hello)?;
    // 调用 add
    mirror_add(&hello_path, Some(name.to_string()))
}

pub fn mirror_remove(name: &String) -> Result<()> {
    // 获取目录路径
    let (_, p) = read_local_mirror_hello(name)?;
    // 移除目录
    try_recycle(p)
}

pub fn mirror_search(text: &String) -> Result<Vec<SearchResult>> {
    // 扫描出所有的镜像源目录
    let root = get_path_mirror()?;
    let mirror_dirs = read_sub_dir(&root)?;
    if mirror_dirs.len() == 0 {
        return Err(anyhow!("Error:No mirror added yet"));
    }

    // 添加扫描结果
    let mut arr = Vec::new();
    for mirror_name in mirror_dirs {
        let search_res = search_index_for_mirror(text, root.join(&mirror_name).join("index"))?;
        let mut mapped: Vec<SearchResult> = search_res
            .iter()
            .map(|raw| {
                let mut node = raw.to_owned();
                node.from_mirror = Some(mirror_name.clone());
                node
            })
            .collect();
        arr.append(&mut mapped);
    }

    Ok(arr)
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

// #[test]
// fn test_mirror_search() {
//     let res=mirror_search(&"code".to_string()).unwrap();
//     println!("{res:#?}");
// }
