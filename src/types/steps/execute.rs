use crate::executor::values_validator_path;
use crate::types::interpretable::Interpretable;
use crate::types::mixed_fs::MixedFS;
use crate::types::permissions::{Generalizable, Permission, PermissionKey, PermissionLevel};
use crate::types::verifiable::Verifiable;
use crate::types::workflow::WorkflowContext;
use crate::utils::{
    command::split_command, format_path, is_starts_with_inner_value, term::read_console,
};
use crate::{log, verify_enum};

use super::TStep;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepExecute {
    /// 需要执行的命令，使用终端为 cmd。
    //# `command = "./installer.exe /S"`
    //@ 不得出现反斜杠（\），需使用正斜杠代替
    //@ 符合 POSIX 命令格式
    //@ 不得出现绝对路径（使用[内置变量](/nep/workflow/2-context.html#内置变量)）
    pub command: String,
    /// 执行目录，缺省为包安装目录。
    //# `pwd = "${AppData}/Microsoft"`
    //@ 是合法路径
    pub pwd: Option<String>,
    /// 当前命令的语义是否为正在调用安装器，缺省为 `false`；请务必正确指定此项，因为这会影响包权限、工作流静态检查等行为。
    //# `call_installer = true`
    pub call_installer: Option<bool>,
    /// 命令等待策略。
    /// `Sync`：同步等待命令执行完成后该步骤才会结束；
    /// `Delay`：异步执行命令并立即完成当前步骤；在当前工作流执行完成时等待该命令执行结束，然后才会结束工作流；
    /// `Abandon`：异步执行命令并立即完成当前步骤；在当前工作流执行完成时若此命令还未结束则直接强行停止此命令。
    //# `wait = "Delay"`
    //* Sync Delay Abandon | Sync
    pub wait: Option<String>,
    /// 是否忽略退出码，缺省则当退出码不为 0 时步骤失败。
    //# `ignore_exit_code = true`
    pub ignore_exit_code: Option<bool>,
}

impl TStep for StepExecute {
    fn run(mut self, cx: &mut WorkflowContext) -> Result<i32> {
        //- 执行自定义命令
        // 配置终端
        let launch_terminal = if cfg!(target_os = "windows") {
            ("cmd", "/c")
        } else {
            ("sh", "-c")
        };

        // 构造执行器
        let mut c = Command::new(launch_terminal.0);
        let c = c.arg(launch_terminal.1);

        // 特殊优化逻辑：如果命令为直接绝对路径则加上双引号，否则无法执行
        let cmd_p = Path::new(&self.command);
        if cmd_p.exists()
            && cmd_p.is_absolute()
            && !(self.command.starts_with('"') || self.command.ends_with('"'))
        {
            self.command = format!("\"{c}\"", c = &self.command);
        }

        let command_str = self.command;

        // 解析命令传入
        let cmd = c.args(split_command(&command_str)?);

        // 指定工作目录
        let workshop = self.pwd.unwrap_or(cx.located.to_owned());
        cmd.current_dir(&workshop);

        // 指定 stdio
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        // 异步执行分流
        let wait = self.wait.unwrap_or("Sync".to_string());
        if wait == *"Sync" {
            // 同步执行并收集结果
            log!("Info(Execute):Running sync command '{command_str}' in '{workshop}'");
            let start_instant = Instant::now();
            let output = cmd.output().map_err(|err| {
                anyhow!("Error(Execute):Command '{command_str}' execution failed : {err}")
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
            let ignore_exit_code = self.ignore_exit_code.unwrap_or(false);
            match output.status.code() {
                Some(val) => {
                    if val == 0 {
                        log!("{level}(Execute):Command '{command_str}' {hint}, output :");
                        println!("{}", read_console(output.stdout));
                    } else {
                        if ignore_exit_code {
                            log!(
                                "Warning(Execute):Ignoring error from failed command '{command_str}' {hint}, output(code={val}) : \n{o}",
                                o = read_console(output.stderr)
                            );
                        } else {
                            log!(
                                "Error(Execute):Failed command '{command_str}' {hint}, output(code={val}) : \n{o}",
                                o = read_console(output.stderr)
                            );
                        }
                        println!("{}", read_console(output.stdout));
                    }
                    Ok(if ignore_exit_code { 0 } else { val })
                }
                None => Err(anyhow!(
                    "Error(Execute):Command '{command_str}' terminated by outer signal"
                )),
            }
        } else {
            // 异步执行
            log!("Info(Execute):Running async command('{wait}') '{command_str}' in '{workshop}'");
            let handler = cmd.spawn().map_err(|e| {
                anyhow!("Error(Execute):Command '{command_str}' spawn failed : {e}")
            })?;
            cx.async_execution_handlers
                .push((command_str, handler, wait == *"Abandon"));

            Ok(0)
        }
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, _fs: &mut MixedFS) -> Vec<String> {
        let mut manifest = Vec::new();

        //@ 若命令调用相对路径的安装包，则该安装包进入装箱单
        // 调用相对路径的安装包
        if self.call_installer.unwrap_or(false) {
            if let Ok(sp_command) = split_command(&self.command) {
                if let Some(exe) = sp_command.first() {
                    if !is_starts_with_inner_value(exe) {
                        manifest.push(exe.to_owned());
                    }
                }
            }
        }

        manifest
    }
}

impl Interpretable for StepExecute {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            command: interpreter(self.command),
            pwd: self.pwd.map(interpreter),
            call_installer: self.call_installer,
            wait: self.wait,
            ignore_exit_code: self.ignore_exit_code,
        }
    }
}

