use anyhow::{anyhow, Result};
use std::{
    collections::HashSet,
    fs::{read_dir, remove_dir, remove_dir_all},
    thread::sleep,
    time::Duration,
};

use crate::{
    executor::{workflow_executor, workflow_reverse_executor},
    log, log_ok_last, p2s,
    parsers::{parse_package, parse_workflow},
    types::{mixed_fs::MixedFS, workflow::WorkflowNode},
    utils::{ask_yn, find_scope_with_name_locally, get_bare_apps, get_path_apps, kill_with_name},
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

pub fn uninstall(package_name: &String) -> Result<()> {
    log!("Info:Preparing to uninstall '{package_name}'");

    // 解析 scope
    let scope = find_scope_with_name_locally(package_name)?;

    // 解析安装路径
    let app_path = get_path_apps(&scope, package_name, false)?;
    if !app_path.exists() {
        return Err(anyhow!("Error:Can't find package '{package_name}'"));
    }
    let app_str = p2s!(app_path);

    // 判断安装路径是否完整
    installed_validator(&app_str)?;

    // 读入 package.toml
    let global = parse_package(&p2s!(app_path.join(".nep_context/package.toml")), None)?;

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
            let mp_opt = global.software.unwrap().main_program;
            if mp_opt.is_some() {
                hit_list.insert(mp_opt.unwrap());
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
    let mut cx = crate::types::workflow::WorkflowContext::_demo();
    crate::types::steps::StepExecute {
        command: "notepad.exe".to_string(),
        pwd: Some(pwd),
        call_installer: None,
        wait: Some("Abandon".to_string()),
    }
    .run(&mut cx)
    .unwrap();
    crate::types::steps::StepWait {
        timeout: 3000,
        break_if: None,
    }
    .run(&mut cx)
    .unwrap();

    uninstall(&"Notepad".to_string()).unwrap();
}
