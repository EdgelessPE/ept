use crate::types::uninstall_reg_entry::UninstallRegEntry;
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey,
};

// 预读所有可能的 uninstall 注册表位置
fn possible_tables() -> Vec<RegKey> {
    // 定义已知的 uninstall 位置
    let possible_uninstall_positions = vec![
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    possible_uninstall_positions
        .into_iter()
        .filter_map(|(predef, sub_key)| {
            let root = RegKey::predef(predef);
            root.open_subkey(sub_key).ok()
        })
        .collect()
}

pub fn get_reg_entry(entry_id: &String) -> UninstallRegEntry {
    for table in possible_tables() {
        // 尝试打开指定 id
        if let Ok(entry) = table.open_subkey(entry_id) {
            // 获取版本
            let version_res: Option<String> = entry.get_value("Version").ok();
            let display_version: Option<String> = entry.get_value("DisplayVersion").ok();
            let version = if display_version.is_some() {
                display_version
            } else {
                version_res
            };

            // 获取卸载指令
            let uninstall_string_hidden: Option<String> =
                entry.get_value("UninstallString_Hidden").ok();
            let quiet_uninstall_string: Option<String> =
                entry.get_value("QuietUninstallString").ok();
            let literal_uninstall_string: Option<String> = entry.get_value("UninstallString").ok();
            let uninstall_string = if uninstall_string_hidden.is_some() {
                uninstall_string_hidden
            } else if quiet_uninstall_string.is_some() {
                quiet_uninstall_string
            } else {
                literal_uninstall_string
            };

            return UninstallRegEntry {
                version,
                uninstall_string,
            };
        }
    }
    UninstallRegEntry {
        version: None,
        uninstall_string: None,
    }
}

#[test]
fn test_get_reg_entry() {
    let res = get_reg_entry(&"Rustup".to_string());
    assert!(
        (res.version.is_some() && res.uninstall_string.is_some())
            || (res.version.is_none() && res.uninstall_string.is_none())
    );
}
