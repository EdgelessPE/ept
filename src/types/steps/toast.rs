use super::TStep;
use crate::log;
use crate::types::interpretable::Interpretable;
use crate::types::permissions::PermissionKey;
use crate::types::steps::Permission;
use crate::types::{
    mixed_fs::MixedFS,
    permissions::{Generalizable, PermissionLevel},
    verifiable::Verifiable,
    workflow::WorkflowContext,
};
use anyhow::{anyhow, Ok, Result};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};

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
        Notification::new()
            .appname("ept")
            .summary(&self.title)
            .body(&self.content)
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

impl Verifiable for StepToast {
    fn verify_self(&self, _: &String) -> Result<()> {
        Ok(())
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
    envmnt::set("DEBUG", "true");
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
    let mut mixed_fs = MixedFS::new("".to_string());

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

    // 解释
    assert_eq!(StepToast{
        title:"${Home}".to_string(),
        content:"${SystemDrive}".to_string()
    }.interpret(|s|s.replace("${Home}", "C:/Users/Nep").replace("${SystemDrive}", "C:")),StepToast{
        title:"C:/Users/Nep".to_string(),
        content:"C:".to_string()
    })
}
