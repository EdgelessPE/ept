use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::Path;
use std::{fs::File, io::Read};
use toml::Value;

use crate::types::{workflow::WorkflowNode, KV};

fn cmd_converter(origin: &String) -> Result<String> {
    // 需要增加 c_ 前缀的字段
    let list = ["if"];

    // 转换器
    let mut text = origin.to_owned();
    for cmd in list {
        let reg_str = String::from("(");
        let rep = Regex::new(&(reg_str + cmd + r"\s?=.+)"))?;
        text = rep.replace_all(&text, "c_$1").to_string();
    }
    Ok(text)
}

pub fn parse_workflow(p: &String) -> Result<Vec<WorkflowNode>> {
    let workflow_path = Path::new(p);
    if !workflow_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find workflow path : {}", p));
    }

    // 读取文件
    let mut text = String::new();
    File::open(p)?.read_to_string(&mut text)?;

    // 替换条件命令字段
    let text_ready = cmd_converter(&text)?;

    // 转换文本为平工作流
    let plain_flow: Value = toml::from_str(&text_ready)
        .map_err(|err| anyhow!("Error:Can't parse '{}' as legal toml file : {}", p, err))?;

    // 通过正则表达式获取工作流顺序
    let reg = Regex::new(r"\s*\[(\w+)\]")?;
    let mut kvs: Vec<KV> = Vec::new();
    for cap in reg.captures_iter(&plain_flow.to_string()) {
        let key = &cap[1];
        let value = plain_flow[key].to_owned();
        kvs.push(KV {
            key: key.to_string(),
            value,
        })
    }
    // println!("{:?}",values);

    // 解析工作流步骤，生成已解析数组
    let mut res = Vec::new();
    for kv_node in kvs {
        // 结构键值对
        let kv = kv_node.clone();
        let key = kv_node.key;
        let val = kv_node.value;

        // 解析步骤头
        let header = val
            .try_into()
            .map_err(|e| anyhow!("Error:Illegal workflow node at key '{}' : {}", key, e))?;

        // 根据步骤名称解析步骤体
        let body = kv.try_into()?;

        res.push(WorkflowNode { header, body })
    }

    Ok(res)
}
