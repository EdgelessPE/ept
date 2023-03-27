use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{log, types::Verifiable, verify_enum};
use crate::types::permissions::{Generalizable, Permission};

use super::TStep;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepLog {
    pub level: String,
    pub msg: String,
}

impl TStep for StepLog {
    fn run(self, _: &String) -> Result<i32> {
        log!("{}(Log):{}", &self.level, &self.msg);
        Ok(0)
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
            level: self.level,
            msg: interpreter(self.msg),
        }
    }
}

impl Verifiable for StepLog {
    fn verify_self(&self) -> Result<()> {
        verify_enum!(
            "Log",
            "level",
            self.level,
            "Debug" | "Info" | "Warning" | "Error" | "Success"
        )
    }
}

impl Generalizable for StepLog {
    fn generalize_permissions(&self)->Result<Vec<Permission>> {
        Ok(vec![])
    }
}

#[test]
fn test_log() {
    let step = StepLog {
        level: String::from("Info"),
        msg: String::from("Hello nep!"),
    };
    step.verify_self().unwrap();
    step.run(&String::from(
        "D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode",
    ))
    .unwrap();
}
