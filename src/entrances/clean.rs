use anyhow::Result;
use std::{
    collections::HashSet,
    fs::{read_dir, remove_dir_all, remove_file},
    path::Path,
};
use trash;

use crate::{
    log, log_ok_last, p2s,
    parsers::parse_workflow,
    types::{Step, WorkflowNode},
    utils::{ask_yn, get_bare_apps, get_path_apps, get_path_bin, parse_bare_temp},
};

use super::info_local;

fn get_valid_entrances(setup: Vec<WorkflowNode>) -> Vec<String> {
    setup
        .into_iter()
        .filter_map(|node| {
            if let Step::StepPath(step) = node.body {
                // TODO:支持 alias
                let stem = p2s!(Path::new(&step.record).file_stem().unwrap());
                Some(stem + ".cmd")
            } else {
                None
            }
        })
        .collect()
}

pub fn clean() -> Result<()> {
    let mut clean_list = Vec::new();
    let mut valid_entrances = HashSet::new();

    // temp 目录，直接删除
    let temp_path = parse_bare_temp()?;
    if temp_path.exists() {
        clean_list.push(temp_path);
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

                        // 读取工作流
                        let setup_path = p2s!(get_path_apps(&scope_name, &app_name, false)?
                            .join(".nep_context/workflows/setup.toml"));
                        let setup = parse_workflow(&setup_path)?;

                        // 解析有效的入口名称
                        get_valid_entrances(setup).into_iter().for_each(|name| {
                            valid_entrances.insert(name);
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
    for entry in read_dir(get_path_bin()?)? {
        let entry = entry?;
        let name = p2s!(entry.file_name());
        if !valid_entrances.contains(&name) {
            clean_list.push(entry.path());
        }
    }

    // 尝试移动到回收站
    if clean_list.len() > 0 {
        log!("Info:Trash list :");
        println!("{:#?}", &clean_list);
        let tip = format!("Info:Moving {} trashes to recycle bin...", clean_list.len());
        log!("{}", tip);
        if let Err(e) = trash::delete_all(clean_list.clone()) {
            log!(
                "Warning:Failed to move some files to recycle bin : {}, force delete all? (y/n)",
                e.to_string()
            );
            if ask_yn() {
                clean_list.into_iter().for_each(|p| {
                    let name = p2s!(p);
                    let res = if p.is_dir() {
                        remove_dir_all(p)
                    } else {
                        remove_file(p)
                    };
                    if let Err(e) = res {
                        log!("Warning:Failed to delete '{}' : {}", name, e.to_string());
                    }
                })
            }
        } else {
            log_ok_last!("{}", tip);
        }
    } else {
        log!("Info:No trash found");
    }

    Ok(())
}

#[test]
fn test_clean() {
    // envmnt::set("DEBUG", "true");
    clean().unwrap();
}
