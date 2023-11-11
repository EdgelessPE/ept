use super::TStep;
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
