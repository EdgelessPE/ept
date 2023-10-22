use crate::{
    executor::{judge_perm_level, values_validator_path},
    log,
    types::{
        mixed_fs::MixedFS,
        permissions::{Generalizable, Permission, PermissionKey},
        verifiable::Verifiable,
        workflow::WorkflowContext,
    },
    utils::wild_match::contains_wild_match,
};
use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};
use std::{fs::create_dir_all, fs::File, path::Path};

use super::TStep;
use crate::types::interpretable::Interpretable;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepNew {
    pub at: String,
    pub overwrite: Option<bool>,
}

fn new_file(at: &String) -> Result<()> {
    File::create(at).map_err(|e| anyhow!("Error(New):Failed to create file at '{at}' : {e}"))?;

    Ok(())
}

fn new_dir(at: &String) -> Result<()> {
    create_dir_all(at)
        .map_err(|e| anyhow!("Error(New):Failed to create directory at '{at}' : {e}"))?;

    Ok(())
}

impl TStep for StepNew {
    fn run(self, _: &mut WorkflowContext) -> Result<i32> {
        // 检测是否存在
        let p = Path::new(&self.at);
        if p.exists() {
            if !self.overwrite.unwrap_or(false) {
                log!("Warning(New):Target '{at}' already exists, enable field 'overwrite' to process still",at=self.at);
                return Ok(0);
            } else {
                log!(
                    "Warning(New):Target '{at}' already exists, overwrite",
                    at = self.at
                );
            }
        }

        // 分流处理
        if self.at.ends_with("/") {
            new_dir(&self.at)?
        } else {
            new_file(&self.at)?
        }

        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        fs.add(&self.at, &"".to_string());
        Vec::new()
    }
}

impl Interpretable for StepNew {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            at: interpreter(self.at),
            overwrite: self.overwrite,
        }
    }
}

impl Verifiable for StepNew {
    fn verify_self(&self, _: &String) -> Result<()> {
        values_validator_path(&self.at)?;
        // 检查 at 是否包含通配符
        if contains_wild_match(&self.at) {
            return Err(anyhow!(
                "Error(New):Field 'at' shouldn't contain wild match : '{at}'",
                at = &self.at
            ));
        }

        Ok(())
    }
}

impl Generalizable for StepNew {
    fn generalize_permissions(&self) -> Result<Vec<crate::types::permissions::Permission>> {
        Ok(vec![Permission {
            key: PermissionKey::fs_write,
            level: judge_perm_level(&self.at)?,
            targets: vec![self.at.clone()],
        }])
    }
}

#[test]
fn test_new() {
    use crate::types::workflow::WorkflowContext;
    use std::fs::metadata;
    use std::path::Path;
    envmnt::set("DEBUG", "true");
    let mut cx = WorkflowContext::_demo();
    crate::utils::test::_ensure_clear_test_dir();

    // 创建目录和文件
    StepNew {
        at: "test/".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    StepNew {
        at: "test/1.txt".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/1.txt").exists());

    // 文件覆盖
    std::fs::copy("src/main.rs", "test/main.rs").unwrap();
    StepNew {
        at: "test/main.rs".to_string(),
        overwrite: Some(true),
    }
    .run(&mut cx)
    .unwrap();
    let meta = metadata("test/main.rs").unwrap();
    assert!(meta.len() < 16);

    // 目录覆盖
    StepNew {
        at: "test".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/1.txt").exists());
}
