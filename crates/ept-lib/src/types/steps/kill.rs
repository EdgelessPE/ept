use super::TStep;
use crate::types::interpretable::Interpretable;
use crate::types::permissions::PermissionKey;
use crate::types::steps::Permission;
use crate::Cfg;
use crate::{
    log,
    types::{
        context::WorkflowContext,
        mixed_fs::MixedFS,
        permissions::{Generalizable, PermissionLevel},
    },
};
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepKill {
    /// 进程名称，注意大小写敏感。
    //# `target = "code.exe"`
    //@ 以 `.exe` 结尾
    pub target: String,
}

fn kill(target: &str) -> Result<()> {
    let s = System::new_all();
    let processes: Vec<_> = s.processes_by_exact_name(target.as_ref()).collect();

    if processes.is_empty() {
        log!("Warning(Kill):No process named '{target}' found.Tip for developer : note that field 'target' is case-sensitive and generally end with '.exe'");
        return Ok(());
    }

    let count_suc = processes.iter().filter(|p| p.kill()).count();
    let count_fail = processes.len() - count_suc;

    let level = if count_fail > 0 { "Warning" } else { "Info" };
    log!(
        "{level}(Kill):Killing '{target}' finished with {count_suc} succeeded, {count_fail} failed"
    );

    Ok(())
}

impl TStep for StepKill {
    fn run(self, _: &mut WorkflowContext) -> Result<i32> {
        //- 杀死某个进程。
        kill(&self.target)?;

        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, _: &mut MixedFS) -> Vec<String> {
        Vec::new()
    }
    fn verify_step(&self, _ctx: &super::VerifyStepCtx) -> Result<()> {
        if !self.target.to_lowercase().ends_with(".exe") {
            log!(
                "Warning(Kill):Generally field 'target' should end with '.exe', got '{t}'",
                t = self.target
            );
        }
        Ok(())
    }
}

impl Interpretable for StepKill {
    fn interpret<F>(self, _: F) -> Self
    where
        F: Fn(String) -> String,
    {
        self
    }
}

impl Generalizable for StepKill {
    fn generalize_permissions(&self, _cfg: &crate::types::context::RuntimeContext) -> Result<Vec<Permission>> {
        Ok(vec![Permission {
            key: PermissionKey::process_kill,
            level: PermissionLevel::Sensitive,
            targets: vec![self.target.clone()],
        }])
    }
}

#[test]
fn test_kill() {
    use crate::types::context::WorkflowContext;
    let mut cx = WorkflowContext::_demo();

    crate::utils::test::_ensure_clear_test_dir();
    std::fs::copy("examples/Notepad/Notepad/notepad.exe", "test/7zGM.exe").unwrap();
    crate::types::steps::StepExecute {
        command: "7zGM.exe".to_string(),
        pwd: Some("test".to_string()),
        call_installer: None,
        wait: Some("Abandon".to_string()),
        ignore_exit_code: None,
    }
    .run(&mut cx)
    .unwrap();

    crate::types::steps::StepWait {
        timeout: 3000,
        break_if: None,
    }
    .run(&mut cx)
    .unwrap();

    StepKill {
        target: "7zGM.exe".to_string(),
    }
    .run(&mut cx)
    .unwrap();
}

#[test]
fn test_kill_corelation() {
    let mut cx = WorkflowContext::_demo();
    let mut mixed_fs = MixedFS::new("");

    // 反向工作流
    StepKill {
        target: "windows".to_string(),
    }
    .reverse_run(&mut cx)
    .unwrap();

    // 装箱单
    assert!(StepKill {
        target: "code.exe".to_string(),
    }
    .get_manifest(&mut mixed_fs)
    .is_empty());

    // 解释
    assert_eq!(
        StepKill {
            target: "${Home}".to_string(),
        }
        .interpret(|s| s.replace("${Home}", "C:/Users/Nep")),
        StepKill {
            target: "${Home}".to_string(),
        }
    );

    // 校验
    let ctx = crate::types::steps::VerifyStepCtx::_demo();
    assert!(StepKill {
        target: "code.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_ok());
    assert!(StepKill {
        target: "code".to_string(),
    }
    .verify_step(&ctx)
    .is_ok());
}
