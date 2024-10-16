use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::interpretable::Interpretable;
use crate::types::mixed_fs::MixedFS;
use crate::types::permissions::{Generalizable, Permission};
use crate::types::workflow::WorkflowContext;
use crate::{log, verify_enum};

use super::TStep;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepLog {
    /// 日志内容。
    //# `msg = "VSCode installed successfully, workflow exit."`
    pub msg: String,
    /// 日志级别。
    //*  Debug Info Warning Error Success | Info
    //# `level = "Success"`
    pub level: Option<String>,
}

impl TStep for StepLog {
    fn run(self, _: &mut WorkflowContext) -> Result<i32> {
        //- 打印日志。
        let level = self.level.unwrap_or("Info".to_string());
        log!("{level}(Log):{m}", m = self.msg);
        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, _fs: &mut MixedFS) -> Vec<String> {
        Vec::new()
    }
    fn verify_step(&self, _ctx: &super::VerifyStepCtx) -> Result<()> {
        if let Some(level) = &self.level {
            verify_enum!(
                "Log",
                "level",
                level,
                "Debug" | "Info" | "Warning" | "Error" | "Success"
            )?;
        }
        Ok(())
    }
}

impl Interpretable for StepLog {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            level: self.level,
            msg: interpreter(self.msg),
        }
    }
}

impl Generalizable for StepLog {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![])
    }
}

#[test]
fn test_log() {
    let mut cx = WorkflowContext::_demo();
    let step = StepLog {
        level: Some(String::from("Info")),
        msg: String::from("Hello nep!"),
    };
    let ctx = crate::types::steps::VerifyStepCtx::_demo();
    step.verify_step(&ctx).unwrap();
    step.run(&mut cx).unwrap();
}

#[test]
fn test_log_corelation() {
    let mut cx = WorkflowContext::_demo();
    let mut mixed_fs = MixedFS::new("");
    // 反向工作流
    StepLog {
        level: Some(String::from("Info")),
        msg: String::from("Hello nep!"),
    }
    .reverse_run(&mut cx)
    .unwrap();

    // 装箱单
    assert!(StepLog {
        level: Some(String::from("Info")),
        msg: String::from("Hello nep!"),
    }
    .get_manifest(&mut mixed_fs)
    .is_empty());

    // 权限
    assert!(StepLog {
        level: Some(String::from("Info")),
        msg: String::from("Hello nep!"),
    }
    .generalize_permissions()
    .unwrap()
    .is_empty());
}
