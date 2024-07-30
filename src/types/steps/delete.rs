use std::{ffi::OsString, path::Path};

use super::TStep;
use crate::types::interpretable::Interpretable;
use crate::types::permissions::PermissionKey;
use crate::types::steps::Permission;
use crate::{
    executor::{judge_perm_level, values_validator_path},
    log, p2s,
    types::{
        mixed_fs::MixedFS, permissions::Generalizable, verifiable::Verifiable,
        workflow::WorkflowContext,
    },
    utils::{
        fs::try_recycle,
        wild_match::{contains_wild_match, parse_wild_match},
    },
};
use anyhow::{anyhow, Result};
use force_delete_win::force_delete_file_folder;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepDelete {
    /// 删除目标路径，支持相对路径和绝对路径，支持使用通配符。
    //# ```toml
    //# # 相对路径写法
    //# at = "./eula.txt"
    //#
    //# # 绝对路径通配符写法
    //# at = "${AppData}/vscode/*.txt"
    //# ```
    //@ 是合法路径
    //@ 符合通配符用法
    pub at: String,
    /// 是否强制删除，缺省为 `false`。
    //# `force = true`
    pub force: Option<bool>,
}

fn delete(target: &String, force: bool) -> Result<()> {
    let p = Path::new(target);
    if !p.exists() {
        log!("Warning(Delete):Target '{target}' not exist, skip deleting");
        return Ok(());
    }
    if let Err(e) = try_recycle(p) {
        if force {
            if force_delete_file_folder(OsString::from(target)) {
                log!("Warning(Delete):Force deleted '{target}'");
                Ok(())
            } else {
                Err(anyhow!(
                    "Error(Delete):Failed to force delete '{target}' : '{e}'"
                ))
            }
        } else {
            Err(anyhow!("Error(Delete):Failed to delete '{target}' : '{e}', enable field 'force' to try shredding"))
        }
    } else {
        log!("Debug(Delete):Deleted '{target}' by moving to recycle bin");
        Ok(())
    }
}

impl TStep for StepDelete {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        //- 删除文件/文件夹。
        let force = self.force.unwrap_or(false);
        if contains_wild_match(&self.at) {
            for target in parse_wild_match(self.at, &cx.located)? {
                delete(&p2s!(target), force)?;
            }
        } else {
            delete(&self.at, force)?;
        }

        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        fs.remove(&self.at);
        Vec::new()
    }
}

impl Interpretable for StepDelete {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            at: interpreter(self.at),
            force: self.force,
        }
    }
}

impl Verifiable for StepDelete {
    fn verify_self(&self, _: &String) -> Result<()> {
        values_validator_path(&self.at).map_err(|e| {
            anyhow!("Error(Delete):Failed to validate field 'at' as valid path : {e}")
        })?;

        Ok(())
    }
}

impl Generalizable for StepDelete {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![Permission {
            key: PermissionKey::fs_write,
            level: judge_perm_level(&self.at)?,
            targets: vec![self.at.clone()],
        }])
    }
}

#[test]
fn test_delete() {
    envmnt::set("DEBUG", "true");
    let mut cx = WorkflowContext::_demo();
    crate::utils::test::_ensure_clear_test_dir();

    // 准备源
    crate::utils::fs::copy_dir("src", "test/src").unwrap();

    // 普通删除
    StepDelete {
        at: "test/src/main.rs".to_string(),
        force: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(!Path::new("test/src/main.rs").exists());

    // 强制删除
    // TODO:找一个强制占用逻辑用于测试
    StepDelete {
        at: "test/src/utils/mod.rs".to_string(),
        force: Some(true),
    }
    .run(&mut cx)
    .unwrap();
    assert!(!Path::new("test/src/utils/mod.rs").exists());

    // 删除不存在的文件
    StepDelete {
        at: "test/src/utils/mod.rs".to_string(),
        force: Some(true),
    }
    .run(&mut cx)
    .unwrap();

    // 普通通配删除
    StepDelete {
        at: "test/src/types/steps/*.rs".to_string(),
        force: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(!Path::new("test/src/types/steps/delete.rs").exists());
    assert!(Path::new("test/src/types/steps/README.md").exists());

    // 强制通配删除
    // TODO:找一个强制占用逻辑用于测试
    StepDelete {
        at: "test/src/entrances/*".to_string(),
        force: Some(true),
    }
    .run(&mut cx)
    .unwrap();
    assert!(!Path::new("test/src/entrances/utils/mod.rs").exists());
    assert!(!Path::new("test/src/entrances/install.rs").exists());
}

#[test]
fn test_delete_corelation() {
    let mut cx = WorkflowContext::_demo();
    let mut mixed_fs = MixedFS::new("".to_string());

    // 反向工作流
    StepDelete {
        at: "test/1.rs".to_string(),
        force: None,
    }
    .reverse_run(&mut cx)
    .unwrap();

    // 装箱单
    assert!(StepDelete {
        at: "${Home}/test".to_string(),
        force: None,
    }
    .get_manifest(&mut mixed_fs)
    .is_empty());

    // 变量解释
    assert_eq!(
        StepDelete {
            at: "${Home}".to_string(),
            force: None,
        }
        .interpret(|s| s.replace("${Home}", "C:/Users/Nep")),
        StepDelete {
            at: "C:/Users/Nep".to_string(),
            force: None,
        }
    );

    // 校验
    assert!(StepDelete {
        at: "C:/Users/Desktop".to_string(),
        force: None,
    }
    .verify_self(&"".to_string())
    .is_err());

    assert!(StepDelete {
        at: "${OtherDesktop}".to_string(),
        force: None,
    }
    .verify_self(&"".to_string())
    .is_err());
}
