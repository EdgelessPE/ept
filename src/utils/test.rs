use crate::types::matcher::PackageMatcher;

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
