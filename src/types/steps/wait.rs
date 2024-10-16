use super::TStep;
use crate::executor::condition_eval;
use crate::log;
use crate::types::interpretable::Interpretable;
use crate::types::steps::Permission;
use crate::types::{mixed_fs::MixedFS, permissions::Generalizable, workflow::WorkflowContext};
use crate::utils::conditions::{get_permissions_from_conditions, verify_conditions};
use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::{thread::sleep, time::Duration};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepWait {
    /// 等待的时长，单位为 ms。
    //# `timeout = "3000"`
    //@ 不超过 30min（1800000ms）
    pub timeout: u64,
    /// 若满足指定条件则提前结束等待，该条件会在等待过程中每 500ms 检查一次。
    //# `break_if = 'Exist("${Desktop}/Visual Studio Code.lnk")'`
    //@ 是合法的条件
    pub break_if: Option<String>,
}

impl TStep for StepWait {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        //- 等待一个指定的时间。
        let d = Duration::from_millis(self.timeout);
        let step_d = Duration::from_millis(500);

        // 处理 break_if 条件
        if let Some(cond) = self.break_if {
            if d <= step_d {
                sleep(d);
            } else {
                let start_instant = Instant::now();
                // 每 500ms 检查一次条件是否成立
                log!(
                    "Info(Wait):Waiting for '{}' ms with break condition '{cond}'...",
                    self.timeout
                );
                loop {
                    sleep(step_d);
                    if start_instant.elapsed() >= d
                        || condition_eval(
                            &cond,
                            cx.exit_code,
                            &cx.located,
                            &cx.pkg.package.version,
                        )?
                    {
                        break;
                    }
                }
                // 最终检查一次条件并配置 ExitCode
                return if condition_eval(&cond, cx.exit_code, &cx.located, &cx.pkg.package.version)?
                {
                    Ok(0)
                } else {
                    Ok(1)
                };
            }
        } else {
            log!("Info(Wait):Start to wait for '{}' ms...", self.timeout);
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
    fn verify_step(&self, ctx: &super::VerifyStepCtx) -> Result<()> {
        let located = &ctx.mixed_fs.located;
        // timeout 时间应当小于等于 30min
        if self.timeout > (30 * 60 * 1000) {
            return Err(anyhow!(
                "Error(Wait):Timeout should not be longer than 30 min, got '{}'",
                &self.timeout
            ));
        }

        // 校验跳出条件
        if let Some(cond) = &self.break_if {
            verify_conditions(vec![cond.to_owned()], located, &"1.0.0.0".to_string())
                .map_err(|e| anyhow!("Error(Wait):Failed to valid field 'break_if' : {e}"))?;
        }

        Ok(())
    }
}

impl Interpretable for StepWait {
    fn interpret<F>(self, _: F) -> Self
    where
        F: Fn(String) -> String,
    {
        self
    }
}

impl Generalizable for StepWait {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        let mut permissions = Vec::new();

        if let Some(cond) = &self.break_if {
            let mut cond_permissions = get_permissions_from_conditions(vec![cond.to_owned()])?;
            permissions.append(&mut cond_permissions);
        }
        //@ key: 由 `break_if` 条件语句产生
        //@ level: 由 `break_if` 条件语句产生
        //@ targets: 由 `break_if` 条件语句产生
        //@ scene: 配置了 `break_if` 时

        Ok(permissions)
    }
}

#[test]
fn test_wait() {
    use crate::types::workflow::WorkflowContext;
    use crate::utils::flags::{set_flag, Flag};
    set_flag(Flag::Debug, true);
    let mut cx = WorkflowContext::_demo();

    // 测试普通等待
    let d = Duration::from_millis(1000);
    let now = Instant::now();

    StepWait {
        timeout: 1000,
        break_if: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(now.elapsed() >= d);

    // 测试小于 500ms 的普通等待
    let now = Instant::now();

    StepWait {
        timeout: 200,
        break_if: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(
        Duration::from_millis(200) <= now.elapsed() && now.elapsed() <= Duration::from_millis(300)
    );

    // 测试恒假等待
    let d = Duration::from_secs(2);
    let now = Instant::now();

    StepWait {
        timeout: 2000,
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
    assert!(Duration::from_millis(500) <= elapsed && elapsed <= Duration::from_millis(600));

    // 测试过短条件等待
    let d = Duration::from_millis(200);
    let now = Instant::now();

    StepWait {
        timeout: 200,
        break_if: None,
    }
    .run(&mut cx)
    .unwrap();
    let e = now.elapsed();
    assert!(e >= d && e <= Duration::from_millis(500));
}

#[test]
fn test_wait_corelation() {
    let mut cx = WorkflowContext::_demo();
    let mut mixed_fs = MixedFS::new("");
    // 反向工作流
    StepWait {
        timeout: 1000,
        break_if: None,
    }
    .reverse_run(&mut cx)
    .unwrap();

    // 装箱单
    assert!(StepWait {
        timeout: 1000,
        break_if: None,
    }
    .get_manifest(&mut mixed_fs)
    .is_empty());

    // 解释
    assert_eq!(
        StepWait {
            timeout: 100,
            break_if: None
        }
        .interpret(|s| s.replace("${Home}", "C:/Users/Nep")),
        StepWait {
            timeout: 100,
            break_if: None
        }
    );

    // 校验
    let ctx = crate::types::steps::VerifyStepCtx::_demo();
    assert!(StepWait {
        timeout: 30 * 60 * 1000,
        break_if: Some("ExitCode == 1".to_string())
    }
    .verify_step(&ctx)
    .is_ok());
    assert!(StepWait {
        timeout: 30 * 60 * 1000 + 1,
        break_if: None
    }
    .verify_step(&ctx)
    .is_err());
    assert!(StepWait {
        timeout: 100,
        break_if: Some("Exit == 0".to_string())
    }
    .verify_step(&ctx)
    .is_err());

    // 生成权限
    assert_eq!(
        StepWait {
            timeout: 100,
            break_if: Some("Exist(\"${SystemDrive}:/test\")".to_string())
        }
        .generalize_permissions()
        .unwrap(),
        vec![Permission {
            key: crate::types::permissions::PermissionKey::fs_read,
            level: crate::types::permissions::PermissionLevel::Sensitive,
            targets: vec!["${SystemDrive}:/test".to_string()]
        }]
    );
}
