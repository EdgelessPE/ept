use crate::{
    log, p2s,
    types::constants::FILE_PACKAGE,
    utils::{
        exe_version::get_exe_version, is_starts_with_inner_value, is_url,
        path::parse_relative_path_with_located,
    },
    verify_enum,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{
    interpretable::Interpretable,
    verifiable::{Verifiable, VerifiableCtx},
};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, TS, PartialEq)]
#[ts(export)]
pub struct Software {
    /// 软件上游 URL，可以是官方网站的下载页或发行商/组织提供的发行详情页。
    //# `upstream = "https://code.visualstudio.com/"`
    pub upstream: String,
    /// 软件分类，推荐为 Edgeless 插件包分类中的一种。
    //# `category = "集成开发"`
    pub category: String,
    /// 软件的编译目标架构，缺省表示安装时不检查架构兼容性。
    /// :::warning
    /// 镜像源已不再接收目标架构为 `X86` 的软件包，该值仅作兼容用途。
    /// :::
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
    /// 别名，用于关联查找。
    /// 不需要重复输入标签中已有的信息；如果该软件在现实中存在多个别名，请选择一个最常用的进行填写，并将其他别名添加到标签字段中。
    //# `alias = "code"`
    pub alias: Option<String>,
    /// 标签，用于联想推荐相似包或聚合多个相近的包。
    /// 不需要重复输入包名、作者名、分类、别名中的信息。
    //# `tags = ["electron", "typescript"]`
    pub tags: Option<Vec<String>>,
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
    fn verify_self(&self, ctx: &VerifiableCtx) -> Result<()> {
        let err_wrapper = |e: anyhow::Error| {
            anyhow!("Error:Failed to verify table 'software' in '{FILE_PACKAGE}' : {e}",)
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
                if !ctx.mixed_fs.exists(main_program) {
                    return Err(err_wrapper(anyhow!(
                        "given main program '{main_program}' doesn't exist"
                    )));
                }

                // 对于相对路径的主程序，尝试进行读取
                let mp_path = parse_relative_path_with_located(main_program, &ctx.mixed_fs.located);
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

        // tags 不应该与 software 表中的字段重复
        let fields: Vec<(&str, &str)> = [
            Some(("category", self.category.as_str())),
            self.alias.as_ref().map(|a| ("alias", a.as_str())),
        ]
        .into_iter()
        .flatten()
        .collect();

        for tag in self.tags.as_ref().unwrap_or(&vec![]) {
            for (field, text) in &fields {
                if text.contains(tag) {
                    return Err(err_wrapper(anyhow!(
                        "Value '{tag}' in field 'tags' contains duplicated key word found in field '{field}' : '{text}'"
                    )));
                }
            }
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
    use crate::types::{mixed_fs::MixedFS, package::GlobalPackage, verifiable::VerifiableCtx};
    use crate::utils::test::_default_test_cfg;

    let base = GlobalPackage::_demo().software.unwrap();
    let mixed_fs = MixedFS::new("");
    let cfg = _default_test_cfg();
    let ctx = VerifiableCtx {
        mixed_fs: &mixed_fs,
        cfg: &cfg,
    };
    assert!(base.verify_self(&ctx).is_ok());

    // 校验架构
    let mut s1 = base.clone();
    s1.arch = Some("X32".to_string());
    assert!(s1.verify_self(&ctx).is_err());

    // 校验语言
    let mut s2 = base.clone();
    s2.language = "ZH-CN".to_string();
    assert!(s2.verify_self(&ctx).is_err());

    // 校验 tags 重复
    let mut s3 = base.clone();
    s3.tags = Some(vec!["Visual Studio".to_string(), "Microsoft".to_string()]);
    s3.alias = Some("Visual Studio Code".to_string());
    assert!(s3.verify_self(&ctx).is_err());
    s3.alias = None;
    assert!(s3.verify_self(&ctx).is_ok());
}
