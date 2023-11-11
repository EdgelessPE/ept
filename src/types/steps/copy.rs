use super::TStep;
use crate::{
    executor::{judge_perm_level, values_validator_path},
    log, p2s,
    types::{
        interpretable::Interpretable,
        mixed_fs::MixedFS,
        permissions::Generalizable,
        permissions::{Permission, PermissionKey},
        verifiable::Verifiable,
        workflow::WorkflowContext,
    },
    utils::{
        fs::{copy_dir, ensure_dir_exist, try_recycle},
        path::parse_relative_path_with_located,
        wild_match::{common_wild_match_verify, contains_wild_match, parse_wild_match},
    },
};
use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepCopy {
    /// 源路径，支持相对路径和绝对路径，支持使用通配符。
    //# ```toml
    //# # 相对路径写法
    //# from = "./examples/config.toml"
    //#
    //# # 绝对路径通配符写法
    //# from = "${SystemDrive}/Windows/system32/*.dll"
    //# ```
    //@ 是合法路径
    //@ 符合通配符用法
    pub from: String,
    /// 目标路径，支持相对路径和绝对路径。
    //# `to = "./config"`
    //@ 是合法路径
    //@ 不包含通配符
    pub to: String,
    /// 是否覆盖，缺省为 `false`。
    //# `overwrite = true`
    pub overwrite: Option<bool>,
}

// 入参不应包含通配符，返回 （指向父目录存在的目标路径，是否在拷贝文件）
pub fn parse_target_for_copy(
    from: &String,
    to: &String,
    located: &String,
    wild_match_mode: bool,
    step_name: &str,
) -> Result<(PathBuf, bool)> {
    let from_path = parse_relative_path_with_located(from, located);
    let to_path = parse_relative_path_with_located(to, located);

    // 如果 from 不存在直接报错
    if !from_path.exists() {
        return Err(anyhow!(
            "Error({step_name}):Field 'from' refers to a non-existent target : '{from}'"
        ));
    }

    // 如果 from 以 / 结尾但不是目录则报错
    if from.ends_with("/") && !from_path.is_dir() {
        return Err(anyhow!("Error({step_name}):Field 'from' ends with '/' but doesn't refer to a directory : '{from}'"));
    }

    // 处理通配模式，将 to 作为父目录
    if wild_match_mode {
        let file_name = from_path.file_name().unwrap();
        ensure_dir_exist(&to_path)?;
        return Ok((to_path.join(file_name).to_path_buf(), from_path.is_file()));
    }

    // 如果 from 是文件夹，则 to 直接视为文件夹
    if from_path.is_dir() {
        ensure_dir_exist(to_path.parent().unwrap())?;
        return Ok((to_path.to_path_buf(), false));
    } else {
        // 此时拷贝的内容为文件，需要确定 to 的性质然后决定是否需要拼接文件名

        // 如果 to 已存在则直接进行判断
        if to_path.exists() {
            if to_path.is_file() {
                return Ok((to_path.to_path_buf(), true));
            } else if to_path.is_dir() {
                let file_name = from_path.file_name().unwrap();
                return Ok((to_path.join(file_name).to_path_buf(), true));
            } else {
                return Err(anyhow!(
                    "Error({step_name}):Field 'to' refers to a existing abnormal target : '{to}'"
                ));
            }
        }

        // 从字面规则判断 to 是否为文件夹
        if to.ends_with("/") {
            // 此时 from 是文件，说明 to 指向的是父目录，因此进行拼接
            let file_name = from_path.file_name().unwrap();
            ensure_dir_exist(&to_path)?;
            return Ok((to_path.join(file_name).to_path_buf(), true));
        }

        // 兜底，表示 to 是文件路径
        ensure_dir_exist(to_path.parent().unwrap())?;
        return Ok((to_path.to_path_buf(), true));
    }
}

fn copy(
    from: &String,
    to: &String,
    located: &String,
    overwrite: bool,
    wild_match_mode: bool,
) -> Result<()> {
    let (to_path, is_copy_file) =
        parse_target_for_copy(from, to, located, wild_match_mode, "Copy")?;
    if to_path.exists() {
        if overwrite {
            log!("Warning(Copy):Target '{to}' exists, overwriting");
            try_recycle(&to_path)?;
        } else {
            // 如果不覆盖则不需要复制
            log!("Warning(Copy):Target '{to}' exists, enable field 'overwrite' to process still");
            return Ok(());
        }
    }
    if is_copy_file {
        std::fs::copy(from, &to_path).map_err(|e| {
            anyhow!(
                "Error(Copy):Failed to copy file from '{from}' to '{to_str}' : {e}",
                to_str = p2s!(to_path)
            )
        })?;
    } else {
        copy_dir(from, &to_path)?;
    }

    Ok(())
}

impl TStep for StepCopy {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        //- 复制文件/文件夹。
        let overwrite = self.overwrite.unwrap_or(false);
        if contains_wild_match(&self.from) {
            for from in parse_wild_match(self.from, &cx.located)? {
                copy(&p2s!(from), &self.to, &cx.located, overwrite, true)?;
            }
        } else {
            copy(&self.from, &self.to, &cx.located, overwrite, false)?;
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

impl Interpretable for StepCopy {
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

impl Generalizable for StepCopy {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![
            Permission {
                key: PermissionKey::fs_read,
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

impl Verifiable for StepCopy {
    fn verify_self(&self, located: &String) -> Result<()> {
        values_validator_path(&self.from)?;
        values_validator_path(&self.to)?;
        common_wild_match_verify(&self.from, &self.to, located)?;
        // 检查 to 是否包含通配符
        if contains_wild_match(&self.to) {
            return Err(anyhow!(
                "Error(Copy):Field 'to' shouldn't contain wild match : '{to}'",
                to = &self.to
            ));
        }

        Ok(())
    }
}

#[test]
fn test_copy() {
    use std::fs::remove_dir_all;
    use std::path::Path;
    envmnt::set("DEBUG", "true");
    let mut cx = WorkflowContext::_demo();
    remove_dir_all("test").unwrap();

    // 文件-文件
    StepCopy {
        from: "src/types/author.rs".to_string(),
        to: "test/1.rs".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/1.rs").exists());

    // 文件-不存在目录
    StepCopy {
        from: "src/types/extended_semver.rs".to_string(),
        to: "test/ca/".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/ca/extended_semver.rs").exists());

    // 文件-已存在目录
    StepCopy {
        from: "src/types/info.rs".to_string(),
        to: "test/ca".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/ca/info.rs").exists());

    // 目录-不存在目录
    StepCopy {
        from: "src/entrances".to_string(),
        to: "test/entry1".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/entry1/utils/mod.rs").exists());

    // 目录-不存在目录
    StepCopy {
        from: "src/entrances".to_string(),
        to: "test/entry2/".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/entry2/utils/mod.rs").exists());

    // 目录-已存在目录
    StepCopy {
        from: "src/entrances".to_string(),
        to: "test/entry2/entrances".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/entry2/entrances/utils/mod.rs").exists());

    // 通配符文件-不存在目录
    StepCopy {
        from: "src/*.rs".to_string(),
        to: "test/main".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/main/main.rs").exists());

    // 通配符目录-目录
    StepCopy {
        from: "key?".to_string(),
        to: "test/keys".to_string(),
        overwrite: None,
    }
    .run(&mut cx)
    .unwrap();
    assert!(Path::new("test/keys/keys/public.pem").exists());
}
