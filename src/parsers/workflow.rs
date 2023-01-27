use std::{fs::File, io::Read};
use std::path::Path;
use anyhow::{anyhow,Result};
use toml::{Value};
use regex::Regex;

use crate::types::{Step, WorkflowNode};

fn cmd_converter(origin:String)->Result<String> {
    // 需要增加 c_ 前缀的字段
    let list=["if"];

    // 转换器
    let mut text=origin;
    for cmd in list {
        let reg_str=String::from("(");
        let rep=Regex::new(&(reg_str+cmd+r"\s?=.+)"))?;
        text=rep.replace_all(&text, "c_$1").to_string();
    }
    Ok(text)
}

pub fn parse_workflow(p:String)->Result<Vec<WorkflowNode>> {
    let workflow_path=Path::new(&p);
    if !workflow_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find workflow path : {}",p));
    }

    // 读取文件
    let mut text=String::new();
    File::open(&p)?.read_to_string(&mut text)?;

    // 替换条件命令字段
    let text_ready=cmd_converter(text)?;

    // 转换文本为平工作流
    let plain_flow:Value=toml::from_str(&text_ready)?;

    // 通过正则表达式获取工作流顺序
    let reg=Regex::new(r"\s*\[(\w+)\]")?;
    let mut values:Vec<Value>=Vec::new();
    for cap in reg.captures_iter(&plain_flow.to_string()) {
        let key=&cap[1];
        values.push(plain_flow[key].to_owned());
    }
    // println!("{:?}",values);

    // 解析工作流步骤，生成已解析数组
    let mut res=Vec::new();
    for val in values {
        // 解析步骤头
        let header=val.to_owned().try_into()?;
        // 根据步骤名称解析步骤体
        let parsed_opt:Option<Step>;
        let step_opt=val["step"].as_str();
        let step=step_opt.unwrap();
        match step {
            "Link"=>{
                parsed_opt=Some(Step::StepLink(val.try_into()?))
            },
            "Execute"=>{
                parsed_opt=Some(Step::StepExecute(val.try_into()?))
            },
            "Path"=>{
                parsed_opt=Some(Step::StepPath(val.try_into()?))
            },
            "Log"=>{
                parsed_opt=Some(Step::StepLog(val.try_into()?))
            },
            _=>{
                return Err(anyhow!("Error:Illegal step name '{}' at {}",&step,&val["name"].as_str().unwrap_or("unknown step")));
            }
        }

        res.push(WorkflowNode{
            header,
            body:parsed_opt.unwrap()
        })
    }

    Ok(res)
}