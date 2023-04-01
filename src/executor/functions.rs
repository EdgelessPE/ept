use crate::types::permissions::{Permission, PermissionLevel};
use anyhow::Result;
use eval::Expr;
use regex::Regex;
use std::path::Path;

use crate::types::{permissions::Generalizable, workflow::WorkflowHeader};

use super::values::{collect_values, match_value_permission};

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
    Ok(arg.to_string())
}

/// 给定内置函数访问的 fs 目标（包含内置变量），需要的权限级别
fn judge_perm_level(fs_target: &String) -> std::result::Result<PermissionLevel, eval::Error> {
    // 收集使用到的内置变量
    let values = collect_values(fs_target).map_err(|e| eval::Error::Custom(e.to_string()))?;
    let mut final_perm = PermissionLevel::Normal;
    for val in values {
        let cur = match_value_permission(&val).map_err(|e| eval::Error::Custom(e.to_string()))?;
        if cur > final_perm {
            final_perm = cur;
        }
    }

    Ok(final_perm)
}

/// 给定条件语句，匹配指定函数使用到的参数
fn match_args(fn_name: String, cond: &String) -> Result<Vec<String>> {
    // 编译正则表达式
    let regex = Regex::new(&(fn_name + r"\(([^)]+)\)"))?;
    // 匹配结果
    let mut res = Vec::new();
    for cap in regex.captures_iter(cond) {
        res.push(cap[1].to_string())
    }

    Ok(res)
}

pub fn functions_decorator(expr: Expr) -> Expr {
    expr.function("Exist", |val| {
        let arg = get_arg(val)?;
        let p = Path::new(&arg);

        Ok(eval::Value::Bool(p.exists()))
    })
    .function("IsDirectory", |val| {
        let arg = get_arg(val)?;
        let p = Path::new(&arg);

        Ok(eval::Value::Bool(p.is_dir()))
    })
}

impl Generalizable for WorkflowHeader {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        // 收集全部的条件语句
        let mut conditions = Vec::new();
        if let Some(cond) = &self.c_if {
            conditions.push(cond);
        }

        let mut permissions = Vec::new();
        for cond in conditions {
            for arg in match_args("Exist".to_string(), cond)? {
                permissions.push(Permission {
                    key: "fs_read".to_string(),
                    level: judge_perm_level(&arg)?,
                    targets: vec![arg],
                });
            }

            for arg in match_args("IsDirectory".to_string(), cond)? {
                permissions.push(Permission {
                    key: "fs_read".to_string(),
                    level: judge_perm_level(&arg)?,
                    targets: vec![arg],
                });
            }
        }

        Ok(permissions)
    }
}

#[test]
fn test_header_perm() {
    let flow=WorkflowHeader{
        name: "Name".to_string(),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };
    println!("{res:#?}", res = flow.generalize_permissions().unwrap());
}
