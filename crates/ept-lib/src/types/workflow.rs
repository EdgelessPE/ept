use super::context::{VerifiableCtx, VerifyStepCtx};
use super::{permissions::Generalizable, steps::Step, verifiable::Verifiable};
use crate::types::context::RuntimeContext;
use crate::types::permissions::Permission;
use crate::utils::conditions::{get_permissions_from_conditions, verify_conditions};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkflowHeader {
    /// 步骤名称，缺省使用步骤键的 sentence case。
    //# ```toml
    //# name = "创建快捷方式"
    //# ```
    pub name: Option<String>,
    /// 步骤类型
    //@ 必须是步骤定义中的一种值。
    //# ```toml
    //# step = "Link"
    //# ```
    pub step: String,
    #[serde(rename = "if")]
    /// 步骤执行条件。
    //@ 是合法的条件。
    //# ```toml
    //# if = "Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")"
    //# ```
    pub c_if: Option<String>,
}

impl WorkflowHeader {
    // 收集 header 中的条件语句
    fn get_conditions(&self) -> Vec<String> {
        let mut conditions = Vec::new();
        if let Some(c_if) = &self.c_if {
            conditions.push(c_if.to_owned());
        }
        conditions
    }
}

impl Generalizable for WorkflowHeader {
    fn generalize_permissions(&self, ctx: &RuntimeContext) -> Result<Vec<Permission>> {
        // 获取条件语句所需的权限
        get_permissions_from_conditions(ctx, self.get_conditions())
    }
}

impl Verifiable for WorkflowHeader {
    fn verify_self(&self, cx: &VerifiableCtx) -> Result<()> {
        // 校验条件，使用上下文中的配置
        verify_conditions(
            cx.runtime_ctx,
            self.get_conditions(),
            &cx.mixed_fs.located,
            "1.0.0.0",
        )
    }
}

#[test]
fn test_header_perm() {
    use crate::types::permissions::{PermissionKey, PermissionLevel};
    let flow=WorkflowHeader{
        name: Some("Name".to_string()),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };
    let res = flow
        .generalize_permissions(&crate::utils::test::_default_test_cfg())
        .unwrap();
    assert_eq!(
        res,
        vec![
            Permission {
                key: PermissionKey::fs_read,
                level: PermissionLevel::Normal,
                targets: vec!["./mc/vsc.exe".to_string(),],
            },
            Permission {
                key: PermissionKey::fs_read,
                level: PermissionLevel::Sensitive,
                targets: vec!["${SystemDrive}/Windows".to_string(),],
            },
            Permission {
                key: PermissionKey::fs_read,
                level: PermissionLevel::Sensitive,
                targets: vec!["${AppData}/Roaming/Edgeless/ept".to_string(),],
            },
        ]
    )
}

#[test]
fn test_header_valid() {
    use crate::utils::test::_default_test_cfg;

    let flow=WorkflowHeader{
        name: Some("Name".to_string()),
        step: "Step".to_string(),
        c_if: Some("Exist(\"./mc/vsc.exe\") && IsDirectory(\"${SystemDrive}/Windows\") || Exist(\"${AppData}/Roaming/Edgeless/ept\")".to_string()),
    };
    use crate::types::mixed_fs::MixedFS;
    let mixed_fs = MixedFS::new("./examples/VSCode");
    let cx = VerifiableCtx {
        mixed_fs: &mixed_fs,
        runtime_ctx: &_default_test_cfg(),
    };

    flow.verify_self(&cx).unwrap();

    let flow = WorkflowHeader {
        name: Some("Name".to_string()),
        step: "Step".to_string(),
        c_if: Some("${Arch}==\"X64\"".to_string()),
    };

    let ctx2 = VerifiableCtx {
        mixed_fs: &mixed_fs,
        runtime_ctx: &_default_test_cfg(),
    };
    assert!(flow.verify_self(&ctx2).is_err());
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkflowNode {
    pub header: WorkflowHeader,
    pub body: Step,
}

impl Generalizable for WorkflowNode {
    fn generalize_permissions(&self, ctx: &RuntimeContext) -> Result<Vec<Permission>> {
        let mut perm = Vec::new();
        perm.append(&mut self.header.generalize_permissions(ctx)?);
        perm.append(&mut self.body.generalize_permissions(ctx)?);

        Ok(perm)
    }
}

impl WorkflowNode {
    pub fn verify_step(&self, cx: &VerifyStepCtx) -> Result<()> {
        let verifiable_ctx = VerifiableCtx {
            mixed_fs: cx.mixed_fs,
            runtime_ctx: cx.runtime_ctx,
        };
        self.header.verify_self(&verifiable_ctx)?;
        self.body.verify_step(cx)
    }
}
