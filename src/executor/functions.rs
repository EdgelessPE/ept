use std::sync::{Arc, Mutex};

use crate::{
    entrances::info,
    types::{
        permissions::{Permission, PermissionLevel},
        verifiable::Verifiable,
    },
    utils::{is_alive_with_name, parse_relative_path_with_located},
};
use anyhow::{anyhow, Result};
use eval::Expr;
use regex::Regex;

use crate::types::{permissions::Generalizable, workflow::WorkflowHeader};

use super::{judge_perm_level, values_validator_path};

lazy_static! {
    static ref RESOURCE_REGEX: Regex = Regex::new(r"^[^/]+/[^/]+$").unwrap();
}

fn get_arg(val: Vec<eval::Value>) -> std::result::Result<String, eval::Error> {
    if val.len() > 1 {
        return Err(eval::Error::ArgumentsGreater(1));
    }
    if val.len() == 0 {
        return Err(eval::Error::ArgumentsLess(1));
    }
    let arg = &val[0];
    if !arg.is_string() {
        return Err(eval::Error::Custom("Argument is not a string".to_string()));
    }

    let mut arg = arg.to_string();
    if arg.starts_with("\"") && arg.ends_with("\"") {
        arg = arg[1..arg.len() - 1].to_string();
    }
    Ok(arg)
}

impl WorkflowHeader {
    /// 使用虚拟的函数定义捕获函数运行信息，返回（函数名，参数，参数是否为路径，所属表达式）
    fn capture_function_info(&self) -> Result<Vec<(String, String, bool, String)>> {
        // 定义已知函数信息，（函数名，入参是否为路径）
        let info_arr = vec![
            ("Exist", true),
            ("IsDirectory", true),
            ("IsAlive", false),
            ("IsInstalled", false),
        ];

        // 收集全部的条件语句
        let mut conditions = Vec::new();
        if let Some(cond) = &self.c_if {
            conditions.push(cond);
        }

        // 迭代所有条件语句
        let res = Arc::new(Mutex::new(Vec::new()));
        for cond in conditions {
            // 初始化表达式
            let mut expr = Expr::new(cond);

            // 迭代函数信息，收集
            for (name, is_fs) in info_arr.clone() {
                let r = res.clone();
                let c = cond.clone();
                expr = expr.function(name, move |val| {
                    let arg = get_arg(val)?;
                    let mut r = r.lock().unwrap();
                    r.push((name.to_string(), arg, is_fs, c.to_owned()));

                    Ok(eval::Value::Bool(true))
                });
            }

            // 执行
            expr.exec().map_err(|e| {
                anyhow!(
                    "Error:Failed to execute expression '{cond}' : {err}",
                    err = e.to_string()
                )
            })?;
        }

        let res = res.lock().unwrap();
        Ok(res.clone())
    }
}

pub fn functions_decorator(expr: Expr, located: &String) -> Expr {
    let l = located.to_owned();
    let expr = expr.function("Exist", move |val| {
        let arg = get_arg(val)?;
        let p = parse_relative_path_with_located(&arg, &l);

        Ok(eval::Value::Bool(p.exists()))
    });

    let l = located.to_owned();
    let expr = expr.function("IsDirectory", move |val| {
        let arg = get_arg(val)?;
        let p = parse_relative_path_with_located(&arg, &l);

        Ok(eval::Value::Bool(p.is_dir()))
    });

    let expr = expr.function("IsAlive", move |val| {
        let arg = get_arg(val)?;

        Ok(eval::Value::Bool(is_alive_with_name(&arg)))
    });

    let expr = expr.function("IsInstalled", move |val| {
        let arg = get_arg(val)?;
        let sp: Vec<&str> = arg.split("/").collect();
        if sp.len() != 2 {
            return Err(eval::Error::Custom(format!("Invalid argument '{arg}'")));
        }
        let info = info(Some(sp[0].to_string()), &sp[1].to_string());

        Ok(eval::Value::Bool(info.is_ok()))
    });

    expr
}

impl Generalizable for WorkflowHeader {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        // 捕获函数执行信息
        let func_info = self.capture_function_info()?;

        // 匹配生成权限信息
        let mut permissions = Vec::new();
        for (name, arg, _, expr) in func_info {
            match name.as_str() {
                "Exist" => {
                    permissions.push(Permission {
                        key: "fs_read".to_string(),
                        level: judge_perm_level(&arg)
                            .map_err(|e| eval::Error::Custom(e.to_string()))?,
                        targets: vec![arg],
                    });
                }
                "IsDirectory" => {
                    permissions.push(Permission {
                        key: "fs_read".to_string(),
                        level: judge_perm_level(&arg)
                            .map_err(|e| eval::Error::Custom(e.to_string()))?,
                        targets: vec![arg],
                    });
                }
                "IsAlive" => {
                    permissions.push(Permission {
                        key: "process_query".to_string(),
                        level: PermissionLevel::Normal,
                        targets: vec![arg],
                    });
                }
                "IsInstalled" => {
                    permissions.push(Permission {
                        key: "nep_installed".to_string(),
                        level: PermissionLevel::Normal,
                        targets: vec![arg],
                    });
                }
                _ => {
                    // 理论上此处是不可到达的，因为会在 eval 执行的时候报错
                    return Err(anyhow!(
                        "Error:Unknown function '{name}' in expression '{expr}'"
                    ));
                }
            }
        }

        Ok(permissions)
    }
}

impl Verifiable for WorkflowHeader {
    fn verify_self(&self, _: &String) -> Result<()> {
        // 捕获函数执行信息
        let func_info = self.capture_function_info()?;

        // 匹配函数名称进行校验
        for (name, arg, need_path_check, expr) in func_info {
            // 特定函数的预校验
            match name.as_str() {
                "IsAlive" => {
                    if !arg.to_ascii_lowercase().ends_with(".exe") {
                        return Err(anyhow!(
                            "Error:Argument of 'IsAlive' should ends with '.exe', got '{arg}'"
                        ));
                    }
                }
                "IsInstalled" => {
                    if !RESOURCE_REGEX.is_match(&arg) {
                        return Err(anyhow!("Error:Argument of 'IsAlive' should match pattern 'SCOPE/NAME' (e.g. Microsoft/VSCode)"));
                    }
                }
                _ => {}
            }
            // 路径参数校验
            if need_path_check {
                values_validator_path(&arg).map_err(|e| {
                    anyhow!(
                        "Error:Failed to validate path argument in expression '{expr}' : {err}",
                        err = e.to_string()
                    )
                })?;
            }
        }

        Ok(())
    }
}

#[test]
fn test_header_perm() {
    let flow=WorkflowHeader{
        name: "Name".to_string(),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };
    let res = flow.generalize_permissions().unwrap();
    assert_eq!(
        res,
        vec![
            Permission {
                key: "fs_read".to_string(),
                level: PermissionLevel::Normal,
                targets: vec!["./mc/vsc.exe".to_string(),],
            },
            Permission {
                key: "fs_read".to_string(),
                level: PermissionLevel::Sensitive,
                targets: vec!["${SystemDrive}/Windows".to_string(),],
            },
            Permission {
                key: "fs_read".to_string(),
                level: PermissionLevel::Sensitive,
                targets: vec!["${AppData}/Roaming/Edgeless/ept".to_string(),],
            },
        ]
    )
}

#[test]
fn test_header_valid() {
    let flow=WorkflowHeader{
        name: "Name".to_string(),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };

    flow.verify_self(&String::from("./apps/VSCode")).unwrap();
}
