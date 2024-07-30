use super::{copy::parse_target_for_copy, TStep};
use crate::types::interpretable::Interpretable;
use crate::types::permissions::PermissionKey;
use crate::{
    executor::{judge_perm_level, values_validator_path},
    log, p2s,
    types::{
        mixed_fs::MixedFS, permissions::Generalizable, permissions::Permission,
        verifiable::Verifiable, workflow::WorkflowContext,
    },
    utils::{
        fs::try_recycle,
        wild_match::{common_wild_match_verify, contains_wild_match, parse_wild_match},
    },
};
use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepMove {
    /// 源路径，支持相对路径和绝对路径，支持使用通配符。
    //# ```toml
    //# # 相对路径写法
    //# from = "./runtime/lib.dll"
    //#
    //# # 绝对路径通配符写法
    //# from = "${Desktop}/Visual Studio Code.lnk"
    //# ```
    //@ 是合法路径
    //@ 符合通配符用法
    pub from: String,
    /// 目标路径，支持相对路径和绝对路径。
    //# `to = "./bin"`
    //@ 是合法路径
    //@ 不包含通配符
    pub to: String,
    /// 是否覆盖，缺省为 `false`。
    //# `overwrite = true`
    pub overwrite: Option<bool>,
}

fn mv(
    from: &String,
    to: &String,
    located: &String,
    overwrite: bool,
    wild_match_mode: bool,
) -> Result<()> {
    let (to_path, _) = parse_target_for_copy(from, to, located, wild_match_mode, "Move")?;
    if to_path.exists() {
        if overwrite {
            log!("Warning(Move):Target '{to}' exists, overwriting");
            try_recycle(&to_path)?;
        } else {
            // 如果不覆盖则不需要移动
            log!("Warning(Move):Ignoring due to target '{to}' exists, enable field 'overwrite' to process still");
            return Ok(());
        }
    }
    std::fs::rename(from, &to_path).map_err(|e| {
        anyhow!(
            "Error:Failed to move file from '{from}' to '{to_str}' : {e}",
            to_str = p2s!(to_path)
        )
    })?;

    log!("Info(Move):Moved '{from}' to '{to}'", to = p2s!(to_path));
    Ok(())
}

impl TStep for StepMove {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        //- 移动文件/文件夹。
        let overwrite = self.overwrite.unwrap_or(false);
        if contains_wild_match(&self.from) {
            for from in parse_wild_match(self.from, &cx.located)? {
                mv(&p2s!(from), &self.to, &cx.located, overwrite, true)?;
            }
        } else {
            mv(&self.from, &self.to, &cx.located, overwrite, false)?;
        }

        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        fs.remove(&self.from);
        fs.add(&self.to, &self.from);
        Vec::new()
    }
}

impl Interpretable for StepMove {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            from: interpreter(self.from),
            to: interpreter(self.to),
            overwrite: self.overwrite,
        }
    }
}

impl Generalizable for StepMove {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![
            Permission {
                key: PermissionKey::fs_write,
                level: judge_perm_level(&self.from)?,
                targets: vec![self.from.clone()],
            },
            Permission {
                key: PermissionKey::fs_write,
                level: judge_perm_level(&self.to)?,
                targets: vec![self.to.clone()],
            },
        ])
    }
}

impl Verifiable for StepMove {
    fn verify_self(&self, located: &String) -> Result<()> {
        values_validator_path(&self.from).map_err(|e| {
            anyhow!("Error(Move):Failed to validate field 'from' as valid path : {e}")
        })?;
        values_validator_path(&self.to).map_err(|e| {
            anyhow!("Error(Move):Failed to validate field 'to' as valid path : {e}")
        })?;
        common_wild_match_verify(&self.from, &self.to, located)
    }
}

