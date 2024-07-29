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
            .json_body(serde_json::json!({
    "tree": {
        "Microsoft": [
            {
                "name": "VSCode",
                "releases": [
                    {
                        "file_name": "VSCode_1.85.1.0_Cno.nep",
                        "version": "1.85.1.0",
                        "size": 94245376,
                        "timestamp": 1704554724
                    }
                ]
            }
        ]
    },
    "timestamp": 1704554724,
    "url_template": "{root_url}/api/redirect?path=/nep/{scope}/{software}/{file_name}".to_string().replace("{root_url}",&root_url)
}));
    });

    root_url
}

pub fn _run_static_file_server() -> (String, std::process::Child) {
    let port = "1919";
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
