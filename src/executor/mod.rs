mod execute;
mod link;
mod log;
mod path;
pub use execute::step_execute;
pub use log::step_log;
pub use link::step_link;
pub use path::step_path;

use anyhow::{Result,anyhow};
use eval::{Expr, to_value};
use std::{path::Path};

use crate::{types::Step, utils::log};
#[macro_use]
extern crate lazy_static;

// 配置部分内置变量的值
lazy_static! {
    static ref SYSTEM_DRIVE="C:";
    static ref DEFAULT_LOCATION="./apps";
}

// 执行条件以判断是否成立
fn condition_eval(condition:String,exit_code:i32)->Result<bool>{
    // 解释变量
    let interpreted=condition_val_interpreter(condition,exit_code)?;
    // 执行 eval
    let eval_res=Expr::new(&interpreted)
    .function("Exist", |val|{
        // 参数校验
        if val.len()!=1 {
            return Err(anyhow!("Error:Internal function 'Exist' only accept 1 arg"));
        }
        let str_opt=val[0].as_str();
        if str_opt.is_none(){
            return Err(anyhow!("Error:Internal function 'Exist' should accept a string"));
        }
        let p=Path::new(str_opt.unwrap());
        
        Ok(p.exists())
    })
    .function("IsDirectory", |val|{
        // 参数校验
        if val.len()!=1 {
            return Err(anyhow!("Error:Internal function 'IsDirectory' only accept 1 arg"));
        }
        let str_opt=val[0].as_str();
        if str_opt.is_none(){
            return Err(anyhow!("Error:Internal function 'IsDirectory' should accept a string"));
        }
        let p=Path::new(str_opt.unwrap());
        
        Ok(p.is_dir())
    })
    .exec();

    // 检查执行结果
    if eval_res.is_err() {
        return Err(anyhow!("Error:Can't eval statement '{}' : {}",&interpreted,eval_res.unwrap_err()));
    }
    let result=eval_res.unwrap().as_bool();
    if result.is_none() {
        return Err(anyhow!("Error:Can't eval statement '{}' into bool result",&interpreted));
    }

    Ok(result.unwrap())
}

// 内置变量解释器
fn condition_val_interpreter(condition:String,exit_code:i32)->Result<String>{
    let replace_arr=vec![
        {
            key:"ExitCode",
            value:exit_code.to_string(),
        },
        {
            key:"SystemDrive",
            value:SYSTEM_DRIVE,
        },
        {
            key:"DefaultLocation",
            value:DEFAULT_LOCATION
        }
    ];

    let mut text=condition;
    for node in replace_arr {
        let find_with=format!("$\{{}\}",node.key);
        text=text.replace(&find_with, &format!("\"{}\"",node.value));
    }

    Ok(text)
}

// 执行工作流
pub fn workflow_executor(flow:Vec<WorkflowNode>)->Result<i32>{
    let mut exit_code=0;

    // 遍历流节点
    for flow_node in flow {
        // 解释节点条件，判断是否需要跳过执行
        let c_if=flow_node.header.c_if;
        if c_if.is_some() && !condition_eval(c_if.unwrap(),exit_code)? {
            continue;
        }
        // 匹配步骤类型以调用步骤解释器
        let exec_res=match flow_node.body {
            Step::StepLink(step)=>{
                step_link(step)
            },
            Step::StepExecute(step)=>{
                step_execute(step)
            },
            Step::StepPath(step)=>{
                step_path(step)
            },
            Step::StepLog(step)=>{
                step_log(step)
            },
        };
        // 处理执行结果
        if exec_res.is_err() {
            log(format!("Warning:Workflow step {} failed to execute : {}, check your workflow syntax again",&step_node.header.name,exec_res.unwrap_err()));
            exit_code=1;
        }else{
            exit_code=exec_res.unwrap();
        }
    }

    Ok(exit_code)
}

#[test]
fn test_condition_val_interpreter(){
    assert_eq!(&condition_val_interpreter("${ExitCode}==514".to_string(), 114),"114==514");
    assert_eq!(&condition_val_interpreter("${SystemDrive}==$${DefaultLocation}".to_string(), 0),"\"C:\"==$\"./apps\"");
}