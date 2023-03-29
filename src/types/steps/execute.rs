use crate::log;
use crate::types::mixed_fs::MixedFS;
use crate::types::permissions::{Generalizable, Permission, PermissionLevel};
use crate::types::verifiable::Verifiable;

use super::TStep;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str::from_utf8;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepExecute {
    pub command: String,
    pub pwd: Option<String>,
    pub call_installer: Option<bool>,
}

fn read_console(v: Vec<u8>) -> String {
    let msg_res = from_utf8(&v);
    if msg_res.is_err() {
        log!("Warning(Execute):Console output can't be parsed with utf8");
        String::new()
    } else {
        msg_res.unwrap().to_string()
    }
}

impl TStep for StepExecute {
    fn run(self, located: &String) -> Result<i32> {
        // 配置终端
        let launch_terminal = if cfg!(target_os = "windows") {
            ("cmd", "/c")
        } else {
            ("sh", "-c")
        };

        // 构造执行器
        let mut c = Command::new(launch_terminal.0);
        let cmd = c.args([launch_terminal.1, &self.command]);

        // 指定工作目录
        let workshop = self.pwd.unwrap_or(located.to_owned());
        cmd.current_dir(&workshop);

        // 执行并收集结果
        log!(
            "Info(Execute):Running command '{}' in '{}'",
            &self.command,
            &workshop
        );
        let output = cmd.output().map_err(|err| {
            anyhow!(
                "Error(Execute):Command '{}' spawned failed : {}",
                &self.command,
                err
            )
        })?;

        // 处理退出码
        match output.status.code() {
            Some(val) => {
                if val == 0 {
                    log!(
                        "Info(Execute):Command '{}' output : \n{}",
                        &self.command,
                        &read_console(output.stdout)
                    );
                } else {
                    log!(
                        "Error(Execute):Command '{}' failed, output : \n{}",
                        &self.command,
                        &read_console(output.stderr)
                    );
                }
                Ok(val)
            }
            None => Err(anyhow!(
                "Error(Execute):Command '{}' terminated by signal",
                &self.command
            )),
        }
    }
    fn reverse_run(self, _: &String) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, _fs: &mut MixedFS) -> Vec<String> {
        Vec::new()
    }
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            command: interpreter(self.command),
            pwd: self.pwd.map(interpreter),
            call_installer: self.call_installer,
        }
    }
}

impl Verifiable for StepExecute {
    fn verify_self(&self) -> Result<()> {
        Ok(())
    }
}

impl Generalizable for StepExecute {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        let node = if self.call_installer.unwrap_or(false) {
            Permission {
                key: "execute_installer".to_string(),
                level: PermissionLevel::Important,
                targets: vec![self.command.to_owned()],
            }
        } else {
            Permission {
                key: "execute_custom".to_string(),
                level: PermissionLevel::Sensitive,
                targets: vec![self.command.to_owned()],
            }
        };
        Ok(vec![node])
    }
}

#[test]
fn test_execute() {
    StepExecute {
        command: "echo hello nep ! && echo 你好，尼普！".to_string(),
        pwd: None,
        call_installer: None,
    }
    .run(&String::from(
        "D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode",
    ))
    .unwrap();
    StepExecute {
        command: "ls".to_string(),
        pwd: Some("./src".to_string()),
        call_installer: None,
    }
    .run(&String::from(
        "D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode",
    ))
    .unwrap();

    let res = StepExecute {
        command: "exit 2".to_string(),
        pwd: None,
        call_installer: None,
    }
    .run(&String::from(
        "D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode",
    ))
    .unwrap();
    assert_eq!(res, 2);
}
