use anyhow::{anyhow, Result};
use humantime::parse_duration;
use reqwest::blocking::get;
use std::{
    fs::{metadata, write},
    time::SystemTime,
};
use toml::{to_string_pretty, Value};
use url::Url;

use crate::types::cfg::Cfg;
use crate::{
    log, log_ok_last,
    types::{
        mirror::{MirrorHello, MirrorPkgSoftware, ServiceKeys},
        verifiable::Verifiable,
    },
    utils::{
        fs::{ensure_dir_exist, read_sub_dir, try_recycle},
        get_path_mirror,
        mirror::{build_index_for_mirror, filter_service_from_meta, read_local_mirror_hello},
    },
};

// 返回远程镜像源申明的名称
pub fn mirror_add(url: &String, should_match_name: Option<String>) -> Result<String> {
    // 尝试解析为 URL 对象
    let parsed_url =
        Url::parse(url).map_err(|e| anyhow!("Error:Failed to parse '{url}' as valid URL : {e}"))?;

    // 没有路径则会自动加上 /api/hello
    let url = if parsed_url.path() == "/" {
        parsed_url.join("/api/hello").unwrap().to_string()
    } else {
        url.to_string()
    };
    log!("Debug:Hand shaking with '{url}'...");

    // 请求 url
    let res: MirrorHello = get(&url)
        .map_err(|e| anyhow!("Error:Failed to fetch '{url}' : {e}"))?
        .json()
        .map_err(|e| {
            anyhow!("Error:Failed to decode response as valid hello content from '{url}' : {e}")
        })?;
    let mirror_name = res.name.clone();

    // 检查名称是否符合
    if let Some(n) = should_match_name {
        if mirror_name != n {
            return Err(anyhow!("Error:Mirror has changed its registry name (from '{n}' to '{mirror_name}'), use 'ept mirror remove {n}' to remove the old mirror first"));
        }
    }

    // 校验
    res.verify_self(&"".to_string())?;

    // 请求软件包列表
    let (ps_url, _) = filter_service_from_meta(res.clone(), ServiceKeys::PkgSoftware)?;
    log!("Debug:Fetching software list from '{ps_url}'...");
    let pkg_software_res: MirrorPkgSoftware = get(&ps_url)
        .map_err(|e| anyhow!("Error:Failed to fetch '{ps_url}' : {e}"))?
        .json()
        .map_err(|e| {
            anyhow!(
                "Error:Failed to decode response as valid software content from '{ps_url}' : {e}"
            )
        })?;

    // 校验
    pkg_software_res.verify_self(&"".to_string())?;

    // 更新索引并写 pkg-software.toml
    let p = get_path_mirror()?.join(&mirror_name);
    build_index_for_mirror(pkg_software_res.clone(), p.join("index"))?;
    let value = Value::try_from(pkg_software_res)?;
    let text = to_string_pretty(&value)?;
    write(p.join("pkg-software.toml"), text)?;

    // [defer] 写 hello.toml
    ensure_dir_exist(&p)?;
    let value = Value::try_from(res)?;
    let text = to_string_pretty(&value)?;
    write(p.join("hello.toml"), text)?;

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

pub fn mirror_list() -> Result<Vec<(String, SystemTime)>> {
    let p = get_path_mirror()?;
    let mut res = Vec::new();
    for name in read_sub_dir(&p)? {
        let file_path = p.join(&name).join("hello.toml");
        let time = metadata(file_path)?.modified()?;

        res.push((name, time));
    }
    Ok(res)
}

pub fn mirror_update_all() -> Result<Vec<String>> {
    let p = get_path_mirror()?;
    let mut names = Vec::new();
    for name in read_sub_dir(p)? {
        let n = mirror_update(&name)?;
        names.push(n);
    }
    Ok(names)
}

// 根据 config 中的超时配置自动判断是否需要更新镜像
pub fn auto_mirror_update_all(cfg: &Cfg) -> Result<bool> {
    // 读取配置
    let duration_cfg = parse_duration(&cfg.online.mirror_update_interval).map_err(|e| anyhow!("Error:Failed to parse config field 'online.mirror_update_interval' as valid time span : {e}, e.g. '5d' '14m54s'"))?;
    let now = SystemTime::now();
    log!(
        "Debug:Mirror update interval : '{i}'",
        i = &cfg.online.mirror_update_interval
    );

    // 列出镜像源，如果其中有一个过期就更新全部
    let ls = mirror_list()?;
    let res = ls
        .into_iter()
        .find(|(_, modified_time)| now.duration_since(*modified_time).unwrap() > duration_cfg);
    if res.is_some() {
        log!("Info:Automatically updating mirror index...");
        mirror_update_all()?;
        log_ok_last!("Info:Automatically updating mirror index...");
        Ok(true)
    } else {
        log!("Debug:No outdated mirror");
        Ok(false)
    }
}

pub fn mirror_remove(name: &String) -> Result<()> {
    // 获取目录路径
    let (_, p) = read_local_mirror_hello(name)?;
    // 移除目录
    try_recycle(p)
}

#[test]
fn test_mirror() {
    envmnt::set("DEBUG", "true");
    use crate::entrances::search;
    use crate::utils::test::_run_mirror_mock_server;
    use std::fs::{remove_dir_all, rename};
    use std::thread::sleep;
    use std::time::Duration;

    // 备份原有的镜像文件夹
    let origin_p = get_path_mirror().unwrap();
    let bak_p = origin_p.parent().unwrap().join("mirror_bak");
    let has_origin_mirror = origin_p.exists();
    if has_origin_mirror {
        if bak_p.exists() {
            remove_dir_all(&origin_p).unwrap();
        } else {
            rename(&origin_p, &bak_p).unwrap();
        }
    }
    assert!(mirror_list().unwrap().is_empty());

    // 此时搜不到内容
    assert!(search(&"vscode".to_string(), false).is_err());

    // 启动 mock 服务器
    let mock_url = _run_mirror_mock_server();

    // 测试添加
    mirror_add(&mock_url, None).unwrap();

    // 测试列出
    let ls = mirror_list().unwrap();
    assert_eq!(ls.len(), 1);
    let (name, old_update_time) = ls.first().unwrap();
    assert_eq!(name, "mock-server");

    // 测试搜索
    let expected_res = vec![crate::types::mirror::SearchResult {
        name: "VSCode".to_string(),
        scope: "Microsoft".to_string(),
        version: "1.75.4.2".to_string(),
        from_mirror: Some("mock-server".to_string()),
    }];
    let search_res = search(&"vscode".to_string(), false).unwrap();
    assert_eq!(search_res, expected_res);
    let search_res = search(&r"vs\w+".to_string(), true).unwrap();
    assert_eq!(search_res, expected_res);
    assert!(search(&"microsoft".to_string(), false).is_err());

    // 测试更新
    sleep(Duration::from_micros(100));
    mirror_update(&"mock-server".to_string()).unwrap();
    let ls = mirror_list().unwrap();
    let (_, new_update_time) = ls.first().unwrap();
    assert!(new_update_time.duration_since(*old_update_time).unwrap() > Duration::from_micros(50));

    // 测试移除
    mirror_remove(&"mock-server".to_string()).unwrap();
    assert!(mirror_list().unwrap().is_empty());

    // 还原原有的镜像文件夹
    if has_origin_mirror {
        remove_dir_all(&origin_p).unwrap();
        rename(&bak_p, &origin_p).unwrap();
    }
}
#[test]
fn test_auto_mirror_update_all() {
    use crate::utils::test::_run_mirror_mock_server;
    use std::fs::{remove_dir_all, rename};
    use std::thread::sleep;
    use std::time::Duration;

    let cfg = Cfg::default();

    // 备份原有的镜像文件夹
    let origin_p = get_path_mirror().unwrap();
    let bak_p = origin_p.parent().unwrap().join("mirror_bak");
    let has_origin_mirror = origin_p.exists();
    if has_origin_mirror {
        if bak_p.exists() {
            remove_dir_all(&origin_p).unwrap();
        } else {
            rename(&origin_p, &bak_p).unwrap();
        }
    }
    assert!(mirror_list().unwrap().is_empty());

    // 启动 mock 服务器
    let mock_url = _run_mirror_mock_server();

    mirror_add(&mock_url, None).unwrap();

    // 使用默认的 1d 过期配置，不会导致更新
    assert!(!auto_mirror_update_all(&cfg).unwrap());

    // 创建一个短过期配置，等 2s 后会导致更新
    let mut short_cfg = cfg.clone();
    short_cfg.online.mirror_update_interval = "1s".to_string();
    sleep(Duration::from_secs(2));

    assert!(auto_mirror_update_all(&short_cfg).unwrap());

    // 还原原有的镜像文件夹
    if has_origin_mirror {
        remove_dir_all(&origin_p).unwrap();
        rename(&bak_p, &origin_p).unwrap();
    }
}

// #[test]
// fn test_mirror_add() {
//     mirror_add(&"http://localhost:3000/".to_string(), None).unwrap();
// }

// #[test]
// fn test_mirror_update() {
//     mirror_update(&"official".to_string()).unwrap();
// }

// #[test]
// fn test_mirror_list() {
//     let res = mirror_list().unwrap();
//     println!("{res:#?}")
// }

// #[test]
// fn test_mirror_remove() {
//     mirror_remove(&"official".to_string()).unwrap();
// }
