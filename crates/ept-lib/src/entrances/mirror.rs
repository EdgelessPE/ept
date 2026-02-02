use crate::utils::request::get;
use anyhow::{anyhow, Result};
use humantime::parse_duration;
use std::{
    fs::{metadata, write},
    time::SystemTime,
};
use toml::{to_string_pretty, Value};
use url::Url;

use crate::{
    log, log_ok_last,
    types::{
        cfg::Cfg,
        context::VerifiableCtx,
        mirror::{MirrorEptToolchain, MirrorHello, MirrorInfo, MirrorPkgSoftware, ServiceKeys},
        mixed_fs::MixedFS,
        verifiable::Verifiable,
    },
    utils::{
        constants::{MIRROR_FILE_EPT_TOOLCHAIN, MIRROR_FILE_HELLO},
        fs::{ensure_dir_exist, read_sub_dir, try_recycle},
        get_path_mirror,
        mirror::{build_index_for_mirror, filter_service_from_meta, read_local_mirror_hello},
    },
};

// 返回远程镜像源申明的名称
pub fn mirror_add(cfg: &crate::types::context::RuntimeContext, url: &str, should_match_name: Option<String>) -> Result<String> {
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
    let mixed_fs = MixedFS::new("");
    let ctx = VerifiableCtx {
        mixed_fs: &mixed_fs,
        runtime_ctx: &cfg,
    };
    res.verify_self(&ctx)?;

    // 请求软件包列表
    let (ps_url, _) = filter_service_from_meta(&res, ServiceKeys::PkgSoftware)?;
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
    pkg_software_res.verify_self(&ctx)?;

    // 更新索引并写 pkg-software.toml
    let p = get_path_mirror(cfg)?.join(&mirror_name);
    build_index_for_mirror(cfg, pkg_software_res.clone(), p.join("index"))?;
    // let value = Value::try_from(pkg_software_res)?;
    // let text = to_string_pretty(&value)?;
    // write(p.join(MIRROR_FILE_PKG_SOFTWARE), text)?;

    // 请求工具链服务
    if let Ok((ps_url, _)) = filter_service_from_meta(&res, ServiceKeys::EptToolchain) {
        log!("Debug:Fetching ept toolchain data from '{ps_url}'...");
        let res: MirrorEptToolchain = get(&ps_url)
            .map_err(|e| anyhow!("Error:Failed to fetch '{ps_url}' : {e}"))?
            .json()
            .map_err(|e| {
                anyhow!(
                    "Error:Failed to decode response as valid ept toolchain data from '{ps_url}' : {e}"
                )
            })?;
        let value = Value::try_from(res)?;
        let text = to_string_pretty(&value)?;
        write(p.join(MIRROR_FILE_EPT_TOOLCHAIN), text)?;
    }

    // [defer] 写 hello.toml
    ensure_dir_exist(&p)?;
    let value = Value::try_from(res)?;
    let text = to_string_pretty(&value)?;
    write(p.join(MIRROR_FILE_HELLO), text)?;

    Ok(mirror_name)
}

pub fn mirror_update(cfg: &crate::types::context::RuntimeContext, name: &str) -> Result<String> {
    // 读取 meta 文件
    let (meta, _) = read_local_mirror_hello(cfg, name)?;
    // 筛选出 hello 服务
    let (hello_path, _) = filter_service_from_meta(&meta, ServiceKeys::Hello)?;
    // 调用 add
    mirror_add(cfg, &hello_path, Some(name.to_string()))
}

pub fn mirror_list(cfg: &crate::types::context::RuntimeContext) -> Result<Vec<MirrorInfo>> {
    let p = get_path_mirror(cfg)?;
    let mut res = Vec::new();
    for name in read_sub_dir(&p)? {
        let file_path = p.join(&name).join(MIRROR_FILE_HELLO);
        let time = metadata(file_path)?.modified()?;
        let (meta, _) = read_local_mirror_hello(cfg, &name)?;

        res.push(MirrorInfo {
            name,
            updated_at: time,
            root_url: meta.root_url,
        });
    }
    Ok(res)
}

pub fn mirror_update_all(cfg: &crate::types::context::RuntimeContext) -> Result<Vec<String>> {
    let p = get_path_mirror(cfg)?;
    let mut names = Vec::new();
    for name in read_sub_dir(p)? {
        let n = mirror_update(cfg, &name)?;
        names.push(n);
    }
    Ok(names)
}

// 根据 config 中的超时配置自动判断是否需要更新镜像
pub fn auto_mirror_update_all(cfg: &crate::types::context::RuntimeContext) -> Result<bool> {
    // 读取配置
    let duration_cfg = parse_duration(&cfg.cfg.online.mirror_update_interval).map_err(|e| anyhow!("Error:Failed to parse config field 'online.mirror_update_interval' as valid time span : '{e}', e.g. '5d' '14m54s'"))?;
    let now = SystemTime::now();
    log!(
        "Debug:Mirror update interval : '{i}'",
        i = &cfg.cfg.online.mirror_update_interval
    );

    // 列出镜像源，如果其中有一个过期就更新全部
    let ls = mirror_list(cfg)?;
    let res = ls
        .into_iter()
        .find(|mirror_info| now.duration_since(mirror_info.updated_at).unwrap() > duration_cfg);
    if res.is_some() {
        log!("Info:Automatically updating mirror index...");
        mirror_update_all(cfg)?;
        log_ok_last!("Info:Automatically updating mirror index...");
        Ok(true)
    } else {
        log!("Debug:No outdated mirror");
        Ok(false)
    }
}

pub fn mirror_remove(cfg: &crate::types::context::RuntimeContext, name: &str) -> Result<()> {
    // 获取目录路径
    let (_, p) = read_local_mirror_hello(cfg, name)?;
    // 移除目录
    try_recycle(p)
}

