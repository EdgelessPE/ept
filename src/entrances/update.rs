use std::{
    fs::{remove_dir_all, rename},
    str::FromStr,
};

use anyhow::{anyhow, Result};

use crate::{
    executor::workflow_executor,
    parsers::{parse_author, parse_workflow},
    types::ExSemVer,
    utils::{ask_yn, get_path_apps, log, log_ok_last},
};

use super::{
    info_local, install_using_package, uninstall,
    utils::{clean_temp, installed_validator, unpack_nep},
};

fn same_authors(a: &Vec<String>, b: &Vec<String>) -> bool {
    let ai = a
        .into_iter()
        .map(|raw| parse_author(raw.to_owned()).unwrap());
    let bi = b
        .into_iter()
        .map(|raw| parse_author(raw.to_owned()).unwrap());

    ai.eq(bi)
}

pub fn update_using_package(source_file: String, verify_signature: bool) -> Result<()> {
    log(format!(
        "Info:Preparing to update with package '{}'",
        &source_file
    ));

    // 解包
    let (temp_dir_inner_path, fresh_package) = unpack_nep(source_file.clone(), verify_signature)?;

    // 确认包是否已安装
    log(format!("Info:Resolving package..."));
    let (local_package, local_diff) =
        info_local(fresh_package.package.name.clone()).map_err(|_| {
            anyhow!(
                "Error:Package '{}' hasn't been installed, use 'ept install \"{}\"' instead",
                &fresh_package.package.name,
                &source_file
            )
        })?;

    // 确认是否允许升级
    let local_version = ExSemVer::from_str(&local_diff.version)?;
    let fresh_version = ExSemVer::from_str(&fresh_package.package.version)?;
    if local_version >= fresh_version {
        return Err(anyhow!("Error:Package '{}' has been up to date ({}), can't update to the version of given package ({})",&fresh_package.package.name,local_version,fresh_version));
    }

    // 确认作者是否一致
    if !same_authors(
        &local_package.package.authors,
        &fresh_package.package.authors,
    ) {
        // 需要卸载然后重新安装
        log(format!("Warning:The given package is not the same as the author of the installed package (local:{}, given:{}), uninstall the installed package first? (y/n)",local_package.package.authors.join(","),fresh_package.package.authors.join(",")));
        if !ask_yn() {
            return Err(anyhow!("Error:Update canceled by user"));
        }
        // 卸载
        uninstall(local_package.package.name.clone())?;
        // 安装
        return install_using_package(source_file, verify_signature);
    }

    let located = get_path_apps().join(&local_package.package.name);
    log_ok_last(format!("Info:Resolving package..."));

    // 执行旧的 remove 工作流
    let remove_path = located
        .join(".nep_context")
        .join("workflows")
        .join("remove.toml");
    let run_remove = if remove_path.exists() {
        log(format!("Info:Running remove workflow..."));
        let remove_workflow = parse_workflow(remove_path.to_string_lossy().to_string())?;
        workflow_executor(remove_workflow, located.to_string_lossy().to_string())?;
        log_ok_last(format!("Info:Running remove workflow..."));
        true
    } else {
        false
    };

    // 移除旧的 app 目录
    // TODO:尽可能提前检查占用，避免无法删除
    log(format!("Info:Removing old package..."));
    remove_dir_all(&located)?;
    log_ok_last(format!("Info:Removing old package..."));

    // 移动程序至 apps 目录
    log(format!("Info:Deploying files..."));
    rename(
        temp_dir_inner_path.join(&fresh_package.package.name),
        &located,
    )?;
    log_ok_last(format!("Info:Deploying files..."));

    // 检查有无 update 工作流
    let update_path = temp_dir_inner_path.join("workflows").join("update.toml");
    if update_path.exists() {
        // 执行 update 工作流
        log(format!("Info:Running update workflow..."));
        let update_workflow = parse_workflow(update_path.to_string_lossy().to_string())?;
        workflow_executor(update_workflow, located.to_string_lossy().to_string())?;
        log_ok_last(format!("Info:Running update workflow..."));
    } else {
        if run_remove {
            // 没有升级但是跑了一遍卸载，需要重新跑一遍 setup
            log(format!("Info:Running setup workflow..."));
            let setup_workflow = parse_workflow(
                update_path
                    .with_file_name("setup.toml")
                    .to_string_lossy()
                    .to_string(),
            )?;
            workflow_executor(setup_workflow, located.to_string_lossy().to_string())?;
            log_ok_last(format!("Info:Running setup workflow..."));
        }
    }

    // 保存上下文
    let ctx_path = located.join(".nep_context");
    rename(temp_dir_inner_path, ctx_path)?;

    // 检查更新是否完整
    log(format!("Info:Validating update..."));
    installed_validator(located.to_string_lossy().to_string())?;
    log_ok_last(format!("Info:Validating update..."));

    // 清理临时文件夹
    clean_temp(source_file)?;

    Ok(())
}

#[test]
fn test_update_using_package() {
    update_using_package("./VSCode_1.75.0.0_Cno.nep".to_string(), true).unwrap();
}
