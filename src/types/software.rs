use crate::{
    log, p2s,
    utils::{
        exe_version::get_exe_version, is_starts_with_inner_value, is_url,
        path::parse_relative_path_with_located,
    },
    verify_enum,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{interpretable::Interpretable, mixed_fs::MixedFS, verifiable::Verifiable};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, TS, PartialEq)]
#[ts(export)]
pub struct Software {
    pub scope: String,
    pub upstream: String,
    pub category: String,
    pub arch: Option<String>,
    pub language: String,
    pub main_program: Option<String>,
    pub tags: Option<Vec<String>>,
    pub alias: Option<Vec<String>>,
    pub registry_entry: Option<String>,
}

impl Verifiable for Software {
    fn verify_self(&self, located: &String) -> Result<()> {
        let err_wrapper = |e: anyhow::Error| {
            anyhow!("Error:Failed to verify table 'software' in 'package.toml' : {e}")
        };

        // 检查 arch 枚举
        if let Some(arch) = &self.arch {
            verify_enum!("arch", arch, "X64" | "X86" | "ARM64").map_err(err_wrapper)?;
        }

        // 检查 language 枚举
        verify_enum!("language", &self.language, "Multi" | "zh-CN" | "en-US")
            .map_err(err_wrapper)?;

        // 上游必须是 URL
        if !is_url(&self.upstream) {
            return Err(err_wrapper(anyhow!(
                "upstream should be a valid url, got '{text}'",
                text = self.upstream
            )));
        }

        if let Some(main_program) = &self.main_program {
            // 区分是绝对路径还是相对路径，仅校验相对路径的主程序
            if !is_starts_with_inner_value(main_program) {
                // 相对路径的主程序应该存在
                let mixed_fs = MixedFS::new(located.to_owned());
                if !mixed_fs.exists(main_program) {
                    return Err(err_wrapper(anyhow!(
                        "given main program '{main_program}' doesn't exist"
                    )));
                }

                // 对于相对路径的主程序，尝试进行读取
                let mp_path = parse_relative_path_with_located(main_program, located);
                if mp_path.exists() {
                    if let Err(e) = get_exe_version(p2s!(mp_path)) {
                        // 读不了版本号则警告
                        log!(
                            "Warning:Failed to get main program ('{main_program}') file version : {e}"
                        );
                    }
                }
            }
        }

        // tags 不应该 software 表中的字段重复
        let mut alias = self
            .alias
            .to_owned()
            .unwrap_or(Vec::new())
            .into_iter()
            .map(|tag| ("alias", tag))
            .collect();
        let mut fields = vec![
            ("scope", self.scope.to_owned()),
            ("category", self.category.to_owned()),
        ];
        fields.append(&mut alias);
        let tag_checker = |tag: &String| {
            for (field, text) in fields.clone() {
                if text.contains(tag) {
                    return Err(anyhow!("Error:Value '{tag}' in field 'tags' contains duplicated key word found in field '{field}' : '{text}'"));
                }
            }

            Ok(())
        };
        for tag in self.tags.to_owned().unwrap_or(Vec::new()) {
            tag_checker(&tag)?;
        }

        Ok(())
    }
}

impl Interpretable for Software {
    fn interpret<F>(mut self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        self.main_program = self.main_program.map(interpreter);
        self
    }
}

#[test]
fn test_verify_software() {
    use crate::types::package::GlobalPackage;
    let located = "".to_string();
    let base = GlobalPackage::_demo().software.unwrap();
    assert!(base.verify_self(&located).is_ok());

    // 校验架构
    let mut s1 = base.clone();
    s1.arch = Some("X32".to_string());
    assert!(s1.verify_self(&located).is_err());

    // 校验语言
    let mut s2 = base.clone();
    s2.language = "ZH-CN".to_string();
    assert!(s2.verify_self(&located).is_err());

    // 校验 tags 重复
    let mut s3 = base.clone();
    s3.tags = Some(vec!["Visual Studio".to_string(), "Microsoft".to_string()]);
    s3.alias = Some(vec!["Visual Studio Code".to_string()]);
    assert!(s3.verify_self(&located).is_err());
    s3.alias = None;
    assert!(s3.verify_self(&located).is_ok());
    s3.scope = "Microsoft".to_string();
    assert!(s3.verify_self(&located).is_err());
}
