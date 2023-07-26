pub fn _ensure_testing_vscode() -> String {
    if crate::entrances::info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_err() {
        let opt = fs_extra::dir::CopyOptions::new().copy_inside(true);
        fs_extra::dir::copy("examples/VSCode", "test/VSCode", &opt).unwrap();
        crate::install_using_package(&"test/VSCode".to_string(), false).unwrap();
    }

    crate::meta(&"VSCode".to_string(), false).unwrap().temp_dir
}

pub fn _ensure_testing_vscode_uninstalled() {
    if crate::entrances::info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_ok() {
        crate::uninstall(&"VSCode".to_string()).unwrap();
    }
}

pub fn _ensure_testing(scope: &str, name: &str) -> String {
    if crate::entrances::info_local(&scope.to_string(), &name.to_string()).is_err() {
        let opt = fs_extra::dir::CopyOptions::new().copy_inside(true);
        fs_extra::dir::copy(format!("examples/{name}"), format!("test/{name}"), &opt).unwrap();
        crate::install_using_package(&format!("test/{name}"), false).unwrap();
    }

    crate::meta(&name.to_string(), false).unwrap().temp_dir
}

pub fn _ensure_testing_uninstalled(scope: &str, name: &str) {
    if crate::entrances::info_local(&scope.to_string(), &name.to_string()).is_ok() {
        crate::uninstall(&name.to_string()).unwrap();
    }
}

pub fn _ensure_clear_test_dir() {
    use std::path::Path;
    if Path::new("test").exists() {
        std::fs::remove_dir_all("test").unwrap();
    }
    std::fs::create_dir_all("test").unwrap();
}
