use anyhow::{anyhow, Result};
use std::fs::remove_dir_all;

use crate::{
    executor::{workflow_executor, workflow_reverse_executor},
    log, log_ok_last,
    parsers::parse_workflow,
    utils::get_path_apps,
};

use super::utils::installed_validator;

pub fn uninstall(package_name: String) -> Result<()> {
    log!("Info:Preparing to uninstall '{}'", &package_name);

    // 解析安装路径
    let app_path = get_path_apps().join(&package_name);
    if !app_path.exists() {
        return Err(anyhow!("Error:Can't find package '{}'", &package_name));
    }
    let app_str = app_path.to_string_lossy().to_string();

    // 判断安装路径是否完整
    installed_validator(app_str.clone())?;

    // 读入卸载工作流
    let remove_flow_path = app_path.join(".nep_context/workflows/remove.toml");
    if remove_flow_path.exists() {
        let remove_flow = parse_workflow(remove_flow_path.to_string_lossy().to_string())?;

        // 执行卸载工作流
        log!("Info:Running remove workflow...");
        workflow_executor(remove_flow, app_str.clone())?;
        log_ok_last!("Info:Running remove workflow...");
    }

    // 读入安装工作流
    let setup_flow_path = app_path.join(".nep_context/workflows/setup.toml");
    let setup_flow = parse_workflow(setup_flow_path.to_string_lossy().to_string())?;

    // 逆向执行安装工作流
    log!("Info:Running reverse setup workflow...");
    workflow_reverse_executor(setup_flow, app_str.clone())?;
    log_ok_last!("Info:Running reverse setup workflow...");

    // 删除 app 目录
    log!("Info:Cleaning...");
    remove_dir_all(&app_str)?;
    log_ok_last!("Info:Cleaning...");

    Ok(())
}

#[test]
fn test_uninstall() {
    uninstall("VSCode".to_string()).unwrap();
}
