use anyhow::{Result, Ok};
use serde::{Deserialize, Serialize};
use std::{thread::sleep, time::Duration};
use crate::types::{verifiable::Verifiable, permissions::Generalizable, workflow::WorkflowContext, mixed_fs::MixedFS};

use super::TStep;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepWait{
    pub timeout:u64,
}


impl TStep for StepWait {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        let d = Duration::from_millis(self.timeout);
        sleep(d);

        Ok(0)
    }
    fn reverse_run(self, cx: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        Vec::new()
    }
    fn interpret<F>(self, interpreter: F) -> Self
        where
            F: Fn(String) -> String {
        self
    }
}

impl Verifiable for StepWait {
    fn verify_self(&self, located: &String) -> Result<()> {
        Ok(())
    }
}

impl Generalizable for StepWait {
    fn generalize_permissions(&self) -> Result<Vec<crate::types::permissions::Permission>> {
        Ok(Vec::new())
    }
}

#[test]
fn test_wait(){
    use crate::types::package::GlobalPackage;
    use crate::types::workflow::WorkflowContext;
    use std::time::Instant;
    envmnt::set("DEBUG", "true");
    let mut cx=WorkflowContext { located: String::from("D:/Desktop/Projects/EdgelessPE/ept"), pkg: GlobalPackage::_demo() };

    let d = Duration::from_millis(3000);
    let now = Instant::now();

    StepWait{
        timeout:3000,
    }.run(&mut cx).unwrap();

    assert!(now.elapsed() >= d);
}