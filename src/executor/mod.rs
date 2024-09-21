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
    utils::{arch::is_current_arch_match, get_bare_apps, get_system_drive},
};

pub use self::functions::{
    get_eval_function_names, get_eval_function_permission, verify_eval_function_arg,
};
pub use self::values::{judge_perm_level, values_replacer, values_validator_path};
use self::{
    functions::set_context_with_function,
    values::{set_context_with_constant_values, set_context_with_mutable_values},
};

// 配置部分内置变量的值
lazy_static! {
    static ref SYSTEM_DRIVE: String = get_system_drive().unwrap();
    static ref DEFAULT_LOCATION: String = p2s!(get_bare_apps().unwrap());
}

pub fn get_eval_context(
    exit_code: i32,
    located: &String,
    package_version: &String,
) -> HashMapContext {
    let mut context = HashMapContext::new();
    set_context_with_constant_values(&mut context);
    set_context_with_mutable_values(&mut context, exit_code, located, package_version);
    set_context_with_function(&mut context, located);
    context
}

// 执行条件以判断是否成立
pub fn condition_eval(
    condition: &String,
    exit_code: i32,
    located: &String,
    package_version: &String,
) -> Result<bool> {
    // 装饰变量与函数
    let condition_with_values_interpreted =
        values_replacer(condition.to_owned(), exit_code, located, package_version);
    let context = get_eval_context(exit_code, located, package_version);

    // 执行 eval
    eval_boolean_with_context(&condition_with_values_interpreted, &context).map_err(|res| {
        anyhow!("Error:Can't eval statement '{condition}'  into bool result : {res}")
    })
}

