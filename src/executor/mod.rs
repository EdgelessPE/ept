mod functions;
mod values;

use anyhow::{anyhow, Result};
use evalexpr::*;

use crate::{
    log, p2s,
    types::{
        package::GlobalPackage,
        workflow::{WorkflowContext, WorkflowNode},
    },
    utils::{get_bare_apps, is_current_arch_match, is_strict_mode},
};

use self::{functions::get_context_with_function, values::values_replacer};

pub use self::values::{judge_perm_level, values_validator_path};

// 配置部分内置变量的值
lazy_static! {
    static ref SYSTEM_DRIVE: String = "C:".to_string();
    static ref DEFAULT_LOCATION: String = p2s!(get_bare_apps().unwrap());
}

// 执行条件以判断是否成立
pub fn condition_eval(condition: &String, exit_code: i32, located: &String) -> Result<bool> {
    // 装饰变量与函数
    let condition_with_values_interpreted =
        values_replacer(condition.to_owned(), exit_code, located);
    let context = get_context_with_function(located);

    // 执行 eval
    eval_boolean_with_context(&condition_with_values_interpreted, &context).map_err(|res| {
        anyhow!("Error:Can't eval statement '{condition}'  into bool result : {res}")
    })
}

// 执行工作流
pub fn workflow_executor(
    flow: Vec<WorkflowNode>,
    located: String,
    pkg: GlobalPackage,
) -> Result<i32> {
    let strict_mode = is_strict_mode();

    // 检查包架构是否与当前架构相同
    if let Some(software) = &pkg.software {
        if let Some(arch) = &software.arch {
            is_current_arch_match(arch)?;
        }
    }

    // 准备上下文
    let mut cx = WorkflowContext {
        pkg,
        located: located.clone(),
        async_execution_handlers: Vec::new(),
        exit_code: 0,
    };

    // 遍历流节点
    for flow_node in flow {
        log!("Debug:Start step '{name}'", name = flow_node.header.name);
        // 解释节点条件，判断是否需要跳过执行
        let c_if = flow_node.header.c_if;
        if c_if.is_some() && !condition_eval(&c_if.unwrap(), cx.exit_code, &located)? {
            continue;
        }

        // 创建变量解释器
        let cur_exit_code = cx.exit_code;
        let interpreter = |raw: String| values_replacer(raw, cur_exit_code, &located);

        // 匹配步骤类型以调用步骤解释器
        let exec_res = flow_node.body.run(&mut cx, interpreter);
        // 处理执行结果
        if let Err(e) = exec_res {
            log!(
                "Warning(Main):Workflow step '{name}' failed to execute : {e}, check your workflow syntax again",
                name=flow_node.header.name,
            );
            cx.exit_code = 1;
        } else {
            cx.exit_code = exec_res.unwrap();
            if cx.exit_code != 0 {
                log!(
                    "Warning(Main):Workflow step '{name}' finished with exit code '{exit_code}'",
                    name = flow_node.header.name,
                    exit_code = cx.exit_code,
                );
            }
        }

        // 在严格模式下立即返回错误
        if cx.exit_code != 0 && strict_mode {
            return Err(anyhow!("Error:Throw due to strict mode"));
        }
        log!("Debug:Stop step '{name}'", name = flow_node.header.name);
    }

    // 完成
    cx.finish()
}

// 宽容地逆向执行 setup 工作流
pub fn workflow_reverse_executor(
    flow: Vec<WorkflowNode>,
    located: String,
    pkg: GlobalPackage,
) -> Result<()> {
    let mut cx = WorkflowContext {
        pkg,
        located: located.clone(),
        async_execution_handlers: Vec::new(),
        exit_code: 0,
    };

    // 遍历流节点
    for flow_node in flow {
        log!(
            "Debug:Start reverse step '{name}'",
            name = flow_node.header.name
        );
        // 创建变量解释器，ExitCode 始终置 0
        let interpreter = |raw: String| values_replacer(raw, 0, &located);
        // 匹配步骤类型以调用逆向步骤解释器
        let exec_res = flow_node.body.reverse_run(&mut cx, interpreter);

        // 对错误进行警告
        if let Err(e) = exec_res {
            log!(
                "Warning(Main):Reverse workflow step '{name}' failed to execute : {e}",
                name = flow_node.header.name
            );
        }
        log!(
            "Debug:Stop reverse step '{name}'",
            name = flow_node.header.name
        );
    }

    // 完成
    cx.finish()?;

    Ok(())
}

