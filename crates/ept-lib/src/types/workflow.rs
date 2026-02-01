use std::{env::current_dir, process::Child};

use super::cfg::Cfg;
use super::steps::VerifyStepCtx;
use super::{
    package::GlobalPackage,
    permissions::Generalizable,
    steps::Step,
    verifiable::{Verifiable, VerifiableCtx},
};
use crate::log;
use crate::utils::test::_default_test_cfg;
use crate::utils::{
    conditions::{get_permissions_from_conditions, verify_conditions},
    term::read_console,
};
use crate::{p2s, types::permissions::Permission};
use anyhow::{anyhow, Result};
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
    fn generalize_permissions(&self, cfg: &Cfg) -> Result<Vec<Permission>> {
        // 获取条件语句所需的权限
        get_permissions_from_conditions(cfg, self.get_conditions())
    }
}

impl Verifiable for WorkflowHeader {
    fn verify_self(&self, ctx: &VerifiableCtx) -> Result<()> {
        // 校验条件，使用上下文中的配置
        verify_conditions(
            ctx.cfg,
            self.get_conditions(),
            &ctx.mixed_fs.located,
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
    let ctx = VerifiableCtx {
        mixed_fs: &mixed_fs,
        cfg: &_default_test_cfg(),
    };

    flow.verify_self(&ctx).unwrap();

    let flow = WorkflowHeader {
        name: Some("Name".to_string()),
        step: "Step".to_string(),
        c_if: Some("${Arch}==\"X64\"".to_string()),
    };

    let ctx2 = VerifiableCtx {
        mixed_fs: &mixed_fs,
        cfg: &_default_test_cfg(),
    };
    assert!(flow.verify_self(&ctx2).is_err());
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkflowNode {
    pub header: WorkflowHeader,
    pub body: Step,
}

impl Generalizable for WorkflowNode {
    fn generalize_permissions(&self, cfg: &Cfg) -> Result<Vec<Permission>> {
        let mut perm = Vec::new();
        perm.append(&mut self.header.generalize_permissions(cfg)?);
        perm.append(&mut self.body.generalize_permissions(cfg)?);

        Ok(perm)
    }
}

impl WorkflowNode {
    pub fn verify_step(&self, ctx: &VerifyStepCtx) -> Result<()> {
        let verifiable_ctx = VerifiableCtx {
            mixed_fs: &ctx.mixed_fs,
            cfg: &ctx.cfg,
        };
        self.header.verify_self(&verifiable_ctx)?;
        self.body.verify_step(ctx)
    }
}

pub struct WorkflowContext {
    pub located: String,
    pub pkg: GlobalPackage,
    pub async_execution_handlers: Vec<(String, Child, bool)>, // 命令，handler，是否被抛弃
    pub exit_code: i32,
    pub cfg: Cfg,
}

impl WorkflowContext {
    pub fn _demo() -> Self {
        Self::new(
            _default_test_cfg(),
            &p2s!(current_dir().unwrap()),
            GlobalPackage::_demo(),
        )
    }

    pub fn new(cfg: Cfg, located: &str, pkg: GlobalPackage) -> Self {
        Self {
            pkg,
            located: located.to_owned(),
            async_execution_handlers: Vec::new(),
            exit_code: 0,
            cfg,
        }
    }

    pub fn finish(self) -> Result<i32> {
        log!("Debug:Finish context");

        // 等待异步 handlers
        for (cmd, mut handler, abandon) in self.async_execution_handlers {
            if abandon {
                if let Err(e) = handler.kill() {
                    log!("Warning(Execute):Failed to kill async abandoned command '{cmd}' : {e}");
                } else {
                    log!("Info(Execute):Killed async abandoned command '{cmd}'");
                }
            } else {
                let output = handler.wait_with_output().map_err(|e| {
                    anyhow!("Error(Execute):Failed to wait on async command '{cmd}' : {e}")
                })?;
                // 处理退出码
                match output.status.code() {
                    Some(val) => {
                        if val == 0 {
                            log!("Info(Execute):Async command '{cmd}' output :");
                            println!("{output}", output = read_console(output.stdout));
                        } else {
                            log!("Error(Execute):Async command '{cmd}' failed, output :");
                            println!("{output}", output = read_console(output.stdout));
                        }
                    }
                    None => {
                        log!("Error(Execute):Async command '{cmd}' terminated by signal");
                    }
                }
            }
        }

        Ok(self.exit_code)
    }
}
