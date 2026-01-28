use std::path::Path;

use anyhow::{anyhow, Result};
use dirs::{data_dir, desktop_dir, home_dir};

use crate::p2s;

use super::ensure_exist;

fn get_data_dir() -> Result<std::path::PathBuf> {
    data_dir().ok_or_else(|| anyhow!("Error:Failed to get data directory"))
}

pub fn env_system_drive() -> Result<String> {
    let data = get_data_dir()?;
    let path_str = p2s!(data);
    path_str
        .get(0..2)
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Error:Invalid data directory path"))
}

pub fn env_appdata() -> Result<String> {
    let data = get_data_dir()?;
    let parent = data
        .parent()
        .ok_or_else(|| anyhow!("Error:Failed to get AppData parent directory"))?;
    Ok(p2s!(parent))
}

pub fn env_home() -> Result<String> {
    home_dir()
        .map(|p| p2s!(p))
        .ok_or_else(|| anyhow!("Error:Failed to get home directory"))
}

pub fn env_program_files_x64() -> Result<String> {
    Ok(env_system_drive()? + "/Program Files")
}

pub fn env_program_files_x86() -> Result<String> {
    Ok(env_system_drive()? + "/Program Files (x86)")
}

pub fn env_desktop() -> Result<String> {
    desktop_dir()
        .map(|p| p2s!(p))
        .ok_or_else(|| anyhow!("Error:Failed to get desktop directory"))
}

pub fn env_public_desktop() -> Result<String> {
    Ok(env_system_drive()? + "/Users/Public/Desktop")
}

pub fn env_start_menu() -> Result<String> {
    let str = env_appdata()? + "/Roaming/Microsoft/Windows/Start Menu/Programs/Nep Apps";
    ensure_exist(Path::new(&str).to_path_buf())?;
    Ok(str)
}

#[test]
fn test_env() {
    // 测试函数能正常执行而不panic
    env_system_drive().unwrap();
    env_appdata().unwrap();
    env_home().unwrap();
    env_program_files_x64().unwrap();
    env_program_files_x86().unwrap();
    env_desktop().unwrap();
    env_start_menu().unwrap();
}
