use anyhow::{anyhow, Result};
use std::fs::remove_dir_all;
use std::path::Path;

use super::{
    info_local,
    utils::package::{clean_temp, unpack_nep},
    utils::validator::installed_validator,
};
use crate::utils::move_or_copy;
use crate::{entrances::update_using_package, utils::ask_yn};
use crate::{executor::workflow_executor, parsers::parse_workflow, utils::get_path_apps};
use crate::{log, log_ok_last, p2s};

pub fn install_using_package(source_file: &String, verify_signature: bool) -> Result<()> {
    log!("Info:Preparing to install with package '{source_file}'");

    // 解包
    let (temp_dir_inner_path, package_struct) = unpack_nep(source_file, verify_signature)?;

    // 读入安装工作流
    log!("Info:Resolving package...");
    let setup_file_path = temp_dir_inner_path.join("workflows/setup.toml");
    let setup_workflow = parse_workflow(&p2s!(setup_file_path))?;
    let package = package_struct.package.clone();
    let software = package_struct.software.clone().unwrap();
    log_ok_last!("Info:Resolving package...");

    // 使用绝对路径的 main_program 字段，检查是否已经全局安装过该软件
    if let Some(installed) = &software.main_program {
        let p = Path::new(installed);
        if p.is_absolute() && p.exists() {
            log!(
                "Warning:'{name}' has been installed at '{installed}', continue? (y/n)",
                name = package.name
            );
            if !ask_yn() {
                return Err(anyhow!("Error:Operation canceled by user"));
            }
        }
    }

    log!("Info:Deploying files...");
    // 检查对应包名有没有被安装过
    if let Ok((_, diff)) = info_local(&software.scope, &package.name) {
        log!(
            "Warning:Package '{name}' has been installed({ver}), switch to update entrance",
            name = package.name,
            ver = diff.version,
        );
        return update_using_package(source_file, verify_signature);
    }

    // 解析最终安装位置
    let into_dir = get_path_apps(&software.scope, &package.name, true)?;
    if into_dir.exists() {
        remove_dir_all(into_dir.clone()).map_err(|_| {
            anyhow!(
                "Error:Can't keep target directory '{dir}' clear, manually delete it then try again",
                dir = p2s!(into_dir.as_os_str())
            )
        })?;
    }

    // 移动程序至 apps 目录
    let app_path = temp_dir_inner_path.join(&package.name);
    if !app_path.exists() {
        return Err(anyhow!(
            "Error:App folder not found : {dir}",
            dir = p2s!(app_path)
        ));
    }
    move_or_copy(app_path.clone(), into_dir.clone())?;
    log_ok_last!("Info:Deploying files...");

    // 执行安装工作流
    let into_dir = p2s!(into_dir);
    log!("Info:Running setup workflow...");
    workflow_executor(setup_workflow, into_dir.clone(), package_struct)?;
    log_ok_last!("Info:Running setup workflow...");

    // 保存上下文
    let ctx_path = Path::new(&into_dir).join(".nep_context");
    move_or_copy(temp_dir_inner_path, ctx_path)?;

    // 检查安装是否完整
    log!("Info:Validating setup...");
    installed_validator(&into_dir)?;
    if let Some(installed) = &software.main_program {
        let p = Path::new(installed);
        if !p.exists() {
            return Err(anyhow!("Error:Validating failed : field 'main_program' provided in table 'software' not exist : '{installed}'"));
        }
    }
    log_ok_last!("Info:Validating setup...");

    // 清理临时文件夹
    clean_temp(source_file)?;

    Ok(())
}

#[test]
fn test_install() {
    envmnt::set("DEBUG", "true");
    envmnt::set("CONFIRM", "true");
    crate::utils::test::_ensure_clear_test_dir();

    // 校验路径
    let shortcut_path = dirs::desktop_dir().unwrap().join("Visual Studio Code.lnk");
    let entry1_path = crate::utils::get_path_bin().unwrap().join("Code.cmd");
    let entry2_path = crate::utils::get_path_bin()
        .unwrap()
        .join("Microsoft-Code.cmd");
    let app_path = get_path_apps(&"Microsoft".to_string(), &"VSCode".to_string(), false).unwrap();
    let mp_path = app_path.join("Code.exe");
    let cx_path = app_path.join(".nep_context").join("package.toml");

    use std::fs::remove_file;
    if shortcut_path.exists() {
        remove_file(&shortcut_path).unwrap();
    }
    if entry1_path.exists() {
        remove_file(&entry1_path).unwrap();
    }
    if entry2_path.exists() {
        remove_file(&entry2_path).unwrap();
    }

    // 卸载
    if crate::entrances::info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_ok() {
        crate::uninstall(&"VSCode".to_string()).unwrap();
    }

    // 打包并安装
    crate::pack(
        &"./examples/VSCode".to_string(),
        Some("./test/VSCode_1.75.0.0_Cno.nep".to_string()),
        true,
    )
    .unwrap();
    install_using_package(&"./test/VSCode_1.75.0.0_Cno.nep".to_string(), true).unwrap();

    assert!(shortcut_path.exists());
    assert!(entry1_path.exists() || entry2_path.exists());
    assert!(mp_path.exists());
    assert!(cx_path.exists());

    // 重复安装，会被要求使用升级，但是会由于同版本导致升级失败
    assert!(install_using_package(&"./test/VSCode_1.75.0.0_Cno.nep".to_string(), true).is_err());

    crate::uninstall(&"VSCode".to_string()).unwrap();

    assert!(!shortcut_path.exists());
    assert!(!entry1_path.exists() || entry2_path.exists());
    assert!(!mp_path.exists());
    assert!(!cx_path.exists());
}
