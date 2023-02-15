use super::TStep;
use crate::utils::{log};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str::from_utf8;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepExecute {
    pub command: String,
    pub pwd: Option<String>,
}

fn read_console(v: Vec<u8>) -> String {
    let msg_res = from_utf8(&v);
    if msg_res.is_err() {
        log("Warning(Execute):Console output can't be parsed with utf8".to_string());
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
        log(format!(
            "Info(Execute):Running command '{}' in '{}'",
            &self.command, &workshop
        ));
        let output_res = cmd.output();
        if output_res.is_err() {
            return Err(anyhow!(
                "Error(Execute):Command '{}' spawned failed : {}",
                &self.command,
                output_res.unwrap_err()
            ));
        }
        let output = output_res.unwrap();

        // 处理退出码
        match output.status.code() {
            Some(val) => {
                if val == 0 {
                    log(format!(
                        "Info(Execute):Command '{}' output : \n{}",
                        &self.command,
                        &read_console(output.stdout)
                    ));
                } else {
                    log(format!(
                        "Error(Execute):Command '{}' failed, output : \n{}",
                        &self.command,
                        &read_console(output.stderr)
                    ));
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
    fn get_manifest(&self) -> Vec<String> {
        Vec::new()
    }
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            command: interpreter(self.command),
            pwd: self.pwd.map(interpreter),
        }
    }
}

#[test]
fn test_execute() {
    StepExecute {
        command: "echo hello nep ! && echo 你好，尼普！".to_string(),
        pwd: None,
    }
    .run(&String::from(
        "D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode",
    ))
    .unwrap();
    StepExecute {
        command: "ls".to_string(),
        pwd: Some("./src".to_string()),
    }
    .run(&String::from(
        "D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode",
    ))
    .unwrap();

    let res = StepExecute {
        command: "exit 2".to_string(),
        pwd: None,
    }
    .run(&String::from(
        "D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode",
    ))
    .unwrap();
    assert_eq!(res, 2);
}
