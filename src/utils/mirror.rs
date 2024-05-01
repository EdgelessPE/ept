use std::path::PathBuf;

use anyhow::{anyhow, Result};
use fs_extra::file::read_to_string;
use toml::from_str;

use crate::{
    p2s,
    types::{
        mirror::{MirrorHello, Service, ServiceKeys},
        verifiable::Verifiable,
    },
    utils::get_path_mirror,
};

// 读取 meta
pub fn read_local_mirror_meta(name: &String) -> Result<(MirrorHello, PathBuf)> {
    let dir_path = get_path_mirror()?.join(name);
    let p = dir_path.join("meta.toml");
    if !p.exists() {
        return Err(anyhow!("Error:Mirror '{name}' hasn't been added"));
    }
    let text = read_to_string(&p)?;
    let meta: MirrorHello = from_str(&text)
        .map_err(|e| anyhow!("Error:Invalid meta content at '{fp}' : {e}", fp = p2s!(p)))?;
    meta.verify_self(&"".to_string())?;
    Ok((meta, dir_path))
}

// 从 meta 中筛选出服务，返回的第一个参数是拼接了 root_url 后的路径
pub fn filter_service_from_meta(hello: MirrorHello, key: ServiceKeys) -> Result<(String, Service)> {
    let res = hello.service.iter().find(|s| s.key == key);
    if let Some(r) = res {
        Ok((format!("{r}{p}", r = hello.root_url, p = r.path), r.clone()))
    } else {
        Err(anyhow!(
            "Error:Failed to find service '{key:?}' in current mirror meta"
        ))
    }
}
