use anyhow::{anyhow, Result};
use evalexpr::*;
use regex::Regex;
use std::sync::{Arc, Mutex};

use crate::{
    executor::{condition_eval, judge_perm_level, values_validator_path},
    types::permissions::{Permission, PermissionLevel},
};

lazy_static! {
    static ref RESOURCE_REGEX: Regex = Regex::new(r"^[^/]+/[^/]+$").unwrap();
}

pub fn ensure_arg(val: &Value) -> std::result::Result<String, error::EvalexprError> {
    if let Value::String(str) = val {
        Ok(str.to_string())
    } else {
        Err(error::EvalexprError::ExpectedString {
            actual: val.clone(),
        })
    }
}

/// 使用虚拟的函数定义捕获函数运行信息，返回（函数名，参数，参数是否为路径，所属表达式）
fn capture_function_info(conditions: &Vec<String>) -> Result<Vec<(String, String, bool, String)>> {
    // 定义已知函数信息，（函数名，入参是否为路径）
    let info_arr = vec![
        ("Exist", true),
        ("IsDirectory", true),
        ("IsAlive", false),
        ("IsInstalled", false),
    ];

    // 迭代所有条件语句
    let res = Arc::new(Mutex::new(Vec::new()));
    for cond in conditions {
        // 初始化上下文
        let mut context = HashMapContext::new();

        // 迭代函数信息，创建收集闭包
        for (name, is_fs) in info_arr.clone() {
            let r = res.clone();
            let c = cond.clone();
            context
                .set_function(
                    name.to_string(),
                    Function::new(move |val| {
                        let arg = ensure_arg(val)?;
                        let mut r = r.lock().unwrap();
                        r.push((name.to_string(), arg, is_fs, c.to_owned()));

                        Ok(Value::Boolean(true))
                    }),
                )
                .unwrap();
        }

        // 执行
        eval_boolean_with_context(&cond, &context)
            .map_err(|e| anyhow!("Error:Failed to execute expression '{cond}' : {e}"))?;
    }

    let res = res.lock().unwrap();
    Ok(res.clone())
}

pub fn get_permissions_from_conditions(conditions: Vec<String>) -> Result<Vec<Permission>> {
    // 捕获函数执行信息
    let func_info = capture_function_info(&conditions)?;

    // 匹配生成权限信息
    let mut permissions = Vec::new();
    for (name, arg, _, expr) in func_info {
        match name.as_str() {
            "Exist" => {
                permissions.push(Permission {
                    key: "fs_read".to_string(),
                    level: judge_perm_level(&arg)?,
                    targets: vec![arg],
                });
            }
            "IsDirectory" => {
                permissions.push(Permission {
                    key: "fs_read".to_string(),
                    level: judge_perm_level(&arg)?,
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

pub fn verify_conditions(conditions: Vec<String>, located: &String) -> Result<()> {
    // 捕获函数执行信息
    let func_info = capture_function_info(&conditions)?;

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
                anyhow!("Error:Failed to validate path argument in expression '{expr}' : {e}")
            })?;
        }
    }

    // 对条件进行 eval 校验
    for cond in conditions {
        condition_eval(&cond, 0, located)
            .map_err(|e| anyhow!("Error:Failed to validate condition in field 'if' : {e}"))?;
    }

    Ok(())
}
