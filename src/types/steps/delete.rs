use std::{ffi::OsString, path::Path};

use super::TStep;
use crate::types::steps::Permission;
use crate::{
    executor::{judge_perm_level, values_validator_path},
    log, p2s,
    types::{
        mixed_fs::MixedFS, permissions::Generalizable, verifiable::Verifiable,
        workflow::WorkflowContext,
    },
    utils::{contains_wild_match, parse_wild_match, try_recycle},
};
use anyhow::{anyhow, Result};
use force_delete_win::force_delete_file_folder;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepDelete {
    pub at: String,
    pub force: Option<bool>,
}

fn delete(target: &String, force: bool) -> Result<()> {
    let p = Path::new(target);
    if !p.exists() {
        return Err(anyhow!("Error(Delete):Target '{target}' not exist"));
    }
    if let Err(e) = try_recycle(p) {
        if force {
            if force_delete_file_folder(OsString::from(target)) {
                log!("Warning(Delete):Force deleted '{target}'");
                Ok(())
            } else {
                return Err(anyhow!(
                    "Error(Delete):Failed to force delete '{target}' : '{e}'"
                ));
            }
        } else {
            return Err(anyhow!("Error(Delete):Failed to delete '{target}' : '{e}', enable field 'force' to try shredding"));
        }
    } else {
        Ok(())
    }
}

impl TStep for StepDelete {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
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
        values_validator_path(&self.at)?;

        Ok(())
    }
}

impl Generalizable for StepDelete {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![Permission {
            key: "fs_write".to_string(),
            level: judge_perm_level(&self.at)?,
            targets: vec![self.at.clone()],
        }])
    }
}

#[test]
fn test_delete() {
    use fs_extra::dir::CopyOptions;
    envmnt::set("DEBUG", "true");
    let mut cx = WorkflowContext::_demo();
    crate::utils::test::_ensure_clear_test_dir();

    // 准备源
    let opt = CopyOptions::new().copy_inside(true);
    fs_extra::dir::copy("src", "test/src", &opt).unwrap();

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
