use crate::log;
use crate::types::mixed_fs::MixedFS;
use crate::types::permissions::{Generalizable, Permission, PermissionLevel};
use crate::types::verifiable::Verifiable;
use crate::types::workflow::WorkflowContext;
use crate::utils::read_console;

use super::TStep;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepExecute {
    pub command: String,
    pub pwd: Option<String>,
    pub call_installer: Option<bool>,
    pub wait: Option<bool>,
}

impl TStep for StepExecute {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
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
        let workshop = self.pwd.unwrap_or(cx.located.to_owned());
        cmd.current_dir(&workshop);

        // 指定 stdio
        cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        // 异步执行分流
        if self.wait.unwrap_or(true){
            // 同步执行并收集结果
            log!(
                "Info(Execute):Running sync command '{cmd}' in '{workshop}'",
                cmd = self.command,
            );
            let output = cmd.output().map_err(|err| {
                anyhow!(
                    "Error(Execute):Command '{cmd}' execution failed : {err}",
                    cmd = self.command,
                )
            })?;
    
            // 处理退出码
            match output.status.code() {
                Some(val) => {
                    if val == 0 {
                        log!(
                            "Info(Execute):Command '{cmd}' output :",
                            cmd = self.command
                        );
                        println!("{output}",output=read_console(output.stdout));
                    } else {
                        log!(
                            "Error(Execute):Command '{cmd}' failed, output : \n{o}",
                            cmd = self.command,
                            o = read_console(output.stderr)
                        );
                        println!("{output}",output=read_console(output.stdout));
                    }
                    Ok(val)
                }
                None => Err(anyhow!(
                    "Error(Execute):Command '{cmd}' terminated by signal",
                    cmd = self.command
                )),
            }

        }else{
            // 异步执行
            log!(
                "Info(Execute):Running async command '{cmd}' in '{workshop}' without wait",
                cmd = self.command,
            );
            let handler=cmd.spawn().map_err(|e|anyhow!("Error(Execute):Command '{cmd}' execution failed : {e}",cmd=self.command))?;
            cx.async_execution_handlers.push((self.command,handler));

            Ok(0)
        }
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
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
            wait: self.wait,
        }
    }
}

impl Verifiable for StepExecute {
    fn verify_self(&self, _: &String) -> Result<()> {
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
    let mut cx = WorkflowContext::_demo();

    StepExecute {
        command: "echo hello nep ! && echo 你好，尼普！".to_string(),
        pwd: None,
        call_installer: None,
        wait: None,
    }
    .run(&mut cx)
    .unwrap();
    StepExecute {
        command: "dir".to_string(),
        pwd: Some("./src".to_string()),
        call_installer: None,
        wait: None,
    }
    .run(&mut cx)
    .unwrap();

    let res = StepExecute {
        command: "exit 2".to_string(),
        pwd: None,
        call_installer: None,
        wait: None,
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 2);
}


#[test]
fn test_async_execute() {
    use crate::types::steps::StepLog;
    let mut cx = WorkflowContext::_demo();

    StepExecute {
        command: "timeout 3 && echo 1st第一条输出".to_string(),
        pwd: None,
        call_installer: None,
        wait: Some(false),
    }
    .run(&mut cx)
    .unwrap();
    StepExecute {
        command: "dir".to_string(),
        pwd: Some("./src".to_string()),
        call_installer: None,
        wait: Some(false),
    }
    .run(&mut cx)
    .unwrap();

    let res = StepExecute {
        command: "exit 2".to_string(),
        pwd: None,
        call_installer: None,
        wait: Some(false),
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 0);

    StepLog{
        level:"Info".to_string(),
        msg:"running other steps...".to_string(),
    }
    .run(&mut cx)
    .unwrap();

    cx.finish().unwrap();
    println!("Exit");
}