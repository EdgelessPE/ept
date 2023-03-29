use dirs::{data_dir, desktop_dir, home_dir};

use crate::p2s;

pub fn env_system_drive() -> String {
    p2s!(data_dir().unwrap())[0..2].to_string()
}

pub fn env_appdata() -> String {
    p2s!(data_dir().unwrap().parent().unwrap())
}

pub fn env_home() -> String {
    p2s!(home_dir().unwrap())
}

pub fn env_program_files_x64() -> String {
    env_system_drive() + "/Program Files"
}

pub fn env_program_files_x86() -> String {
    env_system_drive() + "/Program Files (x86)"
}

pub fn env_desktop() -> String {
    p2s!(desktop_dir().unwrap())
}

#[test]
fn test_env() {
    assert_eq!(env_system_drive(), "C:".to_string());
    assert_eq!(env_appdata(), "C:/Users/dsyou/AppData".to_string());
    assert_eq!(env_home(), "C:/Users/dsyou".to_string());
    assert_eq!(env_program_files_x64(), "C:/Program Files".to_string());
    assert_eq!(
        env_program_files_x86(),
        "C:/Program Files (x86)".to_string()
    );
    assert_eq!(env_desktop(), "D:/Desktop".to_string());
}
