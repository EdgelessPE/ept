use anyhow::{anyhow, Result};
use regex::Regex;
use serde::de;
use std::path::Path;
use std::{fs::File, io::Read};
use toml::Value;

use crate::types::{Step, WorkflowNode};

#[derive(Clone, Debug)]
struct KV {
    key: String,
    value: Value,
}

fn cmd_converter(origin: String) -> Result<String> {
    // 需要增加 c_ 前缀的字段
    let list = ["if"];

    // 转换器
    let mut text = origin;
    for cmd in list {
        let reg_str = String::from("(");
        let rep = Regex::new(&(reg_str + cmd + r"\s?=.+)"))?;
        text = rep.replace_all(&text, "c_$1").to_string();
    }
    Ok(text)
}

fn toml_try_into<'de, T>(kv: KV) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    let val = kv.value;
    let res = val.to_owned().try_into();
    if res.is_err() {
        let key = kv.key;
        let name_brw = val["name"].to_owned();
        let name = name_brw.as_str().unwrap_or("unknown name");
        let step = val["step"].as_str().unwrap_or("unknown step");
        return Err(anyhow!(
            "Error:Can't parse workflow node '{}'({}) into step '{}' : {}",
            &name,
            &key,
            &step,
            &res.err().unwrap().to_string()
        ));
    } else {
        Ok(res.unwrap())
    }
}

pub fn parse_workflow(p: String) -> Result<Vec<WorkflowNode>> {
    let workflow_path = Path::new(&p);
    if !workflow_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find workflow path : {}", p));
    }

    // 读取文件
    let mut text = String::new();
    File::open(&p)?.read_to_string(&mut text)?;

    // 替换条件命令字段
    let text_ready = cmd_converter(text)?;

    // 转换文本为平工作流
    let plain_flow_res = toml::from_str(&text_ready);
    if plain_flow_res.is_err() {
        return Err(anyhow!(
            "Error:Can't parse '{}' as legal toml file : {}",
            &p,
            plain_flow_res.unwrap_err()
        ));
    }
    let plain_flow: Value = plain_flow_res.unwrap();

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
        let header_res = val.to_owned().try_into();
        if header_res.is_err() {
            return Err(anyhow!(
                "Error:Illegal workflow node at key '{}' : {}",
                key,
                header_res.unwrap_err()
            ));
        }

        // 读取步骤名称
        let step_opt = val["step"].as_str();
        let step = step_opt.unwrap();

        // 根据步骤名称解析步骤体
        let body = match step {
            "Link" => Step::StepLink(toml_try_into(kv)?),
            "Execute" => Step::StepExecute(toml_try_into(kv)?),
            "Path" => Step::StepPath(toml_try_into(kv)?),
            "Log" => Step::StepLog(toml_try_into(kv)?),
            _ => {
                return Err(anyhow!(
                    "Error:Illegal step name '{}' at ‘{}’({})",
                    &step,
                    &val["name"].as_str().unwrap_or("unknown step"),
                    &key
                ));
            }
        };

        res.push(WorkflowNode {
            header: header_res.unwrap(),
            body,
        })
    }

    Ok(res)
}
