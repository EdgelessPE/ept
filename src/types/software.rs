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
    /// 软件发行域，通常填写上游组织名称。
    /// 若软件包的直接上游为发行商/组织则使用发行商的名称，例如 `PortableApps`；若软件包的直接上游为官方网站则使用开发商/组织的名称，例如 `Microsoft`。
    /// 若上游组织为正式的、拥有独立域名的组织，则将发行域开头大写，例如对于 GitHub 发布的 `GitHub Desktop` 软件使用 `GitHub` 作为发行域；若上游组织表示对一个群体的泛指，则将发行域开头小写，例如对于将发行托管在 GitHub Releases 上的开源项目使用 `github` 作为发行域。
    //# `scope = "Microsoft"`
    pub scope: String,
    /// 软件上游 URL，可以是官方网站的下载页或发行商/组织提供的发行详情页。
    //# `upstream = "https://code.visualstudio.com/"`
    pub upstream: String,
    /// 软件分类，推荐为 Edgeless 插件包分类中的一种。
    //# `category = "集成开发"`
    pub category: String,
    /// 软件的编译目标架构，缺省表示安装时不检查架构兼容性。
    //# `arch = "X64`
    pub arch: Option<String>,
    /// 软件语言，`Multi`表示多语言。
    //# `language = "Multi"`
    pub language: String,
    /// 主程序路径，可以是相对路径或绝对路径。
    /// 如果使用绝对路径，必须以[内置变量](/nep/workflow/2-context.html#内置变量)开头。
    //# ```toml
    //# # 相对路径写法
    //# main_program = "./code.exe"
    //#
    //# # 绝对路径写法
    //# main_program = "${AppData}/Local/Programs/Microsoft VS Code/Code.exe"
    //# ```
    pub main_program: Option<String>,
    /// 标签，用于联想推荐相似包或聚合多个相近的包。
    /// 不需要重复输入包名、分类或是作者名中的信息。
    //# `tags = ["electron", "typescript"]`
    pub tags: Option<Vec<String>>,
    /// 别名，用于关联查找。
    /// 不需要重复输入标签中的信息。
    //# `alias = ["code", "vsc", "Visual Studio Code"]`
    pub alias: Option<Vec<String>>,
    /// 注册表入口，如果该软件是调用安装器安装的且在注册表中有 Uninstall 入口，提供该字段可以免去编写卸载工作流并帮助 ept 获取更多信息。
    /// 支持如下 3 个位置的入口：
    /// ```
    /// HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
    /// HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
    /// HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall
    /// ```
    //# ```toml
    //# # 对应注册表路径 HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Uninstall\{D9E514E7-1A56-452D-9337-2990C0DC4310}_is1
    //# registry_entry = "{D9E514E7-1A56-452D-9337-2990C0DC4310}_is1"
    //# ```
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
            .unwrap_or_default()
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
        for tag in self.tags.to_owned().unwrap_or_default() {
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
