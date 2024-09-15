use anyhow::{anyhow, Result};
use std::{
    collections::HashSet,
    fs::{read_dir, remove_dir_all, remove_file},
    path::Path,
};

use crate::{
    log, log_ok_last, p2s,
    parsers::parse_workflow,
    types::{steps::Step, workflow::WorkflowNode},
    utils::{
        get_bare_apps, get_path_apps, get_path_bin, get_path_cache, get_path_meta, parse_bare_temp,
        term::ask_yn,
    },
};

use super::info_local;

fn get_valid_entrances(setup: Vec<WorkflowNode>) -> Vec<String> {
    setup
        .into_iter()
        .filter_map(|node| {
            if let Step::StepPath(step) = node.body {
                let alias = step
                    .alias
                    .unwrap_or_else(|| p2s!(Path::new(&step.record).file_stem().unwrap()));
                Some(alias + ".cmd")
            } else {
                None
            }
        })
        .collect()
}

pub fn clean() -> Result<usize> {
    let mut clean_list = Vec::new();
    let mut valid_entrances = HashSet::new();

    // 处理直接删除的目录
    let dirs_to_clean = vec![parse_bare_temp()?, get_path_cache()?, get_path_meta()?];
    for p in dirs_to_clean {
        if p.exists() {
            clean_list.push(p);
        }
    }

    // apps 目录，查找未安装成功的目录
    for scope_entry in read_dir(get_bare_apps()?)? {
        let scope_entry = scope_entry?;
        let scope_path = scope_entry.path();
        let scope_name = p2s!(scope_entry.file_name());

        if scope_path.is_dir() {
            // 当前 scope 目录中的有效应用计数
            let mut valid_apps_count = 0;
            for app_entry in read_dir(scope_path.clone())? {
                let app_entry = app_entry?;
                let app_path = app_entry.path();
                let app_name = p2s!(app_entry.file_name());

                if app_path.is_dir() {
                    // 尝试读取 info
                    let info_res = info_local(&scope_name, &app_name);
                    if info_res.is_ok() {
                        // 有效应用计数
                        valid_apps_count += 1;

                        // 读取 package
                        let (global, _) = info_res.unwrap();

                        // 读取工作流
                        let setup_path = p2s!(get_path_apps(&scope_name, &app_name, false)?
                            .join(".nep_context/workflows/setup.toml"));
                        let setup = parse_workflow(&setup_path)?;

                        // 解析有效的入口名称
                        let scope = global.software.unwrap().scope;
                        get_valid_entrances(setup)
                            .into_iter()
                            .for_each(|entrance_full_name| {
                                // valid_entrances.insert("nep-".to_string()+&entrance_full_name);
                                valid_entrances.insert(scope.clone() + "-" + &entrance_full_name);
                                valid_entrances.insert(entrance_full_name);
                            });
                    } else {
                        // 清理读取失败的
                        clean_list.push(app_path)
                    }
                } else {
                    // 非目录的 scope 内容
                    clean_list.push(app_path);
                }
            }
            // 如果没有有效应用直接删除 scope
            if valid_apps_count == 0 {
                clean_list.push(scope_path);
            }
        } else {
            // 非目录的 apps 内容
            clean_list.push(scope_path);
        }
    }

    // bin 目录，删除名称非法的文件
    // TODO:考虑检查指向的绝对路径是否存在
    for entry in read_dir(get_path_bin()?)? {
        let entry = entry?;
        let name = p2s!(entry.file_name());
        if !valid_entrances.contains(&name) {
            clean_list.push(entry.path());
        }
    }

    // 尝试移动到回收站
    let clean_list_len = clean_list.len();
    if !clean_list.is_empty() {
        log!("Info:Trash list :");
        println!("{clean_list:#?}");
        if !ask_yn(format!("Clean those {clean_list_len} trashes?"), true) {
            return Err(anyhow!("Error:Operation cancelled by user"));
        }
        let tip = format!(
            "Info:Moving {num} trashes to recycle bin...",
            num = clean_list.len()
        );
        log!("{tip}");
        if let Err(e) = trash::delete_all(clean_list.clone()) {
            if ask_yn(
                format!("Failed to move some files to recycle bin : {e}, force delete all?"),
                true,
            ) {
                clean_list.into_iter().for_each(|p| {
                    let name = p2s!(p);
                    let res = if p.is_dir() {
                        remove_dir_all(p)
                    } else {
                        remove_file(p)
                    };
                    if let Err(e) = res {
                        log!("Warning:Failed to delete '{name}' : {e}");
                    }
                })
            }
        } else {
            log_ok_last!("{tip}");
        }
    }

    Ok(clean_list_len)
}

#[test]
fn test_clean() {
    use crate::utils::fs::copy_dir;
    use std::fs::{copy, create_dir_all, write};
    envmnt::set("CONFIRM", "true");

    // 安装 vscode
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    crate::utils::test::_ensure_testing_vscode();
    let bin_path = get_path_bin().unwrap();
    let (vscode_entrance_name, another_entrance_name) =
        if bin_path.join("Microsoft-Code.cmd").exists() {
            ("Microsoft-Code.cmd", "Code.cmd")
        } else {
            ("Code.cmd", "Microsoft-Code.cmd")
        };
    let vscode_entrance_path = bin_path.join(vscode_entrance_name);
    assert!(vscode_entrance_path.exists());

    // 创建几个无效的入口文件
    copy(&vscode_entrance_path, bin_path.join("invalid.cmd")).unwrap();
    copy(&vscode_entrance_path, bin_path.join("Microsoft-Code.bat")).unwrap();
    copy(
        &vscode_entrance_path,
        bin_path.join("Microsoft-VisualStudio.cmd"),
    )
    .unwrap();
    copy(&vscode_entrance_path, bin_path.join(another_entrance_name)).unwrap();

    // 在 apps 目录中添加无效文件
    let apps_path = get_bare_apps().unwrap();
    let fake_scope_foo = apps_path.join("FakeScopeFoo");
    let fake_scope_bar = apps_path.join("FakeScopeBar");
    let fake_scope_foz = apps_path.join("FakeScopeFoz");
    let fake_bar_software = fake_scope_bar.join("Dism++");
    create_dir_all(&fake_scope_foo).unwrap();
    create_dir_all(&fake_scope_bar).unwrap();
    create_dir_all(&fake_scope_foz).unwrap();
    copy_dir("examples/Dism++", &fake_bar_software).unwrap();
    write(apps_path.join("README.md"), "# What can I say").unwrap();
    write(fake_scope_foz.join("README.md"), "# Man!").unwrap();

    // 执行清理
    clean().unwrap();

    // 断言清理结果
    assert!(!bin_path.join("invalid.cmd").exists());
    assert!(!bin_path.join("Microsoft-Code.bat").exists());
    assert!(!bin_path.join("Microsoft-VisualStudio.cmd").exists());
    assert!(bin_path.join(another_entrance_name).exists());
    assert!(!fake_scope_foo.exists());
    assert!(!fake_scope_bar.exists());
    assert!(!fake_scope_foz.exists());
    assert!(!fake_bar_software.exists());
    assert!(!apps_path.join("README.md").exists());
    assert!(!fake_scope_foz.join("README.md").exists());

    // 卸载 vscode
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    // assert!(!bin_path.join(another_entrance_name).exists());
}