#[test]
fn test_copy() {
    use crate::utils::fs::copy_dir;
    use std::path::Path;
    envmnt::set("DEBUG", "true");
    let mut cx = WorkflowContext::_demo();
    crate::utils::test::_ensure_clear_test_dir();

    // 准备源
    copy_dir("src", "test/src").unwrap();
    copy_dir("keys", "test/src/keys").unwrap();

    // 文件-文件
    StepMove {
        from: "test/src/types/author.rs".to_string(),
        to: "test/1.rs".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/1.rs").exists());
    assert!(!Path::new("test/src/types/author.rs").exists());

    // 文件-覆盖文件
    StepMove {
        from: "test/src/types/steps/mv.rs".to_string(),
        to: "test/1.rs".to_string(),
        overwrite: Some(true),
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/1.rs").exists());
    assert!(!Path::new("test/src/types/steps/mv.rs").exists());

    // 文件-隐式跳过文件
    StepMove {
        from: "test/src/types/steps/copy.rs".to_string(),
        to: "test/1.rs".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/1.rs").exists());
    assert!(Path::new("test/src/types/steps/copy.rs").exists());

    // 文件-显式跳过文件
    StepMove {
        from: "test/src/types/steps/copy.rs".to_string(),
        to: "test/1.rs".to_string(),
        overwrite: Some(false),
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/1.rs").exists());
    assert!(Path::new("test/src/types/steps/copy.rs").exists());

    // 文件-不存在目录
    StepMove {
        from: "test/src/types/extended_semver.rs".to_string(),
        to: "test/ca/".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/ca/extended_semver.rs").exists());
    assert!(!Path::new("test/src/types/extended_semver.rs").exists());

    // 文件-已存在目录
    StepMove {
        from: "test/src/types/mod.rs".to_string(),
        to: "test/ca".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/ca/mod.rs").exists());
    assert!(!Path::new("test/src/types/mod.rs").exists());

    // 文件-已存在目录覆盖
    StepMove {
        from: "test/src/types/steps/mod.rs".to_string(),
        to: "test/ca".to_string(),
        overwrite: Some(true),
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/ca/mod.rs").exists());
    assert!(!Path::new("test/src/types/steps/mod.rs").exists());

    // 目录-不存在目录
    StepMove {
        from: "test/src/entrances".to_string(),
        to: "test/entry1".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/entry1/utils/mod.rs").exists());
    assert!(!Path::new("test/src/entrances").exists());

    // 目录-不存在目录
    StepMove {
        from: "test/src/ca".to_string(),
        to: "test/entry2/".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/entry2/mod.rs").exists());
    assert!(!Path::new("test/src/ca").exists());

    // 目录-已存在目录
    StepMove {
        from: "test/src/executor".to_string(),
        to: "test/entry2/entrances".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/entry2/entrances/README.md").exists());
    assert!(!Path::new("test/src/executor").exists());

    // 目录-已存在目录覆盖
    StepMove {
        from: "test/src/compression".to_string(),
        to: "test/entry2/entrances".to_string(),
        overwrite: Some(true),
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/entry2/entrances/tar.rs").exists());
    assert!(!Path::new("test/entry2/entrances/README.md").exists());
    assert!(!Path::new("test/src/compression/mod.rs").exists());

    // 通配符文件-不存在目录
    StepMove {
        from: "test/src/*.rs".to_string(),
        to: "test/main".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/main/main.rs").exists());
    assert!(!Path::new("test/src/main.rs").exists());

    // 通配符目录-目录
    StepMove {
        from: "test/src/key?".to_string(),
        to: "test/keys".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/keys/keys/public.pem").exists());
    assert!(!Path::new("test/src/keys/public.pem").exists());
}

#[test]
fn test_move_corelation() {
    let mut cx = WorkflowContext::_demo();

    // 反向工作流
    StepMove {
        from: "test/src/types/author.rs".to_string(),
        to: "test/1.rs".to_string(),
        overwrite: None,
    }
    .reverse_run(&mut cx)
    .unwrap();

    // 变量解释
    assert_eq!(
        StepMove {
            from: "${Home}".to_string(),
            to: "${Desktop}".to_string(),
            overwrite: None
        }
        .interpret(|s| s
            .replace("${Home}", "C:/Users/Nep")
            .replace("${Desktop}", "C:/Users/Nep/Desktop")),
        StepMove {
            from: "C:/Users/Nep".to_string(),
            to: "C:/Users/Nep/Desktop".to_string(),
            overwrite: None
        }
    );

    // 校验
    assert!(StepMove {
        from: "C:/Users/Desktop".to_string(),
        to: "${Desktop}".to_string(),
        overwrite: None
    }
    .verify_self(&"".to_string())
    .is_err());

    assert!(StepMove {
        from: "${Home}".to_string(),
        to: "C:/Users/Nep/Desktop".to_string(),
        overwrite: None
    }
    .verify_self(&"".to_string())
    .is_err());
    assert!(StepMove {
        from: "${Home}".to_string(),
        to: "${Desktop}/*".to_string(),
        overwrite: None
    }
    .verify_self(&"".to_string())
    .is_err());
}
