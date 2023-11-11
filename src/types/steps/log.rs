use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::interpretable::Interpretable;
use crate::types::mixed_fs::MixedFS;
use crate::types::permissions::{Generalizable, Permission};
use crate::types::workflow::WorkflowContext;
use crate::{log, types::verifiable::Verifiable, verify_enum};

use super::TStep;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepLog {
    /// 日志内容。
    pub msg: String,
    /// 日志级别，枚举值：`Debug` `Info` `Warning` `Error` `Success`，缺省为 `Info`。
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

impl Verifiable for StepLog {
    fn verify_self(&self, _: &String) -> Result<()> {
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
    step.verify_self(&String::from("./")).unwrap();
    step.run(&mut cx).unwrap();
}
