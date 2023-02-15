use crate::utils::log;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::TStep;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepLog {
    pub level: String,
    pub msg: String,
}

impl TStep for StepLog {
    fn run(self, _: &String) -> Result<i32> {
        log(format!("{}(Log):{}", &self.level, &self.msg));
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

#[test]
fn test_log() {
    StepLog {
        level: String::from("Debug"),
        msg: String::from("Hello nep!"),
    }
    .run(&String::from(
        "D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode",
    ))
    .unwrap();
}
