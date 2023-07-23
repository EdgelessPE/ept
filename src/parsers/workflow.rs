use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::Path;
use std::{fs::File, io::Read};
use toml::Value;

use crate::types::steps::Step;
use crate::types::workflow::WorkflowNode;

fn cmd_converter(origin: &String) -> Result<String> {
    // 需要增加 c_ 前缀的字段
    let list = ["if"];

    // 转换器
    let mut text = origin.to_owned();
    for cmd in list {
        let reg_str = String::from("^(");
        let rep = Regex::new(&(reg_str + cmd + r"\s?=.+)"))?;
        text = rep.replace_all(&text, "c_$1").to_string();
    }
    Ok(text)
}

pub fn parse_workflow(p: &String) -> Result<Vec<WorkflowNode>> {
    let workflow_path = Path::new(p);
    if !workflow_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find workflow path : {p}"));
    }

    // 读取文件
    let mut text = String::new();
    File::open(p)?.read_to_string(&mut text)?;

    // 替换条件命令字段
    let text_ready = cmd_converter(&text)?;

    // 反序列化工作流并解析为 Table
    let plain_flow: Value = toml::from_str(&text_ready)
        .map_err(|err| anyhow!("Error:Can't parse '{p}' as legal toml file : {err}"))?;
    let table = plain_flow
        .as_table()
        .ok_or(anyhow!("Error:Failed to convert workflow as valid table"))?
        .to_owned();

    // 解析工作流步骤，生成已解析数组
    let mut res = Vec::new();
    for (key, val) in table {
        // 解析步骤头
        let header = val
            .clone()
            .try_into()
            .map_err(|e| anyhow!("Error:Illegal workflow node at key '{key}' : {e}"))?;

        // 解析步骤体
        let body = Step::try_from_kv(key, val)?;

        res.push(WorkflowNode { header, body })
    }

    Ok(res)
}
