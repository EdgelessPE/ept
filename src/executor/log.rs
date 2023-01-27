use crate::{types::StepLog, utils::log};
use anyhow::Result;

pub fn step_log(step: StepLog) -> Result<i32> {
    let mut msg = String::from(&step.level);
    msg += ":";
    msg += &step.msg;
    log(msg);
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
