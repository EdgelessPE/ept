use std::sync::{Arc, Mutex};

use crate::{
    types::permissions::{Permission, PermissionLevel},
    utils::parse_relative_path_with_located,
};
use anyhow::Result;
use eval::Expr;

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

    let mut arg = arg.to_string();
    if arg.starts_with("\"") && arg.ends_with("\"") {
        arg = arg[1..arg.len() - 1].to_string();
    }
    Ok(arg)
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

pub fn functions_decorator(expr: Expr, located: &String) -> Expr {
    let l = located.to_owned();
    let expr = expr.function("Exist", move |val| {
        let arg = get_arg(val)?;
        let p = parse_relative_path_with_located(&arg, &l);
        // println!("exist {p:?} : {e}",e=p.exists());

        Ok(eval::Value::Bool(p.exists()))
    });

    let l = located.to_owned();
    let expr = expr.function("IsDirectory", move |val| {
        let arg = get_arg(val)?;
        let p = parse_relative_path_with_located(&arg, &l);
        // println!("is_dir {p:?} : {r}",r=p.exists());

        Ok(eval::Value::Bool(p.is_dir()))
    });

    expr
}

impl Generalizable for WorkflowHeader {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        // 收集全部的条件语句
        let mut conditions = Vec::new();
        if let Some(cond) = &self.c_if {
            conditions.push(cond);
        }

        let permissions = Arc::new(Mutex::new(Vec::new()));
        for cond in conditions {
            // 使用函数收集器收集函数执行参数
            let expr = Expr::new(cond);

            let perms = permissions.clone();
            let expr = expr.function("Exist", move |val| {
                let arg = get_arg(val)?;
                let mut perms = perms.lock().unwrap();
                perms.push(Permission {
                    key: "fs_read".to_string(),
                    level: judge_perm_level(&arg)?,
                    targets: vec![arg],
                });

                Ok(eval::Value::Bool(true))
            });

            let perms = permissions.clone();
            let expr = expr.function("IsDirectory", move |val| {
                let arg = get_arg(val)?;
                let mut perms = perms.lock().unwrap();
                perms.push(Permission {
                    key: "fs_read".to_string(),
                    level: judge_perm_level(&arg)?,
                    targets: vec![arg],
                });

                Ok(eval::Value::Bool(true))
            });

            // 执行
            expr.exec()?;
        }
        let permissions = permissions.lock().unwrap();
        Ok(permissions.clone())
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
