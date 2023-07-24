pub fn _ensure_testing_vscode() {
    if crate::entrances::info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_err() {
        let opt = fs_extra::dir::CopyOptions::new().copy_inside(true);
        fs_extra::dir::copy("examples/VSCode", "test/VSCode", &opt).unwrap();
        crate::install_using_package(&"test/VSCode".to_string(), false).unwrap();
    }
}

pub fn _ensure_testing_vscode_uninstalled() {
    if crate::entrances::info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_ok() {
        crate::uninstall(&"VSCode".to_string()).unwrap();
    }
}
