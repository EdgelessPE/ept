use std::{env::current_dir, process::Child};

use super::{
    package::GlobalPackage, permissions::Generalizable, steps::Step, verifiable::Verifiable,
};
use crate::log;
use crate::utils::{get_permissions_from_conditions, read_console, verify_conditions};
use crate::{p2s, types::permissions::Permission};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkflowHeader {
    pub name: String,
    pub step: String,
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
        verify_conditions(self.get_conditions(), located)
    }
}

#[test]
fn test_header_perm() {
    use crate::types::permissions::PermissionLevel;
    let flow=WorkflowHeader{
        name: "Name".to_string(),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };
    let res = flow.generalize_permissions().unwrap();
    assert_eq!(
        res,
        vec![
            Permission {
                key: "fs_read".to_string(),
                level: PermissionLevel::Normal,
                targets: vec!["./mc/vsc.exe".to_string(),],
            },
            Permission {
                key: "fs_read".to_string(),
                level: PermissionLevel::Sensitive,
                targets: vec!["${SystemDrive}/Windows".to_string(),],
            },
            Permission {
                key: "fs_read".to_string(),
                level: PermissionLevel::Sensitive,
                targets: vec!["${AppData}/Roaming/Edgeless/ept".to_string(),],
            },
        ]
    )
}

#[test]
fn test_header_valid() {
    let flow=WorkflowHeader{
        name: "Name".to_string(),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };

    flow.verify_self(&String::from("./examples/VSCode"))
        .unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

impl Verifiable for WorkflowNode {
    fn verify_self(&self, located: &String) -> anyhow::Result<()> {
        self.header.verify_self(located)?;
        self.body.verify_self(located)
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
        Self {
            located: p2s!(current_dir().unwrap()),
            pkg: GlobalPackage::_demo(),
            async_execution_handlers: Vec::new(),
            exit_code: 0,
        }
    }

    pub fn finish(self) -> Result<i32> {
        // 等待异步 handlers
        for (cmd, handler, abandon) in self.async_execution_handlers {
            if abandon {
                continue;
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
