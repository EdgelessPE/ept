mod execute;
mod link;
mod log;
mod path;
pub use execute::step_execute;
pub use link::{manifest_link, step_link};
pub use log::step_log;
pub use path::{manifest_path, step_path};

use anyhow::{anyhow, Result};
use eval::Expr;
use std::path::Path;

use crate::{
    types::{Step, StepExecute, StepLink, StepLog, StepPath, WorkflowHeader, WorkflowNode},
    utils::{get_path_apps, log},
};

use self::{link::reverse_link, path::reverse_path};

// 配置部分内置变量的值
lazy_static! {
    static ref SYSTEM_DRIVE: String = "C:".to_string();
    static ref DEFAULT_LOCATION: String = get_path_apps().to_string_lossy().to_string();
}

// 执行条件以判断是否成立
fn condition_eval(condition: String, exit_code: i32) -> Result<bool> {
    // 执行 eval
    let eval_res = Expr::new(&condition)
        .value("${ExitCode}", exit_code)
        .value(
            "${SystemDrive}",
            eval::Value::String(SYSTEM_DRIVE.to_string()),
        )
        .value("${DefaultLocation}", DEFAULT_LOCATION.to_string())
        .function("Exist", |val| {
            // 参数校验
            if val.len() > 1 {
                return Err(eval::Error::ArgumentsGreater(1));
            }
            if val.len() == 0 {
                return Err(eval::Error::ArgumentsLess(1));
            }
            let str_opt = val[0].as_str();
            if str_opt.is_none() {
                return Err(eval::Error::Custom(
                    "Error:Internal function 'Exist' should accept a string".to_string(),
                ));
            }
            let p = Path::new(str_opt.unwrap());

            Ok(eval::Value::Bool(p.exists()))
        })
        .function("IsDirectory", |val| {
            // 参数校验
            if val.len() > 1 {
                return Err(eval::Error::ArgumentsGreater(1));
            }
            if val.len() == 0 {
                return Err(eval::Error::ArgumentsLess(1));
            }
            let str_opt = val[0].as_str();
            if str_opt.is_none() {
                return Err(eval::Error::Custom(
                    "Error:Internal function 'IsDirectory' should accept a string".to_string(),
                ));
            }
            let p = Path::new(str_opt.unwrap());

            Ok(eval::Value::Bool(p.is_dir()))
        })
        .exec();

    // 检查执行结果
    if eval_res.is_err() {
        return Err(anyhow!(
            "Error:Can't eval statement '{}' : {}",
            &condition,
            eval_res.unwrap_err()
        ));
    }
    let result = eval_res.unwrap().as_bool();
    if result.is_none() {
        return Err(anyhow!(
            "Error:Can't eval statement '{}' into bool result",
            &condition
        ));
    }

    Ok(result.unwrap())
}

// 执行工作流
pub fn workflow_executor(flow: Vec<WorkflowNode>, located: String) -> Result<i32> {
    let mut exit_code = 0;

    // 遍历流节点
    for flow_node in flow {
        // 解释节点条件，判断是否需要跳过执行
        let c_if = flow_node.header.c_if;
        if c_if.is_some() && !condition_eval(c_if.unwrap(), exit_code)? {
            continue;
        }
        // 匹配步骤类型以调用步骤解释器
        let located_cp = located.clone();
        let exec_res = match flow_node.body {
            Step::StepLink(step) => step_link(step, located_cp),
            Step::StepExecute(step) => step_execute(step, located_cp),
            Step::StepPath(step) => step_path(step, located_cp),
            Step::StepLog(step) => step_log(step, located_cp),
        };
        // 处理执行结果
        if exec_res.is_err() {
            log(format!(
                "Warning(Main):Workflow step '{}' failed to execute : {}, check your workflow syntax again",
                &flow_node.header.name,
                exec_res.unwrap_err()
            ));
            exit_code = 1;
        } else {
            exit_code = exec_res.unwrap();
            if exit_code != 0 {
                log(format!(
                    "Warning(Main):Workflow step '{}' finished with exit code '{}'",
                    &flow_node.header.name, exit_code
                ));
            }
        }
    }

    Ok(exit_code)
}

// 宽容地逆向执行 setup 工作流
pub fn workflow_reverse_executor(flow: Vec<WorkflowNode>, located: String) -> Result<()> {
    // 遍历流节点
    for flow_node in flow {
        // 忽略步骤执行条件

        // 匹配步骤类型以调用逆向步骤解释器
        let located_cp = located.clone();
        let exec_res = match flow_node.body {
            Step::StepLink(step) => reverse_link(step, located_cp),
            Step::StepPath(step) => reverse_path(step, located_cp),
            _ => Ok(()),
        };

        // 对错误进行警告
        if exec_res.is_err() {
            log(format!(
                "Warning(Main):Reverse workflow step '{}' failed to execute : {}",
                &flow_node.header.name,
                exec_res.unwrap_err()
            ));
        }
    }

    Ok(())
}

#[test]
fn test_condition_eval() {
    let r1 = condition_eval(String::from("${ExitCode}==114"), 114).unwrap();
    assert!(r1);

    let r2 = condition_eval(String::from("${ExitCode}==514"), 114).unwrap();
    assert_eq!(r2, false);

    let r3 = condition_eval(String::from("${SystemDrive}==\"C:\""), 0).unwrap();
    assert!(r3);

    let r4 = condition_eval(String::from("${DefaultLocation}==\"./apps\""), 0).unwrap();
    assert!(r4);

    let r5 = condition_eval(
        String::from("Exist(\"./src/main.rs\")==IsDirectory(\"./bin\")"),
        0,
    )
    .unwrap();
    assert!(r5);

    let r6 = condition_eval(String::from("Exist(\"./src/main.ts\")"), 0).unwrap();
    assert_eq!(r6, false);
}

#[test]
fn test_workflow_executor() {
    let wf1 = vec![
        WorkflowNode {
            header: WorkflowHeader {
                name: "Log".to_string(),
                step: "Step log".to_string(),
                c_if: None,
            },
            body: Step::StepLog(StepLog {
                level: "Info".to_string(),
                msg: "Hello nep! 你好，尼普！".to_string(),
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: "Throw".to_string(),
                step: "Try throw".to_string(),
                c_if: Some(String::from("${ExitCode}==0")),
            },
            body: Step::StepExecute(StepExecute {
                command: "exit 3".to_string(),
                pwd: None,
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: "Fix".to_string(),
                step: "Try fix".to_string(),
                c_if: Some(String::from("${ExitCode}==3")),
            },
            body: Step::StepLink(StepLink {
                source_file: "D:/CnoRPS/Edgeless Hub/edgeless-hub.exe".to_string(),
                target_name: "Old hub".to_string(),
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: "Exist".to_string(),
                step: "If exist".to_string(),
                c_if: Some("Exist(\"D:/Desktop/Old hub.lnk\")".to_string()),
            },
            body: Step::StepLog(StepLog {
                level: "Warning".to_string(),
                msg: "快捷方式创建成功！".to_string(),
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: "Path".to_string(),
                step: "Create path".to_string(),
                c_if: None,
            },
            body: Step::StepPath(StepPath {
                record: "D:/CnoRPS/chfsgui.exe".to_string(),
            }),
        },
    ];
    let r1 = workflow_executor(
        wf1,
        String::from("D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode"),
    );
    println!("{:?}", r1);
}
