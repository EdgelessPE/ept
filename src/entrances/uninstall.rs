use anyhow::{anyhow, Result};
use std::{
    collections::HashSet,
    fs::{read_dir, remove_dir, remove_dir_all},
    path::Path,
    thread::sleep,
    time::Duration,
};

use crate::{
    executor::{workflow_executor, workflow_reverse_executor},
    log, log_ok_last, p2s,
    parsers::{parse_package, parse_workflow},
    types::{
        mixed_fs::MixedFS,
        steps::{StepExecute, TStep},
        workflow::{WorkflowContext, WorkflowNode},
    },
    utils::{
        get_bare_apps, get_path_apps, path::find_scope_with_name, process::kill_with_name,
        reg_entry::get_reg_entry, term::ask_yn,
    },
};

use super::utils::validator::installed_validator;

fn get_manifest(flow: Vec<WorkflowNode>) -> Vec<String> {
    let mut manifest = Vec::new();
    let mut fs = MixedFS::new("".to_string());
    for node in flow {
        manifest.append(&mut node.body.get_manifest(&mut fs));
    }
    manifest
}

pub fn uninstall(scope: Option<String>, package_name: &String) -> Result<()> {
    log!("Info:Preparing to uninstall '{package_name}'");

    // 查找 scope 并使用 scope 更新纠正大小写
    let (scope, package_name) = find_scope_with_name(package_name, scope)?;

    // 解析安装路径
    let app_path = get_path_apps(&scope, &package_name, false)?;
    if !app_path.exists() {
        return Err(anyhow!("Error:Can't find package '{package_name}'"));
    }
    let app_str = p2s!(app_path);

    // 判断安装路径是否完整
    if let Err(e) = installed_validator(&app_str) {
        // 简单的删除目录
        log!("Warning:Incomplete folder found, simply perform a deletion : {e}");
        return remove_dir_all(&app_str).map_err(|e| {
            anyhow!(
                "Warning:Can't clean the directory, please delete '{app_str}' manually later : {e}"
            )
        });
    }

    // 读入 package.toml
    let global = parse_package(
        &p2s!(app_path.join(".nep_context/package.toml")),
        &app_str,
        false,
    )?;
    let software = global.clone().software.unwrap();

    // 如果提供了注册表入口，则先跑卸载命令（独立的工作流上下文）
    if let Some(entry_id) = software.registry_entry {
        let e = get_reg_entry(&entry_id);
        if let Some(uninstall_string) = e.uninstall_string {
            log!("Info:Running uninstaller due to registry entry...");
            let mut cx = WorkflowContext::new(&app_str, global.clone());
            StepExecute {
                command: uninstall_string,
                pwd: None,
                call_installer: Some(true),
                wait: None,
            }
            .run(&mut cx)?;
            cx.finish()?;
            log_ok_last!("Info:Running uninstaller due to registry entry...");
        }
    }

    // 读入卸载工作流
    let remove_flow_path = app_path.join(".nep_context/workflows/remove.toml");
    if remove_flow_path.exists() {
        let remove_flow = parse_workflow(&p2s!(remove_flow_path))?;

        // 执行卸载工作流
        log!("Info:Running remove workflow...");
        workflow_executor(remove_flow, app_str.clone(), global.clone())?;
        log_ok_last!("Info:Running remove workflow...");
    }

    // 读入安装工作流
    let setup_flow_path = app_path.join(".nep_context/workflows/setup.toml");
    let setup_flow = parse_workflow(&p2s!(setup_flow_path))?;

    // 逆向执行安装工作流
    log!("Info:Running reverse setup workflow...");
    workflow_reverse_executor(setup_flow.clone(), app_str.clone(), global.clone())?;
    log_ok_last!("Info:Running reverse setup workflow...");

    // 删除 app 目录
    log!("Info:Cleaning...");
    let try_rm_res = remove_dir_all(&app_str);
    if try_rm_res.is_err() {
        log!("Warning:Can't clean the directory completely, try killing the related processes? (y/n)");
        if ask_yn() {
            // 拿到装箱单，生成基础暗杀名单
            let setup_manifest = get_manifest(setup_flow);
            let mut hit_list: HashSet<String> = HashSet::from_iter(setup_manifest);

            // 加入主程序
            if let Some(mp) = global.software.unwrap().main_program {
                if let Some(file_name) = Path::new(&mp).file_name() {
                    hit_list.insert(file_name.to_string_lossy().to_string());
                }
            }

            // 杀死其中列出的 exe 程序
            for name in hit_list {
                if name.ends_with(".exe") {
                    if kill_with_name(&name) {
                        log!("Warning:Killed process '{name}'");
                    } else {
                        log!("Warning:Failed to kill process '{name}'");
                    }
                }
            }

            // 延时
            sleep(Duration::from_secs(3));

            // 再次尝试删除
            let try_rm_res = remove_dir_all(&app_str);
            if try_rm_res.is_err() {
                log!(
                    "Warning:Can't clean the directory still, please delete '{app_str}' manually later"
                );
            }
        }
    }

    // 删除空的 scope
    let scope_dir = get_bare_apps()?.join(&scope);
    if read_dir(scope_dir.clone())?.next().is_none() {
        let _ = remove_dir(scope_dir);
    }

    log_ok_last!("Info:Cleaning...");

    Ok(())
}

#[test]
fn test_uninstall() {
    // 完整的安装和卸载流程案例位于entrances::install::test_install

    // 这里测试一下需要杀进程的案例
    use crate::types::steps::TStep;
    envmnt::set("CONFIRM", "true");
    let pwd = crate::utils::test::_ensure_testing("Microsoft", "Notepad");
    let mut cx = WorkflowContext::_demo();
    StepExecute {
        command: "notepad.exe".to_string(),
        pwd: Some(pwd.clone()),
        call_installer: None,
        wait: Some("Abandon".to_string()),
    }
    .run(&mut cx)
    .unwrap();
    crate::types::steps::StepWait {
        timeout: 2000,
        break_if: None,
    }
    .run(&mut cx)
    .unwrap();

    uninstall(None, &"Notepad".to_string()).unwrap();
    assert!(!Path::new(&pwd).exists());
}
