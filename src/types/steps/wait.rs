use super::TStep;
use crate::executor::condition_eval;
use crate::log;
use crate::types::steps::Permission;
use crate::types::{
    mixed_fs::MixedFS, permissions::Generalizable, verifiable::Verifiable,
    workflow::WorkflowContext,
};
use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::{thread::sleep, time::Duration};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepWait {
    pub timeout: u64,
    pub break_if: Option<String>,
}

impl TStep for StepWait {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        let d = Duration::from_millis(self.timeout);
        let step_d = Duration::from_millis(500);

        // 处理 break_if 条件
        if let Some(cond) = self.break_if {
            if d <= step_d {
                sleep(d);
            } else {
                let start_instant = Instant::now();
                // 每 500ms 检查一次条件是否成立
                log!("Info(Wait):Waiting with break condition '{cond}'...");
                loop {
                    sleep(step_d);
                    // TODO:exit_code需要通过上下文获取
                    if start_instant.elapsed() >= d
                        || condition_eval(&cond, cx.exit_code, &cx.located)?
                    {
                        break;
                    }
                }
                // 最终检查一次条件并配置 ExitCode
                return if condition_eval(&cond, cx.exit_code, &cx.located)? {
                    Ok(0)
                } else {
                    Ok(1)
                };
            }
        } else {
            sleep(d);
        }
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
    fn verify_self(&self, located: &String) -> Result<()> {
        // timeout 时间应当小于等于 30min
        if &self.timeout > &(30 * 60 * 1000) {
            return Err(anyhow!(
                "Error:Timeout should not be longer than 30 min, got '{}'",
                &self.timeout
            ));
        }

        // 测试跳出条件 eval
        if let Some(cond) = &self.break_if {
            condition_eval(cond, 0, located)?;
        }

        Ok(())
    }
}

impl Generalizable for StepWait {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(Vec::new())
    }
}

#[test]
fn test_wait() {
    use crate::types::workflow::WorkflowContext;
    envmnt::set("DEBUG", "true");
    let mut cx = WorkflowContext::_demo();

    // 测试普通等待
    let d = Duration::from_millis(3000);
    let now = Instant::now();

    StepWait {
        timeout: 3000,
        break_if: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(now.elapsed() >= d);

    // 测试恒假等待
    let d = Duration::from_secs(5);
    let now = Instant::now();

    StepWait {
        timeout: 5000,
        break_if: Some("Exist(\"${ExitCode}.err\")".to_string()),
    }
    .run(&mut cx)
    .unwrap();

    assert!(now.elapsed() >= d);

    // 测试恒真等待
    let now = Instant::now();
    cx.exit_code = 1;
    StepWait {
        timeout: 5000,
        break_if: Some("${ExitCode}==1".to_string()),
    }
    .run(&mut cx)
    .unwrap();

    let elapsed = now.elapsed();
    assert!(Duration::from_millis(500) <= elapsed && elapsed <= Duration::from_millis(550));
}
