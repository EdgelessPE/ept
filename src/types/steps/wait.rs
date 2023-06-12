use crate::types::{
    mixed_fs::MixedFS, permissions::Generalizable, verifiable::Verifiable,
    workflow::WorkflowContext,
};
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::{thread::sleep, time::Duration};

use super::TStep;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepWait {
    pub timeout: u64,
}

impl TStep for StepWait {
    fn run(self, _: &mut WorkflowContext) -> Result<i32> {
        let d = Duration::from_millis(self.timeout);
        sleep(d);

        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, _: &mut MixedFS) -> Vec<String> {
        Vec::new()
    }
    fn interpret<F>(self, _: F) -> Self
    where
        F: Fn(String) -> String,
    {
        self
    }
}

impl Verifiable for StepWait {
    fn verify_self(&self, _: &String) -> Result<()> {
        Ok(())
    }
}

impl Generalizable for StepWait {
    fn generalize_permissions(&self) -> Result<Vec<crate::types::permissions::Permission>> {
        Ok(Vec::new())
    }
}

#[test]
fn test_wait() {
    use crate::types::package::GlobalPackage;
    use crate::types::workflow::WorkflowContext;
    use std::time::Instant;
    envmnt::set("DEBUG", "true");
    let mut cx = WorkflowContext {
        located: String::from("D:/Desktop/Projects/EdgelessPE/ept"),
        pkg: GlobalPackage::_demo(),
    };

    let d = Duration::from_millis(3000);
    let now = Instant::now();

    StepWait { timeout: 3000 }.run(&mut cx).unwrap();

    assert!(now.elapsed() >= d);
}
