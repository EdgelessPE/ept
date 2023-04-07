use std::sync::{Arc, Mutex};

use crate::{
    types::{permissions::{Permission, PermissionLevel}, verifiable::Verifiable},
    utils::parse_relative_path_with_located,
};
use anyhow::{Result, anyhow};
use eval::Expr;

use crate::types::{permissions::Generalizable, workflow::WorkflowHeader};

use super::{values::{collect_values, match_value_permission}, values_validator_path};

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

impl WorkflowHeader {
    /// 使用虚拟的函数定义捕获函数运行信息，返回（函数名，参数，参数是否为路径，所属表达式）
    fn capture_function_info(&self)->Result<Vec<(String,String,bool,String)>>{
        // 定义已知函数信息
        let info_arr=vec![
            ("Exist",true),
            ("IsDirectory",true),
            ("IsAlive",false),
            ("IsInstalled",false),
            ];

        // 收集全部的条件语句
        let mut conditions = Vec::new();
        if let Some(cond) = &self.c_if {
            conditions.push(cond);
        }

        // 迭代所有条件语句
        let res=Arc::new(Mutex::new(Vec::new()));
        for cond in conditions{
            // 初始化表达式
            let mut expr = Expr::new(cond);

            // 迭代函数信息，收集
            for (name,is_fs) in info_arr.clone(){
                let r = res.clone();
                let c=cond.clone();
                expr = expr.function(name, move |val| {
                    let arg = get_arg(val)?;
                    let mut r = r.lock().unwrap();
                    r.push((name.to_string(),arg,is_fs,c.to_owned()));

                    Ok(eval::Value::Bool(true))
                });
            }

            // 执行
            expr.exec().map_err(|e|anyhow!("Error:Failed to execute expression '{cond}' : {err}",err=e.to_string()))?;
        }

        let res=res.lock().unwrap();
        Ok(res.clone())
    }
}

impl Generalizable for WorkflowHeader {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        // 捕获函数执行信息
        let func_info=self.capture_function_info()?;

        // 匹配生成权限信息
        let mut permissions = Vec::new();
        for (name,arg,_,expr) in func_info {
            match name.as_str() {
                "Exist"=>{
                    permissions.push(Permission {
                        key: "fs_read".to_string(),
                        level: judge_perm_level(&arg)?,
                        targets: vec![arg],
                    });
                },
                "IsDirectory"=>{
                    permissions.push(Permission {
                        key: "fs_read".to_string(),
                        level: judge_perm_level(&arg)?,
                        targets: vec![arg],
                    });
                },
                _=>{
                    // 理论上此处是不可到达的，因为会在 eval 执行的时候报错
                    return Err(anyhow!("Error:Unknown function '{name}' in expression '{expr}'"));
                }
            }
        }
        
        Ok(permissions)
    }
}

impl Verifiable for WorkflowHeader {
    fn verify_self(&self,_:&String) -> Result<()> {
        // 捕获函数执行信息
        let func_info=self.capture_function_info()?;
        
        // 匹配函数名称进行校验
        for (name,arg,need_path_check,expr) in func_info {
            // 特定函数的预校验
            match name.as_str() {
                _=>{}
            }
            // 路径参数校验
            if need_path_check{
                values_validator_path(&arg).map_err(|e|anyhow!("Error:Failed to validate path argument in expression '{expr}' : {err}",err=e.to_string()))?;
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
    println!("{res:#?}", res = flow.generalize_permissions().unwrap());
}

#[test]
fn test_header_valid() {
    let flow=WorkflowHeader{
        name: "Name".to_string(),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };
    println!("{res:#?}", res = flow.verify_self(&String::from("D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode")).unwrap());
}
