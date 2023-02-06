use anyhow::{Result,anyhow};
use std::{fs::{read_dir,remove_dir_all}, path::Path};

use crate::{parsers::{parse_package, parse_workflow}, executor::workflow_executor};

use super::validator::installed_validator;

pub fn uninstall(package_name:String)->Result<()>{
    // 解析安装路径
    let app_path=Path::new("./apps").join(&package_name);
    if !app_path.exists(){
        return Err(anyhow!("Error:Can't find app '{}'",&package_name));
    }
    let app_str=app_path.to_string_lossy().to_string();

    // 判断安装路径是否完整
    installed_validator(app_str.clone())?;

    // 读入包信息和卸载工作流
    let global=parse_package(app_path.join(".nep_context/package.toml").to_string_lossy().to_string())?;
    let remove_flow=parse_workflow(app_path.join(".nep_context/workflows/remove.toml").to_string_lossy().to_string())?;

    // 执行卸载工作流
    workflow_executor(remove_flow, app_str.clone())?;

    // 删除 app 目录
    remove_dir_all(&app_str)?;

    Ok(())
}
