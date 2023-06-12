use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::mixed_fs::MixedFS;
use crate::types::permissions::{Generalizable, Permission};
use crate::types::workflow::WorkflowContext;
use crate::{log, types::verifiable::Verifiable, verify_enum};

use super::TStep;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepLog {
    pub level: String,
    pub msg: String,
}

impl TStep for StepLog {
    fn run(self, _: &mut WorkflowContext) -> Result<i32> {
        log!("{l}(Log):{m}", l = self.level, m = self.msg);
        Ok(0)
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
            level: self.level,
            msg: interpreter(self.msg),
        }
    }
}

impl Verifiable for StepLog {
    fn verify_self(&self, _: &String) -> Result<()> {
        verify_enum!(
            "Log",
            "level",
            self.level,
            "Debug" | "Info" | "Warning" | "Error" | "Success"
        )
    }
}

impl Generalizable for StepLog {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![])
    }
}

#[test]
fn test_log() {
    use crate::types::package::GlobalPackage;
    let mut cx = WorkflowContext {
        pkg: GlobalPackage::_demo(),
        located: String::from("./apps/VSCode"),
    };
    let step = StepLog {
        level: String::from("Info"),
        msg: String::from("Hello nep!"),
    };
    step.verify_self(&String::from(
        "./",
    ))
    .unwrap();
    step.run(&mut cx).unwrap();
}
