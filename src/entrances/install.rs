use anyhow::{anyhow, Result};
use std::fs::{create_dir_all, rename};
use std::path::Path;

use super::utils::clean_temp;
use super::{
    info_local,
    utils::{installed_validator, unpack_nep},
};
use crate::utils::get_path_apps;
use crate::{
    executor::workflow_executor,
    parsers::parse_workflow,
    utils::{log, log_ok_last},
};

pub fn install_using_package(source_file: String, verify_signature: bool) -> Result<()> {
    log(format!(
        "Info:Preparing to install with package '{}'",
        &source_file
    ));

    // 解包
    let (temp_dir_inner_path, package_struct) = unpack_nep(source_file.clone(), verify_signature)?;

    // 读入安装工作流
    log(format!("Info:Resolving package..."));
    let setup_file_path = temp_dir_inner_path.join("workflows/setup.toml");
    let setup_workflow = parse_workflow(setup_file_path.to_string_lossy().to_string())?;
    log_ok_last(format!("Info:Resolving package..."));

    // 创建 apps 文件夹
    log(format!("Info:Deploying files..."));
    let apps_path = get_path_apps();
    if !apps_path.exists() {
        create_dir_all(apps_path)?;
    }

    // 检查对应包名有没有被安装过
    let try_get_info_res = info_local(package_struct.package.name.clone());
    if try_get_info_res.is_ok() {
        // TODO:支持升级后此处进行升级
        let (_, diff) = try_get_info_res.unwrap();
        return Err(anyhow!(
            "Error:Package '{}' has been installed({}), use 'ept update \"{}\"' instead",
            &package_struct.package.name,
            diff.version,
            &source_file
        ));
    }

    // 解析最终安装位置
    let into_dir = get_path_apps()
        .join(&package_struct.package.name)
        .to_string_lossy()
        .to_string();

    // 移动程序至 apps 目录
    let app_path = temp_dir_inner_path.join(&package_struct.package.name);
    if !app_path.exists() {
        return Err(anyhow!(
            "Error:App folder not found : {}",
            app_path.to_string_lossy()
        ));
    }
    rename(app_path, into_dir.clone())?;
    log_ok_last(format!("Info:Deploying files..."));

    // 执行安装工作流
    log(format!("Info:Running setup workflow..."));
    workflow_executor(setup_workflow, into_dir.clone())?;
    log_ok_last(format!("Info:Running setup workflow..."));

    // 保存上下文
    let ctx_path = Path::new(&into_dir).join(".nep_context");
    rename(temp_dir_inner_path, ctx_path)?;

    // 检查安装是否完整
    log(format!("Info:Validating setup..."));
    installed_validator(into_dir)?;
    log_ok_last(format!("Info:Validating setup..."));

    // 清理临时文件夹
    clean_temp(source_file)?;

    Ok(())
}

#[test]
fn test_install() {
    // envmnt::set("OFFLINE", "true");
    install_using_package(
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode_1.75.0.0_Cno\VSCode_1.75.0.0_Cno.tar"
            .to_string(),
        true,
    )
    .unwrap();
}
