use std::path::PathBuf;

use crate::types::matcher::PackageMatcher;
use anyhow::anyhow;
use httpmock::prelude::*;
use which::which;

pub fn _ensure_testing_vscode() -> String {
    if crate::entrances::info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_err() {
        crate::utils::fs::copy_dir("examples/VSCode", "test/VSCode").unwrap();
        crate::install_using_package(&"test/VSCode".to_string(), false).unwrap();
    }

    crate::meta(
        crate::types::matcher::PackageInputEnum::PackageMatcher(PackageMatcher {
            name: "VSCode".to_string(),
            scope: None,
            mirror: None,
            version_req: None,
        }),
        false,
    )
    .unwrap()
    .temp_dir
}

pub fn _ensure_testing_vscode_uninstalled() {
    if crate::entrances::info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_ok() {
        crate::uninstall(Some("Microsoft".to_string()), &"VSCode".to_string()).unwrap();
    }
}

pub fn _ensure_testing(scope: &str, name: &str) -> String {
    if crate::entrances::info_local(&scope.to_string(), &name.to_string()).is_err() {
        crate::utils::fs::copy_dir(format!("examples/{name}"), format!("test/{name}")).unwrap();
        crate::install_using_package(&format!("test/{name}"), false).unwrap();
    }

    crate::meta(
        crate::types::matcher::PackageInputEnum::PackageMatcher(PackageMatcher {
            name: name.to_string(),
            scope: Some(scope.to_string()),
            mirror: None,
            version_req: None,
        }),
        false,
    )
    .unwrap()
    .temp_dir
}

pub fn _ensure_testing_uninstalled(scope: &str, name: &str) {
    let s = scope.to_string();
    if crate::entrances::info_local(&s, &name.to_string()).is_ok() {
        crate::uninstall(Some(s), &name.to_string()).unwrap();
    }
}

pub fn _ensure_clear_test_dir() {
    use std::path::Path;
    if Path::new("test").exists() {
        std::fs::remove_dir_all("test").unwrap();
    }
    std::fs::create_dir_all("test").unwrap();
}

pub fn _run_mirror_mock_server() -> String {
    let mock_server = MockServer::start();
    let root_url = format!("http://{}", mock_server.address());

    mock_server.mock(|when, then| {
        when.method("GET").path("/api/hello");
        then.status(200)
            .header("Content-Type", "application/json")
            .json_body(serde_json::json!({ "name": "mock-server",
    "locale": "zh-CN",
    "description": "Mocked nep mirror",
    "maintainer": "Edgeless",
    "protocol": "1.0.0",
    "root_url": root_url,
    "service": [
        {
            "key": "HELLO",
            "path": "/api/hello"
        },
        {
            "key": "PKG_SOFTWARE",
            "path": "/api/pkg/software"
        },
        {
            "key": "EPT_TOOLCHAIN",
            "path": "/api/ept/toolchain"
        }
    ],
    "property": {
        "deploy_region": "zh-CN",
        "proxy_storage": true,
        "upload_bandwidth": 1000,
        "sync_interval": 0
    } }));
    });

    mock_server.mock(|when, then| {
        when.method("GET")
            .path("/api/pkg/software");
        then.status(200)
            .header("Content-Type", "application/json")
            .json_body(serde_json::json!(
            {
                "tree": {
                    "Microsoft": [
                        {
                            "name": "VSCode",
                            "releases": [
                                {
                                    "file_name": "VSCode_1.75.4.2_Cno.nep",
                                    "version": "1.75.4.2",
                                    "size": 94245376,
                                    "timestamp": 1704554724
                                }
                            ]
                        },
                        {
                            "name": "Notepad",
                            "releases": [
                                {
                                    "file_name": "Notepad_22.1.0.0_Cno.nep",
                                    "version": "22.1.0.0",
                                    "size": 94245376,
                                    "timestamp": 1704554724
                                }
                            ]
                        }
                    ],
                    "PortableApps":[
                        {
                            "name":"Firefox",
                            "releases":[
                                {
                                    "file_name":"Firefox_127.0.0.1_Cno.I.nep",
                                    "version":"127.0.0.1",
                                    "size": 94245376,
                                    "timestamp": 1704554724
                                },
                                {
                                    "file_name":"Firefox_127.0.0.1_Cno.IE.nep",
                                    "version":"127.0.0.1",
                                    "size": 94245376,
                                    "timestamp": 1704554724
                                },
                                {
                                    "file_name":"Firefox_127.0.0.1_Cno.P.nep",
                                    "version":"127.0.0.1",
                                    "size": 94245376,
                                    "timestamp": 1704554724
                                },
                                {
                                    "file_name":"Firefox_127.0.0.1_Cno.PE.nep",
                                    "version":"127.0.0.1",
                                    "size": 94245376,
                                    "timestamp": 1704554724
                                },
                            ]
                        }
                    ]
                },
                "timestamp": 1704554724,
                "url_template": "http://localhost:19191/static/{file_name}?scope={scope}&software={software}".to_string()
            }));
    });

    mock_server.mock(|when, then| {
        when.method("GET").path("/api/ept/toolchain");
        then.status(200)
            .header("Content-Type", "application/json")
            .json_body(serde_json::json!(
            {
                "update": {
                    "wild_gaps": []
                },
                "releases": [
                    {
                        "name": "ept_0.1.1.zip",
                        "version": "0.1.1",
                        "url": "https://registry.edgeless.top/api/redirect?path=/ept_builds/ept_0.1.1.zip",
                        "size": 7244837,
                        "timestamp": 1726559266
                    },
                    {
                        "name": "ept_9999.9999.9999.zip",
                        "version": "9999.9999.9999",
                        "url": "http://localhost:19191/ept_9999.9999.9999.zip",
                        "size": 6805967,
                        "timestamp": 1726563312
                    }
                ]
            }));
    });

    root_url
}