// 执行工作流，返回最后一个步骤的退出码
pub fn workflow_executor(
    flow: Vec<WorkflowNode>,
    located: String,
    pkg: GlobalPackage,
) -> Result<i32> {
    let strict_mode = pkg.package.strict.unwrap_or(true);

    // 检查包架构是否与当前架构相同
    if let Some(software) = &pkg.software {
        if let Some(arch) = &software.arch {
            is_current_arch_match(arch)?;
        }
    }

    // 准备上下文
    let package_version = pkg.package.version.clone();
    let mut cx = WorkflowContext::new(&located, pkg);

    // 遍历流节点
    for flow_node in flow {
        let name = flow_node.header.name.unwrap();
        log!("Debug:Start step '{name}'");
        // 解释节点条件，判断是否需要跳过执行
        if let Some(c_if) = flow_node.header.c_if {
            if !condition_eval(&c_if, cx.exit_code, &located, &package_version)? {
                continue;
            }
        }

        // 创建变量解释器
        let cur_exit_code = cx.exit_code;
        let interpreter =
            |raw: String| values_replacer(raw, cur_exit_code, &located, &package_version);

        // 匹配步骤类型以调用步骤解释器
        let exec_res = flow_node.body.run(&mut cx, interpreter);
        // 处理执行结果
        if let Err(e) = exec_res {
            log!(
                "Warning(Main):Workflow step '{name}' failed to execute : {e}, check your workflow syntax again",
            );
            cx.exit_code = 1;
        } else {
            cx.exit_code = exec_res.unwrap();
            if cx.exit_code != 0 {
                log!(
                    "Warning(Main):Workflow step '{name}' finished with exit code '{exit_code}'",
                    exit_code = cx.exit_code,
                );
            }
        }

        // 在严格模式下立即返回错误
        if cx.exit_code != 0 && strict_mode {
            return Err(anyhow!(
                "Error:Throw due to strict mode : got exit code '{}' at step '{name}'",
                cx.exit_code
            ));
        }
        log!("Debug:Stop step '{name}'");
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
    let package_version = pkg.package.version.clone();
    let mut cx = WorkflowContext::new(&located, pkg);

    // 遍历流节点
    for flow_node in flow {
        let name = flow_node.header.name.unwrap();
        log!("Debug:Start reverse step '{name}'");
        // 创建变量解释器，ExitCode 始终置 0
        let interpreter = |raw: String| values_replacer(raw, 0, &located, &package_version);
        // 匹配步骤类型以调用逆向步骤解释器
        let exec_res = flow_node.body.reverse_run(&mut cx, interpreter);

        // 对错误进行警告
        if let Err(e) = exec_res {
            log!("Warning(Main):Reverse workflow step '{name}' failed to execute : {e}");
        }
        log!("Debug:Stop reverse step '{name}'");
    }

    // 完成
    cx.finish()?;

    Ok(())
}

#[test]
fn test_condition_eval() {
    let located = &String::from("./examples/VSCode");
    let r1 = condition_eval(
        &String::from("\"${ExitCode}\"==\"114\" && ExitCode==114 && \"${PackageVersion}\"==\"1.0.0.0\" && PackageVersion==\"1.0.0.0\""),
        114,
        located,
        &"1.0.0.0".to_string(),
    )
    .unwrap();
    assert!(r1);

    let r2 = condition_eval(
        &String::from("\"${ExitCode}\"!=\"114\" || ExitCode==514"),
        114,
        located,
        &"1.0.0.0".to_string(),
    )
    .unwrap();
    assert!(!r2);

    let r3 = condition_eval(
        &String::from("\"${SystemDrive}\"==\"C:\" && SystemDrive==\"C:\""),
        0,
        located,
        &"1.0.0.0".to_string(),
    )
    .unwrap();
    assert!(r3);

    let r4 = condition_eval(
        &String::from("\"${DefaultLocation}\"==\"./unknown/VSCode\""),
        0,
        located,
        &"1.0.0.0".to_string(),
    )
    .unwrap();
    assert!(!r4);

    let r5 = condition_eval(
        &String::from("Exist(\"src/main.rs\") && IsDirectory(\"src\")"),
        0,
        &String::from("./"),
        &"1.0.0.0".to_string(),
    )
    .unwrap();
    assert!(r5);

    let r6 = condition_eval(
        &String::from("Exist(\"./src/main.ts\")"),
        0,
        located,
        &"1.0.0.0".to_string(),
    )
    .unwrap();
    assert!(!r6);

    let r7 = condition_eval(
        &String::from("Exist(\"${AppData}\") && IsDirectory(\"${SystemDrive}/Windows\")"),
        0,
        located,
        &"1.0.0.0".to_string(),
    )
    .unwrap();
    assert!(r7);
}

#[test]
fn test_workflow_executor() {
    use crate::utils::flags::{set_flag, Flag};
    set_flag(Flag::Debug, true);
    use crate::types::steps::{Step, StepExecute, StepLog};
    use crate::types::workflow::{WorkflowHeader, WorkflowNode};
    let cx = WorkflowContext::_demo();
    let wf1 = vec![
        WorkflowNode {
            header: WorkflowHeader {
                name: Some("Log".to_string()),
                step: "Step log".to_string(),
                c_if: None,
            },
            body: Step::StepLog(StepLog {
                level: None,
                msg: "Hello nep! 你好，尼普！".to_string(),
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: Some("Throw".to_string()),
                step: "Try throw".to_string(),
                c_if: Some(String::from("${ExitCode}==0")),
            },
            body: Step::StepExecute(StepExecute {
                command: "exit 3".to_string(),
                pwd: None,
                call_installer: None,
                wait: None,
                ignore_exit_code: None,
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: Some("Exist".to_string()),
                step: "If exist".to_string(),
                c_if: Some(
                    "IsAlive(\"unknown.exe\") && IsInstalled(\"Microsoft/VSCode\")".to_string(),
                ),
            },
            body: Step::StepLog(StepLog {
                level: Some("Warning".to_string()),
                msg: "桌面路径：${Desktop}，应用路径：${DefaultLocation}".to_string(),
            }),
        },
    ];
    assert!(workflow_executor(wf1, cx.located, cx.pkg).is_err());
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
                name: Some("Throw".to_string()),
                step: "Try throw".to_string(),
                c_if: Some(String::from("${ExitCode}==0")),
            },
            body: Step::StepExecute(StepExecute {
                command: "exit 3".to_string(),
                pwd: None,
                call_installer: None,
                wait: None,
                ignore_exit_code: None,
            }),
        },
        WorkflowNode {
            header: WorkflowHeader {
                name: Some("Log".to_string()),
                step: "Step log".to_string(),
                c_if: None,
            },
            body: Step::StepLog(StepLog {
                level: None,
                msg: "exit : ${ExitCode}".to_string(),
            }),
        },
    ];
    let mut cx = WorkflowContext::_demo();
    cx.pkg.package.strict = Some(false);
    let code = workflow_executor(flow, cx.located, cx.pkg).unwrap();
    assert_eq!(code, 0);
}

#[test]
fn test_workflow_with_strict_mode() {
    use crate::types::{
        steps::{Step, StepExecute},
        workflow::{WorkflowHeader, WorkflowNode},
    };

    let flow = vec![WorkflowNode {
        header: WorkflowHeader {
            name: Some("Throw".to_string()),
            step: "Try throw".to_string(),
            c_if: Some(String::from("${ExitCode}==0")),
        },
        body: Step::StepExecute(StepExecute {
            command: "exit 3".to_string(),
            pwd: None,
            call_installer: None,
            wait: None,
            ignore_exit_code: None,
        }),
    }];

    // 默认情况下是严格模式
    let cx = WorkflowContext::_demo();
    assert!(workflow_executor(flow.clone(), cx.located, cx.pkg).is_err());

    // 显式申明禁用严格模式
    let mut cx = WorkflowContext::_demo();
    cx.pkg.package.strict = Some(false);
    let code = workflow_executor(flow.clone(), cx.located, cx.pkg).unwrap();
    assert_eq!(code, 3);
}
