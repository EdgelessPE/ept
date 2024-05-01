use std::fs::write;

use anyhow::{anyhow, Result};
use reqwest::blocking::get;
use toml::{to_string_pretty, Value};

use crate::{
    types::mirror::{MirrorHello, ServiceKeys},
    utils::{
        fs::ensure_dir_exist,
        get_path_mirror,
        mirror::{filter_service_from_meta, read_local_mirror_meta},
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

    // 写 mirror 目录
    let p = get_path_mirror()?.join(&res.name);
    ensure_dir_exist(&p)?;
    let value = Value::try_from(res)?;
    let text = to_string_pretty(&value)?;
    write(p.join("meta.toml"), text)?;

    Ok(())
}

pub fn mirror_update(name: &String) -> Result<()> {
    // 读取 meta 文件
    let meta = read_local_mirror_meta(name)?;
    // 筛选出 hello 服务
    let (hello_path, _) = filter_service_from_meta(meta, ServiceKeys::Hello)?;
    // 调用 add
    mirror_add(&hello_path, Some(name.to_string()))
}

#[test]
fn test_mirror_add() {
    mirror_add(&"http://localhost:3000/api/hello".to_string(), None).unwrap();
}

#[test]
fn test_mirror_update() {
    mirror_update(&"official".to_string()).unwrap();
}
