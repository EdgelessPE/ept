use super::TStep;
use crate::log;
use crate::types::interpretable::Interpretable;
use crate::types::permissions::PermissionKey;
use crate::types::steps::Permission;
use crate::types::{
    mixed_fs::MixedFS,
    permissions::{Generalizable, PermissionLevel},
    workflow::WorkflowContext,
};
use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};
use winrt_notification::{Duration, Sound, Toast};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepToast {
    /// 消息标题。
    //# `title = "你好 Nep"`
    pub title: String,
    /// 消息内容。
    //# `content = "Hello Nep"`
    pub content: String,
}

impl TStep for StepToast {
    fn run(self, _: &mut WorkflowContext) -> Result<i32> {
        //- 弹出消息通知。
        Toast::new(Toast::POWERSHELL_APP_ID)
            .title(&self.title)
            .text1(&self.content)
            .sound(Some(Sound::SMS))
            .duration(Duration::Short)
            .show()
            .map_err(|e| {
                anyhow!(
                    "Error(Toast):Failed to send toast : '{e}' (title : '{t}', content : '{c}')",
                    t = self.title,
                    c = self.content
                )
            })?;

        log!(
            "Log(Toast):Sent toast with title : '{t}', content : '{c}'",
            t = self.title,
            c = self.content
        );
        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, _: &mut MixedFS) -> Vec<String> {
        Vec::new()
    }
    fn verify_step(&self, _ctx: &super::VerifyStepCtx) -> Result<()> {
        Ok(())
    }
}

impl Interpretable for StepToast {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            title: interpreter(self.title),
            content: interpreter(self.content),
        }
    }
}

impl Generalizable for StepToast {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![Permission {
            key: PermissionKey::notify_toast,
            level: PermissionLevel::Normal,
            targets: vec![self.title.clone()],
        }])
    }
}

#[test]
fn test_toast() {
    use crate::types::workflow::WorkflowContext;
    use crate::utils::flags::{set_flag, Flag};
    set_flag(Flag::Debug, true);
    let mut cx = WorkflowContext::_demo();

    StepToast {
        title: "测试标题😘".to_string(),
        content: "Hey, love from ept\n你好，爱来自乙烯丙烯三元聚合物".to_string(),
    }
    .run(&mut cx)
    .unwrap();
}

#[test]
fn test_toast_corelation() {
    use crate::types::workflow::WorkflowContext;
    let mut cx = WorkflowContext::_demo();
    let mut mixed_fs = MixedFS::new("");

    // 反向工作流
    StepToast {
        title: "测试标题😘".to_string(),
        content: "Hey, love from ept\n你好，爱来自乙烯丙烯三元聚合物".to_string(),
    }
    .reverse_run(&mut cx)
    .unwrap();

    // 装箱单
    assert!(StepToast {
        title: "测试标题😘".to_string(),
        content: "Hey, love from ept\n你好，爱来自乙烯丙烯三元聚合物".to_string(),
    }
    .get_manifest(&mut mixed_fs)
    .is_empty());

    // 校验
    assert!(StepToast {
        title: "测试标题😘".to_string(),
        content: "Hey, love from ept\n你好，爱来自乙烯丙烯三元聚合物".to_string(),
    }
    .verify_step(&super::VerifyStepCtx {
        mixed_fs: MixedFS::new(""),
        is_expand_flow: false,
    })
    .is_ok());

    // 解释
    assert_eq!(
        StepToast {
            title: "${Home}".to_string(),
            content: "${SystemDrive}".to_string()
        }
        .interpret(|s| s
            .replace("${Home}", "C:/Users/Nep")
            .replace("${SystemDrive}", "C:")),
        StepToast {
            title: "C:/Users/Nep".to_string(),
            content: "C:".to_string()
        }
    )
}
