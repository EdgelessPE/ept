use anyhow::{anyhow, Result};
use std::{fs::remove_dir_all};

use crate::{
    executor::workflow_executor,
    parsers::parse_workflow,
    utils::{log, log_ok_last, get_path_apps},
};

use super::validator::installed_validator;

pub fn uninstall(package_name: String) -> Result<()> {
    log(format!("Info:Preparing to uninstall '{}'", &package_name));

    // 解析安装路径
    let app_path = get_path_apps().join(&package_name);
    if !app_path.exists() {
        return Err(anyhow!("Error:Can't find package '{}'", &package_name));
    }
    let app_str = app_path.to_string_lossy().to_string();

    // 判断安装路径是否完整
    installed_validator(app_str.clone())?;

    // 读入包信息和卸载工作流
    // let global=parse_package(app_path.join(".nep_context/package.toml").to_string_lossy().to_string())?;
    let remove_flow = parse_workflow(
        app_path
            .join(".nep_context/workflows/remove.toml")
            .to_string_lossy()
            .to_string(),
    )?;

    // 执行卸载工作流
    log(format!("Info:Running remove workflow..."));
    workflow_executor(remove_flow, app_str.clone())?;
    log_ok_last(format!("Info:Running remove workflow..."));

    // 删除 app 目录
    log(format!("Info:Cleaning..."));
    remove_dir_all(&app_str)?;
    log_ok_last(format!("Info:Cleaning..."));

    Ok(())
}

#[test]
fn test_uninstall() {
    uninstall("VSCode".to_string()).unwrap();
}
