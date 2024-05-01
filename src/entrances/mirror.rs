use std::fs::write;

use anyhow::{anyhow, Result};
use reqwest::blocking::get;
use toml::{to_string_pretty, Value};

use crate::{
    types::mirror::MirrorHello,
    utils::{fs::ensure_dir_exist, get_path_mirror},
};

pub fn mirror_add(url: &String) -> Result<()> {
    // 请求 url
    let res: MirrorHello = get(url)?.json()?;

    // 写 mirror 目录
    let p = get_path_mirror()?.join(&res.name);
    ensure_dir_exist(&p)?;
    let value = Value::try_from(res)?;
    let text = to_string_pretty(&value)?;
    write(p.join("meta.toml"), text)?;

    Ok(())
}

#[test]
fn test_mirror_add() {
    mirror_add(&"http://localhost:3000/api/hello".to_string()).unwrap();
}