impl Verifiable for StepExecute {
    fn verify_self(&self, _: &String) -> Result<()> {
        // 不得出现反斜杠
        if self.command.contains('\\') {
            return Err(anyhow!("Error(Execute):Backslash (\\) in '{cmd}' is not allowed, use forward slash (/) instead",cmd=&self.command));
        }

        // 校验 pwd 为合法路径
        if let Some(pwd) = &self.pwd {
            values_validator_path(pwd).map_err(|e| {
                anyhow!("Error(Execute):Failed to validate field 'pwd' as valid path : {e}")
            })?;
        }

        // 禁止出现 :/
        let formatted_cmd = format_path(&self.command);
        if formatted_cmd.contains(":/") {
            return Err(anyhow!("Error(Execute):Absolute path in '{formatted_cmd}' is not allowed (keyword ':/' detected), use proper inner values instead"));
        }

        // 校验 wait 枚举值
        if let Some(wait) = &self.wait {
            verify_enum!("Execute", "wait", wait, "Sync" | "Delay" | "Abandon")?;
        }

        // 命令应该是有效的 posix 命令
        split_command(&self.command)?;

        Ok(())
    }
}

impl Generalizable for StepExecute {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        let node = if self.call_installer.unwrap_or(false) {
            Permission {
                key: PermissionKey::execute_installer,
                level: PermissionLevel::Important,
                targets: vec![self.command.to_owned()],
                //@ scene: `call_installer` 为 `true`
            }
        } else {
            Permission {
                key: PermissionKey::execute_custom,
                level: PermissionLevel::Sensitive,
                targets: vec![self.command.to_owned()],
                //@ scene: `call_installer` 为 `false` （缺省）
            }
        };
        Ok(vec![node])
    }
}

#[test]
fn test_execute_validate() {
    let located = String::new();

    let res = StepExecute {
        command: "${AppData}/Installer.exe /S".to_string(),
        pwd: None,
        call_installer: Some(true),
        wait: None,
        ignore_exit_code: None,
    }
    .verify_self(&located);
    assert!(res.is_ok());

    let res = StepExecute {
        command: "C:/Windows/Installer.exe /S".to_string(),
        pwd: None,
        call_installer: Some(true),
        wait: None,
        ignore_exit_code: None,
    }
    .verify_self(&located);
    assert!(res.is_err());
}

#[test]
fn test_execute_manifest() {
    let mut fs = MixedFS::new("examples/Dism++/Dism++".to_string());

    let manifest = StepExecute {
        command: "./Dism++x64.exe /S".to_string(),
        pwd: None,
        call_installer: Some(true),
        wait: None,
        ignore_exit_code: None,
    }
    .get_manifest(&mut fs);
    assert_eq!(manifest, vec!["./Dism++x64.exe".to_string()]);

    let manifest = StepExecute {
        command: "\"./Dism++x64.exe\" /S".to_string(),
        pwd: None,
        call_installer: Some(true),
        wait: None,
        ignore_exit_code: None,
    }
    .get_manifest(&mut fs);
    assert_eq!(manifest, vec!["./Dism++x64.exe".to_string()]);

    let manifest = StepExecute {
        command: "\"${ProgramFiles_X64}/Oray/SunLogin/SunloginClient/SunloginClient.exe\" --mod=uninstall".to_string(),
        pwd: None,
        call_installer: Some(true),
        wait: None,
        ignore_exit_code:None,
    }
    .get_manifest(&mut fs);
    assert!(manifest.is_empty());
}

#[test]
fn test_execute() {
    let mut cx = WorkflowContext::_demo();

    let res = StepExecute {
        command: "echo hello nep ! && echo 你好，尼普！".to_string(),
        pwd: None,
        call_installer: None,
        wait: None,
        ignore_exit_code: None,
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 0);

    let res = StepExecute {
        command: "dir".to_string(),
        pwd: Some("./src".to_string()),
        call_installer: None,
        wait: None,
        ignore_exit_code: None,
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 0);

    let res = StepExecute {
        command: "exit 2".to_string(),
        pwd: None,
        call_installer: None,
        wait: None,
        ignore_exit_code: None,
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 2);

    let res = StepExecute {
        command: "exit 2".to_string(),
        pwd: None,
        call_installer: None,
        wait: None,
        ignore_exit_code: Some(false),
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 2);

    let res = StepExecute {
        command: "exit 0".to_string(),
        pwd: None,
        call_installer: Some(true),
        wait: None,
        ignore_exit_code: None,
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 0);

    let res = StepExecute {
        command: "exit 2".to_string(),
        pwd: None,
        call_installer: Some(true),
        wait: None,
        ignore_exit_code: Some(true),
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 0);
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
        ignore_exit_code: None,
    }
    .run(&mut cx)
    .unwrap();
    StepExecute {
        command: "dir".to_string(),
        pwd: Some("./src".to_string()),
        call_installer: None,
        wait: Some("Delay".to_string()),
        ignore_exit_code: None,
    }
    .run(&mut cx)
    .unwrap();

    let res = StepExecute {
        command: "exit 2".to_string(),
        pwd: None,
        call_installer: None,
        wait: Some("Delay".to_string()),
        ignore_exit_code: None,
    }
    .run(&mut cx)
    .unwrap();
    assert_eq!(res, 0);

    StepLog {
        level: None,
        msg: "running other steps...".to_string(),
    }
    .run(&mut cx)
    .unwrap();

    cx.finish().unwrap();
    println!("Exit");
}
