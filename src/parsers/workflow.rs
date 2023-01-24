use std::{fs::File, io::Read};
use std::path::Path;
use anyhow::{anyhow,Result};
use toml::{Value};
use crate::types::{WorkflowNode};
use regex::Regex;

pub fn parse_workflow(p:String)->Result<Vec<WorkflowNode>> {
    let workflow_path=Path::new(&p);
    if !workflow_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find workflow path : {}",p));
    }

    let mut text=String::new();
    File::open(&p)?.read_to_string(&mut text)?;
    let plain_flow:Value=toml::from_str(&text)?;

    // 通过正则表达式获取工作流顺序
    let reg=Regex::new(r"\[(\w+)\]")?;
    let res:Vec<WorkflowNode>=Vec::new();
    for cap in reg.captures_iter(&text) {
        let key=&cap[1];
        let value:WorkflowNode=plain_flow[key];
        res.push(value);
    }

    Ok(res)
}