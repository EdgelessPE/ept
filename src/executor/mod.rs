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

use crate::{types::{Step, WorkflowNode}, utils::log};

// 配置部分内置变量的值
lazy_static! {
    static ref SYSTEM_DRIVE:String="C:".to_string();
    static ref DEFAULT_LOCATION:String="./apps".to_string();
}

// 执行条件以判断是否成立
fn condition_eval(condition:String,exit_code:i32)->Result<bool>{
    // 执行 eval
    let eval_res=Expr::new(&condition)
    .value("${ExitCode}", exit_code)
    .value("${SystemDrive}", eval::Value::String(SYSTEM_DRIVE.to_string()))
    .value("${DefaultLocation}", DEFAULT_LOCATION.to_string())
    .function("Exist", |val|{
        // 参数校验
        if val.len()>1 {
            return Err(eval::Error::ArgumentsGreater(1));
        }
        if val.len()==0 {
            return Err(eval::Error::ArgumentsLess(1));
        }
        let str_opt=val[0].as_str();
        if str_opt.is_none(){
            return Err(eval::Error::Custom("Error:Internal function 'Exist' should accept a string".to_string()));
        }
        let p=Path::new(str_opt.unwrap());
        
        Ok(eval::Value::Bool(p.exists()))
    })
    .function("IsDirectory", |val|{
        // 参数校验
        if val.len()>1 {
            return Err(eval::Error::ArgumentsGreater(1));
        }
        if val.len()==0 {
            return Err(eval::Error::ArgumentsLess(1));
        }
        let str_opt=val[0].as_str();
        if str_opt.is_none(){
            return Err(eval::Error::Custom("Error:Internal function 'IsDirectory' should accept a string".to_string()));
        }
        let p=Path::new(str_opt.unwrap());
        
        Ok(eval::Value::Bool(p.is_dir()))
    })
    .exec();

    // 检查执行结果
    if eval_res.is_err() {
        return Err(anyhow!("Error:Can't eval statement '{}' : {}",&condition,eval_res.unwrap_err()));
    }
    let result=eval_res.unwrap().as_bool();
    if result.is_none() {
        return Err(anyhow!("Error:Can't eval statement '{}' into bool result",&condition));
    }

    Ok(result.unwrap())
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
            log(format!("Warning:Workflow step {} failed to execute : {}, check your workflow syntax again",&flow_node.header.name,exec_res.unwrap_err()));
            exit_code=1;
        }else{
            exit_code=exec_res.unwrap();
        }
    }

    Ok(exit_code)
}

#[test]
fn test_condition_eval(){
    let r1=condition_eval(String::from("${ExitCode}==114"), 114).unwrap();
    assert!(r1);

    let r2=condition_eval(String::from("${ExitCode}==514"), 114).unwrap();
    assert_eq!(r2,false);

    let r3=condition_eval(String::from("${SystemDrive}==\"C:\""), 0).unwrap();
    assert!(r3);

    let r4=condition_eval(String::from("${DefaultLocation}==\"./apps\""), 0).unwrap();
    assert!(r4);

    let r5=condition_eval(String::from("Exist(\"./src/main.rs\")==IsDirectory(\"./bin\")"), 0).unwrap();
    assert!(r5);

    let r6=condition_eval(String::from("Exist(\"./src/main.ts\")"), 0).unwrap();
    assert_eq!(r6,false);
}