// 将 test 目录在 19191 端口上提供文件下载服务
pub fn _run_static_file_server() -> (String, std::process::Child) {
    let port = "19191";
    // 检查 miniserve 是否已安装
    which("miniserve")
        .map_err(|_| anyhow!("Error:Bin 'miniserve' not installed"))
        .unwrap();
    // 启动 miniserve 服务器
    let handler = std::process::Command::new("cmd")
        .args(["/c", "miniserve", "test", "-p", port])
        .stdout(std::process::Stdio::null())
        .spawn()
        .map_err(|e| anyhow!("Error:Failed to spawn miniserve : {e}"))
        .unwrap();

    (format!("http://localhost:{port}"), handler)
}

pub fn _mount_custom_mirror() -> (bool, PathBuf, PathBuf) {
    use crate::entrances::mirror_add;
    use crate::utils::get_path_mirror;
    use std::fs::{remove_dir_all, rename};

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

    // 启动 mock 服务器
    let mock_url = _run_mirror_mock_server();

    // 添加镜像
    mirror_add(&mock_url, None).unwrap();
    (has_origin_mirror, origin_p, bak_p)
}

pub fn _unmount_custom_mirror(tup: (bool, PathBuf, PathBuf)) {
    use std::fs::{remove_dir_all, rename};
    let (has_origin_mirror, origin_p, bak_p) = tup;
    // 还原原有的镜像文件夹
    if has_origin_mirror {
        remove_dir_all(&origin_p).unwrap();
        rename(bak_p, &origin_p).unwrap();
    }
}

pub fn _modify_package_dir_version(dir: &str, to_version: &str) {
    let dir = dir.to_string();
    let pkg_path = format!("{dir}/package.toml");
    let version = to_version.to_string();
    let mut pkg = crate::parsers::parse_package(&pkg_path, &dir, false).unwrap();
    pkg.package.version = version;
    let text = toml::to_string_pretty(&pkg).unwrap();
    std::fs::write(pkg_path, text).unwrap();
}

pub fn _fork_example_with_version(origin_dir: &str, to_version: &str) -> String {
    let temp_dir = std::path::Path::new("test").join(super::random::random_short_string());
    std::fs::create_dir_all(&temp_dir).unwrap();
    crate::utils::fs::copy_dir(origin_dir, &temp_dir).unwrap();

    let dir = crate::p2s!(temp_dir);
    let pkg_path = format!("{dir}/package.toml");
    let version = to_version.to_string();
    let mut pkg = crate::parsers::parse_package(&pkg_path, &dir, false).unwrap();
    pkg.package.version = version;
    let text = toml::to_string_pretty(&pkg).unwrap();
    std::fs::write(pkg_path, text).unwrap();
    dir
}

// 将当前的镜像数据替换为 _run_mirror_mock_server 中定义的 mock 数据；在结束时使用 _restore_mirror_data 恢复
pub fn _use_mock_mirror_data() -> (bool, PathBuf, PathBuf) {
    use crate::utils::test::_run_mirror_mock_server;
    use std::fs::{remove_dir_all, rename};

    // 备份原有的镜像文件夹
    let origin_p = crate::utils::get_path_mirror().unwrap();
    let bak_p = origin_p.parent().unwrap().join("mirror_bak");
    let has_origin_mirror = origin_p.exists();
    if has_origin_mirror {
        if bak_p.exists() {
            remove_dir_all(&origin_p).unwrap();
        } else {
            rename(&origin_p, &bak_p).unwrap();
        }
    }
    assert!(crate::entrances::mirror_list().unwrap().is_empty());

    // 使用 mock 的镜像数据
    let mock_url = _run_mirror_mock_server();
    crate::entrances::mirror_add(&mock_url, None).unwrap();

    (has_origin_mirror, origin_p, bak_p)
}

// 还原原有的镜像文件夹
pub fn _restore_mirror_data(tup: (bool, PathBuf, PathBuf)) {
    use std::fs::{remove_dir_all, rename};
    let (has_origin_mirror, origin_p, bak_p) = tup;
    if has_origin_mirror {
        remove_dir_all(&origin_p).unwrap();
        rename(&bak_p, &origin_p).unwrap();
    }
}
