use std::{fs::remove_dir_all, path::Path};

use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::types::interpretable::Interpretable;
use crate::types::permissions::PermissionKey;
use crate::{
    executor::{judge_perm_level, values_validator_path},
    log, p2s,
    types::{
        permissions::{Generalizable, Permission},
        verifiable::Verifiable,
    },
    utils::{
        path::{parse_relative_path_with_located, split_parent},
        wild_match::contains_wild_match,
    },
};

use super::TStep;

lazy_static! {
    static ref PURE_NAME_NOT_MATCH_REGEX: Regex = Regex::new(r"[\\\/\*\:\$]").unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepRename {
    /// 目标路径，支持相对路径和绝对路径。
    //# `from = "./config.toml.example"`
    //@ 是合法路径
    //@ 不包含通配符
    pub from: String,
    /// 新的名称。
    //# `to = "config.toml"`
    //@ 是合法的文件名或文件夹名而非路径
    pub to: String,
}

// 将to的文件名替代拼接到from末尾
fn concat_to(to: &String, from: &str, located: &String) -> String {
    let (parent, _) = split_parent(from, located);
    p2s!(parent.join(to))
}

fn rename(from: &String, to: &String, located: &String) -> Result<()> {
    let from_path = parse_relative_path_with_located(from, located);
    // 检查是否存在
    if !from_path.exists() {
        return Err(anyhow!(
            "Error(Rename):Field 'from' refers to a non-existent target : '{from}'"
        ));
    }

    // 拼接to path
    let final_to = concat_to(to, from, located);

    // 清理存在的目录
    let to_path = Path::new(&final_to);
    if to_path.exists() {
        log!("Warning(Rename):Target '{to}' exists, overwriting");
        if to_path.is_dir() {
            remove_dir_all(to_path)
                .map_err(|e| anyhow!("Error(Rename):Failed to remove '{final_to}' : {e}"))?;
        }
    }

    // 执行重命名
    std::fs::rename(&from_path, &final_to).map_err(|e| {
        anyhow!("Error(Rename):Error:Failed to rename '{from}' to '{final_to}' : {e}")
    })?;

    log!(
        "Info(Rename):Renamed '{from}' to '{final_to}'",
        from = p2s!(from_path)
    );
    Ok(())
}

impl TStep for StepRename {
    fn run(self, cx: &mut crate::types::workflow::WorkflowContext) -> Result<i32> {
        //- 重命名文件/文件夹。
        rename(&self.from, &self.to, &cx.located)?;
        Ok(0)
    }
    fn reverse_run(self, _: &mut crate::types::workflow::WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut crate::types::mixed_fs::MixedFS) -> Vec<String> {
        fs.remove(&self.from);
        fs.add(&concat_to(&self.to, &self.from, &String::new()), &self.from);
        Vec::new()
    }
}

impl Interpretable for StepRename {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            from: interpreter(self.from),
            to: interpreter(self.to),
        }
    }
}

impl Generalizable for StepRename {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![Permission {
            key: PermissionKey::fs_write,
            level: judge_perm_level(&self.from)?,
            targets: vec![self.from.clone()],
        }])
    }
}

impl Verifiable for StepRename {
    fn verify_self(&self, _: &String) -> Result<()> {
        values_validator_path(&self.from).map_err(|e| {
            anyhow!("Error(Rename):Failed to validate field 'from' as valid path : {e}")
        })?;
        // 检查 from 是否包含通配符
        if contains_wild_match(&self.from) {
            return Err(anyhow!(
                "Error(Rename):Field 'from' shouldn't contain wild match : '{from}'",
                from = &self.from
            ));
        }
        // 检查 to 的正则表达式
        if PURE_NAME_NOT_MATCH_REGEX.is_match(&self.to) {
            return Err(anyhow!(
                "Error(Rename):Field 'to' illegal, expect pure file or directory name, got '{to}'",
                to = &self.to
            ));
        }

        Ok(())
    }
}

#[test]
fn test_rename() {
    use crate::types::workflow::WorkflowContext;
    use std::path::Path;
    envmnt::set("DEBUG", "true");
    let mut cx = WorkflowContext::_demo();
    crate::utils::test::_ensure_clear_test_dir();

    // 准备源
    crate::utils::fs::copy_dir("src", "test/src").unwrap();

    // 文件
    StepRename {
        from: "test/src/types/author.rs".to_string(),
        to: "1.rs".to_string(),
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/src/types/1.rs").exists());
    assert!(!Path::new("test/src/types/author.rs").exists());

    // 文件覆盖
    StepRename {
        from: "test/src/types/info.rs".to_string(),
        to: "1.rs".to_string(),
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/src/types/1.rs").exists());
    assert!(!Path::new("test/src/types/info.rs").exists());

    // 目录
    StepRename {
        from: "test/src".to_string(),
        to: "source".to_string(),
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/source/types/1.rs").exists());
    assert!(!Path::new("test/src/types/1.rs").exists());

    // 目录覆盖
    StepRename {
        from: "test/source/utils".to_string(),
        to: "tools".to_string(),
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/source/tools/cfg.rs").exists());
    assert!(!Path::new("test/source/utils/cfg.rs").exists());

    StepRename {
        from: "test/source/types".to_string(),
        to: "tools".to_string(),
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/source/tools/steps/rename.rs").exists());
    assert!(!Path::new("test/source/types/steps/rename.rs").exists());
}
