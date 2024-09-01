use crate::types::software::Software;
use crate::verify_enum;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use ts_rs::TS;

use super::{extended_semver::ExSemVer, interpretable::Interpretable, verifiable::Verifiable};

#[derive(Serialize, Deserialize, Clone, Debug, TS, PartialEq)]
#[ts(export)]
pub struct Package {
    /// 包名，推荐仅使用英文、中文和空格。
    /// 不得包含下划线（`_`），请使用空格或横杠线（`-`）代替。
    //# `name = "VSCode"`
    pub name: String,
    /// 包的简短描述，尽量从官方渠道摘取简介。
    //# `description = "微软开发的跨平台开源编辑器"`
    pub description: String,
    /// 包模板，当前版本中仅能为 "Software"。
    //# `template = "Software"`
    pub template: String,
    /// 包版本号，使用 ExSemVer 规范。
    /// ExSemVer 规范在 [SemVer](https://semver.org) 的基础上在`PATCH`和`PRE`之间增加了一位`RESERVED`位，用于标记不同的打包版本，或是用来兼容在 Windows 平台常见的 4 位版本号；若上游版本号符合 SemVer 规范则将`RESERVED`位置`0`即可。
    //# `version = "1.0.0.0"`
    pub version: String,
    /// 包作者，第一作者应为打包者，后面通常跟发行商、制作方。
    /// 支持使用 `<>` 包裹作者邮箱。
    //# `authors = ["Cno <dsyourshy@qq.com>", "Microsoft"]`
    pub authors: Vec<String>,
    /// 开源许可证的 [SPDX 标识符](https://spdx.org/licenses/)或 EULA 链接。
    //# `license = "MIT"`
    pub license: Option<String>,
    /// 包图标 URL。
    //# `icon = "https://code.visualstudio.com/favicon.ico"`
    pub icon: Option<String>,
    /// 是否使用严格模式，缺省为`true`。
    /// 启用严格模式时，如果某一步骤出错则工作流会立即停止执行并报告错误；否则工作流只会对错误进行警告然后继续运行后续步骤。
    ///
    /// :::warning
    /// 注意如果希望使用内置变量[`ExitCode`](/nep/definition/2-context#exitcode)，请将`strict`设置为`false`。
    /// :::
    //# `strict = false`
    pub strict: Option<bool>,
}

impl Verifiable for Package {
    fn verify_self(&self, _: &String) -> Result<()> {
        let err_wrapper = |e: anyhow::Error| {
            anyhow!("Error:Failed to verify table 'package' in 'package.toml' : {e}")
        };

        // name 不能包含下划线
        if self.name.contains('_') {
            return Err(err_wrapper(anyhow!(
                "field 'name' shouldn't contain underline (_), got '{n}'",
                n = &self.name
            )));
        }

        // 模板只能是 Software
        verify_enum!("template", &self.template, "Software").map_err(err_wrapper)?;

        // 版本号必须可以解析
        ExSemVer::parse(&self.version).map_err(err_wrapper)?;

        Ok(())
    }
}

impl Interpretable for Package {
    fn interpret<F>(self, _interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS, PartialEq)]
#[ts(export)]
pub struct GlobalPackage {
    pub nep: String,
    pub package: Package,
    pub software: Option<Software>,
}

impl GlobalPackage {
    pub fn _demo() -> Self {
        GlobalPackage {
            nep: env!("CARGO_PKG_VERSION")[0..3].to_string(),
            package: Package {
                name: "ept".to_string(),
                description: "demo package".to_string(),
                template: "Software".to_string(),
                version: "1.0.0".to_string(),
                authors: vec!["Cno".to_string()],
                license: None,
                icon: None,
                strict: None,
            },
            software: Some(Software {
                scope: "Edgeless".to_string(),
                upstream: "https://github.com/EdgelessPE/ept".to_string(),
                category: "实用工具".to_string(),
                arch: None,
                language: "en-US".to_string(),
                main_program: None,
                tags: None,
                alias: None,
                registry_entry: None,
            }),
        }
    }
}

impl Verifiable for GlobalPackage {
    fn verify_self(&self, located: &String) -> Result<()> {
        if !Path::new(located).exists() {
            return Err(anyhow!("Error:Path '{located}' not exist"));
        }
        self.package.verify_self(located)?;
        if let Some(software) = &self.software {
            software.verify_self(located)?;
        }

        Ok(())
    }
}

impl Interpretable for GlobalPackage {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            nep: self.nep,
            package: self.package.interpret(&interpreter),
            software: self.software.map(|soft| soft.interpret(interpreter)),
        }
    }
}
