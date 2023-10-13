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
    /// 包名
    //# `name = "VSCode"`
    pub name: String,
    /// 包的简短描述，尽量从官方渠道摘取
    //# `description = "微软出品的开源编辑器"`
    pub description: String,
    /// 包模板，当前版本中仅能为 "Software"
    //# `template = "Software"`
    pub template: String,
    /// 包版本号，使用 ExSemVer 规范
    //# `version = "1.0.0.0"`
    pub version: String,
    /// 包作者，第一作者应为打包者，后面通常跟发行商、制作方
    /// 支持使用 `<>` 包裹作者邮箱
    //# `authors = ["Cno <dsyourshy@qq.com>", "Microsoft"]`
    pub authors: Vec<String>,
    /// 开源许可证的 [SPDX 标识符](https://spdx.org/licenses/)或 EULA 链接
    //# `license = "MIT"`
    pub license: Option<String>,
    /// 包图标 URL
    //# `icon = "https://code.visualstudio.com/favicon.ico"`
    pub icon: Option<String>,
}

impl Verifiable for Package {
    fn verify_self(&self, _: &String) -> Result<()> {
        let err_wrapper = |e: anyhow::Error| {
            anyhow!("Error:Failed to verify table 'package' in 'package.toml' : {e}")
        };

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
