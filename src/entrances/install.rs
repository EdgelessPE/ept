use anyhow::{anyhow, Result};
use std::fs::{rename};
use std::path::Path;

use super::utils::clean_temp;
use super::{
    info_local,
    utils::{installed_validator, unpack_nep},
};
use crate::{executor::workflow_executor, parsers::parse_workflow, utils::get_path_apps};
use crate::{log, log_ok_last, p2s};

pub fn install_using_package(source_file: String, verify_signature: bool) -> Result<()> {
    log!("Info:Preparing to install with package '{}'", &source_file);

    // 解包
    let (temp_dir_inner_path, package_struct) = unpack_nep(source_file.clone(), verify_signature)?;

    // 读入安装工作流
    log!("Info:Resolving package...");
    let setup_file_path = temp_dir_inner_path.join("workflows/setup.toml");
    let setup_workflow = parse_workflow(p2s!(setup_file_path))?;
    let package = package_struct.package;
    let software = package_struct.software.unwrap();
    log_ok_last!("Info:Resolving package...");

    // 创建 apps 文件夹
    log!("Info:Deploying files...");

    // 检查对应包名有没有被安装过
    let try_get_info_res = info_local(&software.scope, &package.name);
    if try_get_info_res.is_ok() {
        let (_, diff) = try_get_info_res.unwrap();
        return Err(anyhow!(
            "Error:Package '{}' has been installed({}), use 'ept update \"{}\"' instead",
            &package.name,
            diff.version,
            &source_file
        ));
    }

    // 解析最终安装位置
    let into_dir = p2s!(get_path_apps(&software.scope, &package.name)?);

    // 移动程序至 apps 目录
    let app_path = temp_dir_inner_path.join(&package.name);
    if !app_path.exists() {
        return Err(anyhow!("Error:App folder not found : {}", p2s!(app_path)));
    }
    rename(app_path, into_dir.clone())?;
    log_ok_last!("Info:Deploying files...");

    // 执行安装工作流
    log!("Info:Running setup workflow...");
    workflow_executor(setup_workflow, into_dir.clone())?;
    log_ok_last!("Info:Running setup workflow...");

    // 保存上下文
    let ctx_path = Path::new(&into_dir).join(".nep_context");
    rename(temp_dir_inner_path, ctx_path)?;

    // 检查安装是否完整
    log!("Info:Validating setup...");
    installed_validator(into_dir)?;
    log_ok_last!("Info:Validating setup...");

    // 清理临时文件夹
    clean_temp(source_file)?;

    Ok(())
}

#[test]
fn test_install() {
    // envmnt::set("OFFLINE", "true");
    install_using_package(
        r"D:\Download\VSCode_1.75.0.0_Cno.nep"
            .to_string(),
        true,
    )
    .unwrap();
}
