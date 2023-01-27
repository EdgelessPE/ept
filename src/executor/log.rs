use crate::{types::StepLog, utils::log};
use anyhow::Result;

pub fn step_log(step: StepLog) -> Result<i32> {
    log(format!("{}(Log):{}",&step.level,&step.msg));
    Ok(0)
}

#[test]
fn test_log() {
    let res = step_log(StepLog {
        level: String::from("Debug"),
        msg: String::from("Hello nep!"),
    });
    println!("{:?}", res);
}
