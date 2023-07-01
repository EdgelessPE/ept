use std::{env::current_dir, process::Child};

use super::{
    package::GlobalPackage, permissions::Generalizable, steps::Step, verifiable::Verifiable,
};
use crate::log;
use crate::utils::read_console;
use crate::{p2s, types::permissions::Permission};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkflowHeader {
    pub name: String,
    pub step: String,
    pub c_if: Option<String>,
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
    pub async_execution_handlers: Vec<(String, Child)>,
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
        for (cmd, handler) in self.async_execution_handlers {
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

        Ok(self.exit_code)
    }
}
