use std::{env::current_dir, process::Child};

use super::steps::VerifyStepCtx;
use super::{
    package::GlobalPackage, permissions::Generalizable, steps::Step, verifiable::Verifiable,
};
use crate::log;
use crate::utils::{
    conditions::{get_permissions_from_conditions, verify_conditions},
    term::read_console,
};
use crate::{p2s, types::permissions::Permission};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkflowHeader {
    /// 步骤名称，缺省使用步骤键的 sentence case。
    pub name: Option<String>,
    /// 步骤类型
    //@ 必须是[步骤](/nep/definition/4-steps/0-general.html)定义中的一种值。
    pub step: String,
    #[serde(rename = "if")]
    /// 步骤执行条件。
    //@ 是合法的条件。
    pub c_if: Option<String>,
}

impl WorkflowHeader {
    // 收集 header 中的条件语句
    fn get_conditions(&self) -> Vec<String> {
        let mut conditions = Vec::new();
        if let Some(c_if) = &self.c_if {
            conditions.push(c_if.to_owned());
        }
        conditions
    }
}

impl Generalizable for WorkflowHeader {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        // 获取条件语句所需的权限
        get_permissions_from_conditions(self.get_conditions())
    }
}

impl Verifiable for WorkflowHeader {
    fn verify_self(&self, located: &String) -> Result<()> {
        // 校验条件
        verify_conditions(self.get_conditions(), located, &"1.0.0.0".to_string())
    }
}

#[test]
fn test_header_perm() {
    use crate::types::permissions::{PermissionKey, PermissionLevel};
    let flow=WorkflowHeader{
        name: Some("Name".to_string()),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };
    let res = flow.generalize_permissions().unwrap();
    assert_eq!(
        res,
        vec![
            Permission {
                key: PermissionKey::fs_read,
                level: PermissionLevel::Normal,
                targets: vec!["./mc/vsc.exe".to_string(),],
            },
            Permission {
                key: PermissionKey::fs_read,
                level: PermissionLevel::Sensitive,
                targets: vec!["${SystemDrive}/Windows".to_string(),],
            },
            Permission {
                key: PermissionKey::fs_read,
                level: PermissionLevel::Sensitive,
                targets: vec!["${AppData}/Roaming/Edgeless/ept".to_string(),],
            },
        ]
    )
}

#[test]
fn test_header_valid() {
    let flow=WorkflowHeader{
        name: Some("Name".to_string()),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };

    flow.verify_self(&String::from("./examples/VSCode"))
        .unwrap();

    let flow = WorkflowHeader {
        name: Some("Name".to_string()),
        step: "Step".to_string(),
        c_if: Some("${Arch}==\"X64\"".to_string()),
    };

    assert!(flow
        .verify_self(&String::from("./examples/VSCode"))
        .is_err());
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkflowNode {
    pub header: WorkflowHeader,
    pub body: Step,
}

impl Generalizable for WorkflowNode {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        let mut perm = Vec::new();
        perm.append(&mut self.header.generalize_permissions()?);
        perm.append(&mut self.body.generalize_permissions()?);

        Ok(perm)
    }
}

impl WorkflowNode {
    pub fn verify_step(&self, ctx: &VerifyStepCtx) -> Result<()> {
        self.header.verify_self(&ctx.located)?;
        self.body.verify_step(ctx)
    }
}

pub struct WorkflowContext {
    pub located: String,
    pub pkg: GlobalPackage,
    pub async_execution_handlers: Vec<(String, Child, bool)>, // 命令，handler，是否被抛弃
    pub exit_code: i32,
}

impl WorkflowContext {
    pub fn _demo() -> Self {
        Self::new(&p2s!(current_dir().unwrap()), GlobalPackage::_demo())
    }

    pub fn new(located: &String, pkg: GlobalPackage) -> Self {
        Self {
            pkg,
            located: located.to_owned(),
            async_execution_handlers: Vec::new(),
            exit_code: 0,
        }
    }

    pub fn finish(self) -> Result<i32> {
        log!("Debug:Finish context");

        // 等待异步 handlers
        for (cmd, mut handler, abandon) in self.async_execution_handlers {
            if abandon {
                if let Err(e) = handler.kill() {
                    log!("Warning(Execute):Failed to kill async abandoned command '{cmd}' : {e}");
                } else {
                    log!("Info(Execute):Killed async abandoned command '{cmd}'");
                }
            } else {
                let output = handler.wait_with_output().map_err(|e| {
                    anyhow!("Error(Execute):Failed to wait on async command '{cmd}' : {e}")
                })?;
                // 处理退出码
                match output.status.code() {
                    Some(val) => {
                        if val == 0 {
                            log!("Info(Execute):Async command '{cmd}' output :");
                            println!("{output}", output = read_console(output.stdout));
                        } else {
                            log!("Error(Execute):Async command '{cmd}' failed, output :");
                            println!("{output}", output = read_console(output.stdout));
                        }
                    }
                    None => {
                        log!("Error(Execute):Async command '{cmd}' terminated by signal");
                    }
                }
            }
        }

        Ok(self.exit_code)
    }
}
