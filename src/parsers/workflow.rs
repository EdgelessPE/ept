use anyhow::{anyhow, Result};
use inflector::cases::sentencecase::to_sentence_case;
use std::path::Path;
use std::{fs::File, io::Read};
use toml::Value;

use crate::types::steps::Step;
use crate::types::workflow::{WorkflowHeader, WorkflowNode};

pub fn parse_workflow(p: &String) -> Result<Vec<WorkflowNode>> {
    let workflow_path = Path::new(p);
    if !workflow_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find workflow path : {p}"));
    }

    // 读取文件
    let mut text = String::new();
    File::open(p)?.read_to_string(&mut text)?;

    // 反序列化工作流并解析为 Table
    let plain_flow: Value = toml::from_str(&text)
        .map_err(|err| anyhow!("Error:Can't parse '{p}' as legal toml file : {err}"))?;
    let table = plain_flow
        .as_table()
        .ok_or(anyhow!("Error:Failed to convert workflow as valid table"))?
        .to_owned();

    // 解析工作流步骤，生成已解析数组
    let mut res = Vec::new();
    for (key, val) in table {
        // 解析步骤头
        let mut header: WorkflowHeader = val
            .clone()
            .try_into()
            .map_err(|e| anyhow!("Error:Illegal workflow node at key '{key}' : {e}"))?;

        // 如果步骤头没有提供 name 则使用 key 的 sentence case
        if header.name.is_none() {
            header.name = Some(to_sentence_case(&key));
        }

        // 解析步骤体
        let body = Step::try_from_kv(key, val)?;

        res.push(WorkflowNode { header, body })
    }

    Ok(res)
}

#[test]
fn test_parse_workflow() {
    use crate::types::steps::{Step, StepLink, StepPath, StepWait};
    use crate::types::workflow::{WorkflowHeader, WorkflowNode};

    let res = parse_workflow(&"examples/ComplexSteps/workflows/setup.toml".to_string()).unwrap();
    let answer = vec![
        WorkflowNode {
            header: WorkflowHeader {
                name: Some("Create shortcut".to_string()),
                step: "Link".to_string(),
                c_if: None,
            },
            body: Step::StepLink(StepLink {
                source_file: "Code.exe".to_string(),
                target_name: Some("Visual Studio Code".to_string()),
                target_args: None,
                target_icon: None,
                at: None,
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: Some("Add PATH".to_string()),
                step: "Path".to_string(),
                c_if: Some("${AppData} if = 114514".to_string()),
            },
            body: Step::StepPath(StepPath {
                record: "Code.exe".to_string(),
                alias: None,
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: Some("Wait".to_string()),
                step: "Wait".to_string(),
                c_if: None,
            },
            body: Step::StepWait(StepWait {
                timeout: 30000,
                break_if: Some("true".to_string()),
            }),
        },
    ];
    assert_eq!(res, answer);
}