#[test]
fn test_condition_eval() {
    let located = &String::from("./examples/VSCode");
    let r1 = condition_eval(&String::from("${ExitCode}==114"), 114, located).unwrap();
    assert!(r1);

    let r2 = condition_eval(&String::from("${ExitCode}==514"), 114, located).unwrap();
    assert_eq!(r2, false);

    let r3 = condition_eval(&String::from("\"${SystemDrive}\"==\"C:\""), 0, located).unwrap();
    assert!(r3);

    let r4 = condition_eval(
        &String::from("\"${DefaultLocation}\"==\"./unknown/VSCode\""),
        0,
        located,
    )
    .unwrap();
    assert!(!r4);

    let r5 = condition_eval(
        &String::from("Exist(\"src/main.rs\") && IsDirectory(\"src\")"),
        0,
        &String::from("./"),
    )
    .unwrap();
    assert!(r5);

    let r6 = condition_eval(&String::from("Exist(\"./src/main.ts\")"), 0, located).unwrap();
    assert_eq!(r6, false);

    let r7 = condition_eval(
        &String::from("Exist(\"${AppData}\") && IsDirectory(\"${SystemDrive}/Windows\")"),
        0,
        located,
    )
    .unwrap();
    assert!(r7);
}

#[test]
fn test_workflow_executor() {
    envmnt::set("DEBUG", "true");
    use crate::types::steps::{
        Step,
        StepExecute,
        // StepLink,
        StepLog,
        // StepPath,
    };
    use crate::types::workflow::{WorkflowHeader, WorkflowNode};
    let cx = WorkflowContext::_demo();
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
                call_installer: None,
                wait: None,
            }),
        },
        // WorkflowNode {
        //     header: WorkflowHeader {
        //         name: "Fix".to_string(),
        //         step: "Try fix".to_string(),
        //         c_if: Some(String::from("${ExitCode}==3")),
        //     },
        //     body: Step::StepLink(StepLink {
        //         source_file: "D:/CnoRPS/Edgeless Hub/edgeless-hub.exe".to_string(),
        //         target_name: "Old hub".to_string(),
        //     }),
        // },
        WorkflowNode {
            header: WorkflowHeader {
                name: "Exist".to_string(),
                step: "If exist".to_string(),
                c_if: Some(
                    "IsAlive(\"unknown.exe\") && IsInstalled(\"Microsoft/VSCode\")".to_string(),
                ),
            },
            body: Step::StepLog(StepLog {
                level: "Warning".to_string(),
                msg: "桌面路径：${Desktop}，应用路径：${DefaultLocation}".to_string(),
            }),
        },
        // WorkflowNode {
        //     header: WorkflowHeader {
        //         name: "Exist".to_string(),
        //         step: "If exist".to_string(),
        //         c_if: Some("Exist(\"${ProgramFiles_X64}/nodejs/node.exe\") && IsDirectory(\"${Desktop}/Projects\") && Exist(\"./Cargo.lock\")".to_string()),
        //     },
        //     body: Step::StepLog(StepLog {
        //         level: "Warning".to_string(),
        //         msg: "桌面路径：${Desktop}，应用路径：${DefaultLocation}".to_string(),
        //     }),
        // },
        // WorkflowNode {
        //     header: WorkflowHeader {
        //         name: "Path".to_string(),
        //         step: "Create path".to_string(),
        //         c_if: None,
        //     },
        //     body: Step::StepPath(StepPath {
        //         record: "D:/CnoRPS/chfsgui.exe".to_string(),
        //     }),
        // },
    ];
    let r1 = workflow_executor(wf1, cx.located, cx.pkg).unwrap();
    assert_eq!(r1, 3);
}

#[test]
fn test_workflow_executor_interpreter() {
    use crate::types::{
        steps::{Step, StepExecute, StepLog},
        workflow::{WorkflowHeader, WorkflowNode},
    };

    let flow = vec![
        WorkflowNode {
            header: WorkflowHeader {
                name: "Throw".to_string(),
                step: "Try throw".to_string(),
                c_if: Some(String::from("${ExitCode}==0")),
            },
            body: Step::StepExecute(StepExecute {
                command: "exit 3".to_string(),
                pwd: None,
                call_installer: None,
                wait: None,
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: "Log".to_string(),
                step: "Step log".to_string(),
                c_if: None,
            },
            body: Step::StepLog(StepLog {
                level: "Info".to_string(),
                msg: "exit : ${ExitCode}".to_string(),
            }),
        },
    ];
    let cx = WorkflowContext::_demo();
    workflow_executor(flow, cx.located, cx.pkg).unwrap();
}
