use crate::{
    executor::{judge_perm_level, values_validator_path},
    log,
    types::{
        mixed_fs::MixedFS,
        permissions::{Generalizable, Permission, PermissionKey},
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
    /// 新建位置，以 `/` 结尾表示新建一个文件夹。
    //# ```toml
    //# # 创建空文件
    //# at = "./empty.txt"
    //#
    //# # 创建文件夹
    //# at = "${Desktop}/Microsoft/"
    //# ```
    //@ 是合法路径
    //@ 不包含通配符
    pub at: String,
    /// 是否覆盖，缺省为 false。
    //# `overwrite = true`
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
        //- 新建文件/文件夹。
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
        if self.at.ends_with('/') {
            new_dir(&self.at)?;
            log!("Info(New):Created directory '{at}'", at = self.at);
        } else {
            new_file(&self.at)?;
            log!("Info(New):Created file '{at}'", at = self.at);
        }

        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        fs.add(&self.at, "");
        Vec::new()
    }
    fn verify_step(&self, _ctx: &super::VerifyStepCtx) -> Result<()> {
        values_validator_path(&self.at)
            .map_err(|e| anyhow!("Error(New):Failed to validate field 'at' as valid path : {e}"))?;
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

impl Generalizable for StepNew {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
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
    use crate::utils::flags::{set_flag, Flag};
    use std::fs::metadata;
    use std::path::Path;
    set_flag(Flag::Debug, true);
    let mut cx = WorkflowContext::_demo();
    if Path::new("test").exists() {
        std::fs::remove_dir_all("test").unwrap();
    }

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

#[test]
fn test_new_corelation() {
    let mut cx = WorkflowContext::_demo();
    let mut mixed_fs = MixedFS::new("");

    // 反向工作流
    StepNew {
        at: "test/1.rs".to_string(),
        overwrite: None,
    }
    .reverse_run(&mut cx)
    .unwrap();

    // 装箱单
    assert!(StepNew {
        at: "${Home}/test".to_string(),
        overwrite: None,
    }
    .get_manifest(&mut mixed_fs)
    .is_empty());

    // 变量解释
    assert_eq!(
        StepNew {
            at: "${Home}".to_string(),
            overwrite: None,
        }
        .interpret(|s| s.replace("${Home}", "C:/Users/Nep")),
        StepNew {
            at: "C:/Users/Nep".to_string(),
            overwrite: None,
        }
    );

    // 校验
    let ctx = crate::types::steps::VerifyStepCtx::_demo();
    assert!(StepNew {
        at: "C:/Users/Desktop".to_string(),
        overwrite: None,
    }
    .verify_step(&ctx)
    .is_err());
    assert!(StepNew {
        at: "C:/Users/Desktop/*".to_string(),
        overwrite: None,
    }
    .verify_step(&ctx)
    .is_err());

    assert!(StepNew {
        at: "${OtherDesktop}".to_string(),
        overwrite: None,
    }
    .verify_step(&ctx)
    .is_err());
}