#[test]
fn test_mirror() {
    use crate::utils::flags::{set_flag, Flag};
    set_flag(Flag::Debug, true);
    use crate::entrances::search;
    use crate::utils::test::_default_test_cfg;
    use crate::utils::test::_run_mirror_mock_server;
    use std::fs::{remove_dir_all, rename};
    use std::thread::sleep;
    use std::time::Duration;

    let cfg = &_default_test_cfg();

    // 备份原有的镜像文件夹
    let origin_p = get_path_mirror(cfg).unwrap();
    let bak_p = origin_p.parent().unwrap().join("mirror_bak");
    let has_origin_mirror = origin_p.exists();
    if has_origin_mirror {
        if bak_p.exists() {
            remove_dir_all(&origin_p).unwrap();
        } else {
            rename(&origin_p, &bak_p).unwrap();
        }
    }
    assert!(mirror_list(cfg).unwrap().is_empty());

    // 此时搜不到内容
    assert!(search(cfg, "vscode", false).is_err());

    // 启动 mock 服务器
    let mock_url = _run_mirror_mock_server();

    // 测试添加
    mirror_add(cfg, &mock_url, None).unwrap();

    // 测试列出
    let ls = mirror_list(cfg).unwrap();
    assert_eq!(ls.len(), 1);
    let mirror_info = ls.first().unwrap();
    let old_update_time = mirror_info.updated_at;
    assert_eq!(mirror_info.name, "mock-server");

    // 测试搜索
    let expected_res = vec![crate::types::mirror::SearchResult {
        name: "VSCode".to_string(),
        scope: "Microsoft".to_string(),
        version: "1.75.4.2".to_string(),
        description: "Visual Studio Code".to_string(),
        from_mirror: Some("mock-server".to_string()),
    }];
    // 精准名称
    let search_res = search(cfg, "vscode", false).unwrap();
    assert_eq!(search_res, expected_res);
    // 大小写不敏感别名
    let search_res = search(cfg, "Code", false).unwrap();
    assert_eq!(search_res, expected_res);
    // 大小写不敏感名称
    let search_res = search(cfg, "FIREFOx", false).unwrap();
    assert_eq!(
        search_res,
        vec![crate::types::mirror::SearchResult {
            name: "Firefox".to_string(),
            scope: "PortableApps".to_string(),
            version: "127.0.0.1".to_string(),
            description: "".to_string(),
            from_mirror: Some("mock-server".to_string()),
        }]
    );
    // Tag 搜索
    let search_res = search(cfg, "ELECTRON", false).unwrap();
    assert_eq!(search_res, expected_res);
    // 二进制搜索
    let search_res = search(cfg, "ntpd", false).unwrap();
    assert_eq!(
        search_res,
        vec![crate::types::mirror::SearchResult {
            name: "Notepad".to_string(),
            scope: "Microsoft".to_string(),
            version: "22.1.0.0".to_string(),
            description: "Notepad".to_string(),
            from_mirror: Some("mock-server".to_string()),
        }]
    );
    // 正则名称
    let search_res = search(cfg, r"vs\w+", true).unwrap();
    assert_eq!(search_res, expected_res);
    assert!(search(cfg, "microsoft", false).is_err());

    // 测试更新
    sleep(Duration::from_micros(100));
    mirror_update(cfg, "mock-server").unwrap();
    let ls = mirror_list(cfg).unwrap();
    let mirror_info = ls.first().unwrap();
    assert!(
        mirror_info
            .updated_at
            .duration_since(old_update_time)
            .unwrap()
            > Duration::from_micros(50)
    );

    // 测试移除
    mirror_remove(cfg, "mock-server").unwrap();
    assert!(mirror_list(cfg).unwrap().is_empty());

    // 还原原有的镜像文件夹
    if has_origin_mirror {
        remove_dir_all(&origin_p).unwrap();
        rename(&bak_p, &origin_p).unwrap();
    }
}
#[test]
fn test_auto_mirror_update_all() {
    use crate::utils::test::_default_test_cfg;
    use crate::utils::test::_run_mirror_mock_server;
    use std::fs::{remove_dir_all, rename};
    use std::thread::sleep;
    use std::time::Duration;

    let cfg = _default_test_cfg();

    // 备份原有的镜像文件夹
    let origin_p = get_path_mirror(&cfg).unwrap();
    let bak_p = origin_p.parent().unwrap().join("mirror_bak");
    let has_origin_mirror = origin_p.exists();
    if has_origin_mirror {
        if bak_p.exists() {
            remove_dir_all(&origin_p).unwrap();
        } else {
            rename(&origin_p, &bak_p).unwrap();
        }
    }
    assert!(mirror_list(&cfg).unwrap().is_empty());

    // 启动 mock 服务器
    let mock_url = _run_mirror_mock_server();

    mirror_add(&cfg, &mock_url, None).unwrap();

    // 使用默认的 1d 过期配置，不会导致更新
    assert!(!auto_mirror_update_all(&cfg).unwrap());

    // 创建一个短过期配置，等 2s 后会导致更新
    let mut short_cfg = cfg.clone();
    short_cfg.cfg.online.mirror_update_interval = "1s".to_string();
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
//     mirror_add("http://localhost:3000/", None).unwrap();
// }

// #[test]
// fn test_mirror_update() {
//     mirror_update("official").unwrap();
// }

// #[test]
// fn test_mirror_list() {
//     let res = mirror_list().unwrap();
//     println!("{res:#?}")
// }

// #[test]
// fn test_mirror_remove() {
//     mirror_remove("official").unwrap();
// }
