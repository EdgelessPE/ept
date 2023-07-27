use crate::types::mixed_fs::MixedFS;
use crate::types::permissions::{Generalizable, Permission, PermissionLevel};
use crate::types::verifiable::Verifiable;
use crate::types::workflow::WorkflowContext;
use crate::utils::{format_path, read_console};
use crate::{log, verify_enum};

use super::TStep;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepExecute {
    pub command: String,
    pub pwd: Option<String>,
    pub call_installer: Option<bool>,
    pub wait: Option<String>,
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
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        // 异步执行分流
        let wait = self.wait.unwrap_or("Sync".to_string());
        if wait == "Sync".to_string() {
            // 同步执行并收集结果
            log!(
                "Info(Execute):Running sync command '{cmd}' in '{workshop}'",
                cmd = self.command,
            );
            let start_instant = Instant::now();
            let output = cmd.output().map_err(|err| {
                anyhow!(
                    "Error(Execute):Command '{cmd}' execution failed : {err}",
                    cmd = self.command,
                )
            })?;

            // 如果在调用安装器，检查是否过快退出
            let duration = start_instant.elapsed();
            let sec = duration.as_secs_f32();
            let (level, hint) = if self.call_installer.unwrap_or(false)
                && duration.as_millis() <= 500
            {
                ("Warning",format!("exited in {sec:.1}s, you may need to manually operate the installer/uninstaller to complete the steps"))
            } else {
                ("Info", format!("exited in {sec:.1}s"))
            };

            // 处理退出码
            match output.status.code() {
                Some(val) => {
                    if val == 0 {
                        log!(
                            "{level}(Execute):Command '{cmd}' {hint}, output :",
                            cmd = self.command
                        );
                        println!("{output}", output = read_console(output.stdout));
                    } else {
                        log!(
                            "Error(Execute):Failed command '{cmd}' {hint}, output(code={val}) : \n{o}",
                            cmd = self.command,
                            o = read_console(output.stderr)
                        );
                        println!("{output}", output = read_console(output.stdout));
                    }
                    Ok(val)
                }
                None => Err(anyhow!(
                    "Error(Execute):Command '{cmd}' terminated by outer signal",
                    cmd = self.command
                )),
            }
        } else {
            // 异步执行
            log!(
                "Info(Execute):Running async command('{wait}') '{cmd}' in '{workshop}'",
                cmd = self.command,
            );
            let handler = cmd.spawn().map_err(|e| {
                anyhow!(
                    "Error(Execute):Command '{cmd}' spawn failed : {e}",
                    cmd = self.command
                )
            })?;
            cx.async_execution_handlers.push((
                self.command,
                handler,
                wait == "Abandon".to_string(),
            ));

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
        let formatted_cmd = format_path(&self.command);

        // 禁止出现 :/
        if formatted_cmd.contains(":/") {
            return Err(anyhow!("Error:Absolute path in '{formatted_cmd}' is not allowed (keyword ':/' detected), use proper inner values instead"));
        }

        // 校验 wait 枚举值
        if let Some(wait) = &self.wait {
            verify_enum!("Execute", "wait", wait, "Sync" | "Delay" | "Abandon")?;
        }

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

    StepExecute {
        command: "exit 0".to_string(),
        pwd: None,
        call_installer: Some(true),
        wait: None,
    }
    .run(&mut cx)
    .unwrap();
}

#[test]
fn test_async_execute() {
    use crate::types::steps::StepLog;
    let mut cx = WorkflowContext::_demo();

    StepExecute {
        command: "timeout 3 && echo 1st第一条输出".to_string(),
        pwd: None,
        call_installer: None,
        wait: Some("Delay".to_string()),
    }
    .run(&mut cx)
    .unwrap();
    StepExecute {
        command: "dir".to_string(),
        pwd: Some("./src".to_string()),
        call_installer: None,
        wait: Some("Delay".to_string()),
    }
    .run(&mut cx)
    .unwrap();

    let res = StepExecute {
        command: "exit 2".to_string(),
        pwd: None,
        call_installer: None,
        wait: Some("Delay".to_string()),
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 0);

    StepLog {
        level: "Info".to_string(),
        msg: "running other steps...".to_string(),
    }
    .run(&mut cx)
    .unwrap();

    cx.finish().unwrap();
    println!("Exit");
}
